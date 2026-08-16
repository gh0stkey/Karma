pub mod config;
pub mod decode;
#[cfg(target_os = "macos")]
pub mod mlx_backend;
#[cfg(not(target_os = "macos"))]
pub mod onnx_backend;

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use tokenizers::Tokenizer;

use config::{load_viterbi_biases, placeholder_map, resolve_model_dir, ModelConfig, ViterbiBiases};
use decode::{extract_spans, viterbi_decode, LabelSpace};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedSpan {
    pub label: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub placeholder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionResult {
    pub schema_version: u32,
    pub text: String,
    pub redacted_text: String,
    pub detected_spans: Vec<DetectedSpan>,
    pub summary: HashMap<String, u32>,
    pub latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub architecture: String,
    pub num_labels: i32,
    pub hidden_size: i64,
    pub vocab_size: i64,
    pub max_position_embeddings: i64,
}

pub trait InferenceBackend: Send {
    fn forward_logits(&mut self, ids: &[u32]) -> Result<Vec<Vec<f32>>>;
}

struct Redactor {
    backend: Box<dyn InferenceBackend>,
    tokenizer: Tokenizer,
    label_space: LabelSpace,
    biases: ViterbiBiases,
    placeholders: HashMap<String, String>,
    info: ModelInfo,
}

impl Redactor {
    fn load(model_path: &Path) -> Result<Self> {
        let model_dir = resolve_model_dir(model_path)?;
        let config = ModelConfig::load(&model_dir)?;

        let tokenizer_path = model_dir.join("tokenizer.json");
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow::anyhow!("failed to load {}: {e}", tokenizer_path.display()))?;

        let biases = load_viterbi_biases(&model_dir);
        let label_space = LabelSpace::from_id2label(&config.id2label)?;
        let placeholders = placeholder_map();
        let info = ModelInfo {
            name: ModelConfig::model_name(&model_dir),
            architecture: config.architecture().to_string(),
            num_labels: config.num_labels() as i32,
            hidden_size: config.hidden_size as i64,
            vocab_size: config.vocab_size as i64,
            max_position_embeddings: config.max_position_embeddings as i64,
        };

        let backend = create_backend(&model_dir, &config)?;

        Ok(Self {
            backend,
            tokenizer,
            label_space,
            biases,
            placeholders,
            info,
        })
    }

    fn redact(&mut self, text: &str) -> Result<RedactionResult> {
        let started = Instant::now();

        let encoding = self
            .tokenizer
            .encode(text, false)
            .map_err(|e| anyhow::anyhow!("tokenization failed: {e}"))?;
        let ids = encoding.get_ids().to_vec();
        let offsets = encoding.get_offsets().to_vec();

        let chars: Vec<char> = text.chars().collect();
        let mut byte_to_char = vec![chars.len(); text.len() + 1];
        let mut ci = 0usize;
        for (bi, ch) in text.char_indices() {
            for b in bi..bi + ch.len_utf8() {
                byte_to_char[b] = ci;
            }
            ci += 1;
        }
        let mut detected_spans: Vec<DetectedSpan> = Vec::new();

        if !ids.is_empty() {
            let logits = self.backend.forward_logits(&ids)?;
            let path = viterbi_decode(&logits, &self.label_space, &self.biases);
            let raw_spans = extract_spans(&path, &self.label_space);

            for raw in raw_spans {
                let label = &self.label_space.entity_names[raw.entity];
                let mut start = byte_to_char[offsets[raw.start_token].0];
                let mut end = byte_to_char[offsets[raw.end_token].1];
                while start < end && chars[start].is_whitespace() {
                    start += 1;
                }
                while end > start && chars[end - 1].is_whitespace() {
                    end -= 1;
                }
                if start >= end {
                    continue;
                }
                let span_text: String = chars[start..end].iter().collect();
                let placeholder = self
                    .placeholders
                    .get(label)
                    .cloned()
                    .unwrap_or_else(|| format!("[{}]", label.to_uppercase()));
                detected_spans.push(DetectedSpan {
                    label: label.clone(),
                    start,
                    end,
                    text: span_text,
                    placeholder,
                });
            }
        }

        let mut redacted = String::with_capacity(text.len());
        let mut cursor = 0usize;
        let mut summary: HashMap<String, u32> = HashMap::new();
        for span in &detected_spans {
            if span.start > cursor {
                redacted.extend(chars[cursor..span.start].iter());
            }
            redacted.push_str(&span.placeholder);
            *summary.entry(span.label.clone()).or_insert(0u32) += 1;
            cursor = span.end;
        }
        redacted.extend(chars[cursor.min(chars.len())..].iter());

        let latency_ms = (started.elapsed().as_secs_f64() * 1000.0 * 100.0).round() / 100.0;

        Ok(RedactionResult {
            schema_version: 1,
            text: text.to_string(),
            redacted_text: redacted,
            detected_spans,
            summary,
            latency_ms,
        })
    }
}

#[cfg(target_os = "macos")]
fn create_backend(model_dir: &Path, config: &ModelConfig) -> Result<Box<dyn InferenceBackend>> {
    Ok(Box::new(mlx_backend::MlxBackend::load(model_dir, config)?))
}

#[cfg(not(target_os = "macos"))]
fn create_backend(model_dir: &Path, config: &ModelConfig) -> Result<Box<dyn InferenceBackend>> {
    Ok(Box::new(onnx_backend::OnnxBackend::load(
        model_dir, config,
    )?))
}

struct EngineState {
    redactor: Option<Redactor>,
    info: Option<ModelInfo>,
}

pub struct InferenceEngine {
    state: Mutex<EngineState>,
}

impl Default for InferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl InferenceEngine {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(EngineState {
                redactor: None,
                info: None,
            }),
        }
    }

    pub fn load_model(&self, model_path: &Path) -> Result<ModelInfo> {
        let redactor = Redactor::load(model_path)?;
        let info = redactor.info.clone();
        let mut state = self.state.lock().unwrap();
        state.redactor = Some(redactor);
        state.info = Some(info.clone());
        Ok(info)
    }

    pub fn redact(&self, text: &str) -> Result<RedactionResult> {
        if text.trim().is_empty() {
            return Ok(RedactionResult {
                schema_version: 1,
                text: text.to_string(),
                redacted_text: text.to_string(),
                detected_spans: Vec::new(),
                summary: HashMap::new(),
                latency_ms: 0.0,
            });
        }
        let mut state = self.state.lock().unwrap();
        match state.redactor.as_mut() {
            Some(redactor) => redactor.redact(text),
            None => bail!("model not loaded"),
        }
    }

    pub fn is_model_loaded(&self) -> bool {
        self.state.lock().unwrap().redactor.is_some()
    }

    pub fn get_info(&self) -> Result<ModelInfo> {
        self.state
            .lock()
            .unwrap()
            .info
            .clone()
            .context("model not loaded")
    }
}
