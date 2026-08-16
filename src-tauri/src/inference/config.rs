use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct RopeParameters {
    #[serde(default = "default_rope_theta")]
    pub rope_theta: f64,
    #[serde(default = "default_rope_factor")]
    pub factor: f64,
    #[serde(default = "default_beta_fast")]
    pub beta_fast: f64,
    #[serde(default = "default_beta_slow")]
    pub beta_slow: f64,
    #[serde(default = "default_original_max_pos")]
    pub original_max_position_embeddings: i64,
}

fn default_rope_theta() -> f64 {
    150000.0
}
fn default_rope_factor() -> f64 {
    32.0
}
fn default_beta_fast() -> f64 {
    32.0
}
fn default_beta_slow() -> f64 {
    1.0
}
fn default_original_max_pos() -> i64 {
    4096
}

impl Default for RopeParameters {
    fn default() -> Self {
        Self {
            rope_theta: default_rope_theta(),
            factor: default_rope_factor(),
            beta_fast: default_beta_fast(),
            beta_slow: default_beta_slow(),
            original_max_position_embeddings: default_original_max_pos(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfig {
    #[serde(default)]
    pub architectures: Vec<String>,
    #[serde(default)]
    pub hidden_size: usize,
    #[serde(default)]
    pub intermediate_size: usize,
    #[serde(default)]
    pub num_hidden_layers: usize,
    #[serde(default)]
    pub num_attention_heads: usize,
    #[serde(default)]
    pub num_key_value_heads: usize,
    #[serde(default)]
    pub head_dim: usize,
    #[serde(default)]
    pub vocab_size: usize,
    #[serde(default)]
    pub num_local_experts: usize,
    #[serde(default)]
    pub num_experts_per_tok: usize,
    #[serde(default = "default_sliding_window")]
    pub sliding_window: usize,
    #[serde(default = "default_rms_eps")]
    pub rms_norm_eps: f64,
    #[serde(default)]
    pub max_position_embeddings: usize,
    #[serde(default)]
    pub rope_parameters: RopeParameters,
    #[serde(default)]
    pub id2label: HashMap<String, String>,
}

fn default_sliding_window() -> usize {
    128
}
fn default_rms_eps() -> f64 {
    1e-5
}

impl ModelConfig {
    pub fn load(model_dir: &Path) -> Result<Self> {
        let path = model_dir.join("config.json");
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let cfg: Self = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(cfg)
    }

    pub fn num_labels(&self) -> usize {
        self.id2label.len()
    }

    pub fn architecture(&self) -> &str {
        self.architectures
            .first()
            .map(String::as_str)
            .unwrap_or("unknown")
    }

    pub fn model_name(model_dir: &Path) -> String {
        model_dir
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ViterbiBiases {
    pub background_stay: f32,
    pub background_to_start: f32,
    pub end_to_background: f32,
    pub end_to_start: f32,
    pub inside_to_continue: f32,
    pub inside_to_end: f32,
}

#[derive(Debug, Deserialize)]
struct ViterbiFile {
    #[serde(default)]
    operating_points: HashMap<String, ViterbiOperatingPoint>,
}

#[derive(Debug, Deserialize)]
struct ViterbiOperatingPoint {
    #[serde(default)]
    transition_bias: ViterbiBiasesRaw,
}

#[derive(Debug, Default, Deserialize)]
struct ViterbiBiasesRaw {
    #[serde(default)]
    background_stay: f32,
    #[serde(default)]
    background_to_start: f32,
    #[serde(default)]
    end_to_background: f32,
    #[serde(default)]
    end_to_start: f32,
    #[serde(default)]
    inside_to_continue: f32,
    #[serde(default)]
    inside_to_end: f32,
}

pub fn load_viterbi_biases(model_dir: &Path) -> ViterbiBiases {
    let path = model_dir.join("viterbi_calibration.json");
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return ViterbiBiases::default();
    };
    let Ok(file) = serde_json::from_str::<ViterbiFile>(&raw) else {
        return ViterbiBiases::default();
    };
    let Some(point) = file.operating_points.get("default") else {
        return ViterbiBiases::default();
    };
    let b = &point.transition_bias;
    ViterbiBiases {
        background_stay: b.background_stay,
        background_to_start: b.background_to_start,
        end_to_background: b.end_to_background,
        end_to_start: b.end_to_start,
        inside_to_continue: b.inside_to_continue,
        inside_to_end: b.inside_to_end,
    }
}

pub fn placeholder_map() -> HashMap<String, String> {
    [
        ("account_number", "[ACCOUNT_NUMBER]"),
        ("private_address", "[ADDRESS]"),
        ("private_date", "[DATE]"),
        ("private_email", "[EMAIL]"),
        ("private_person", "[PERSON]"),
        ("private_phone", "[PHONE]"),
        ("private_url", "[URL]"),
        ("secret", "[SECRET]"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

pub fn resolve_model_dir(model_path: &Path) -> Result<PathBuf> {
    if model_path.is_dir() {
        return Ok(model_path.to_path_buf());
    }
    if model_path.is_file() {
        let parent = model_path
            .parent()
            .context("model file has no parent directory")?;
        return Ok(parent.to_path_buf());
    }
    bail!("model path does not exist: {}", model_path.display())
}
