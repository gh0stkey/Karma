use std::path::Path;

use anyhow::{Context, Result, bail};
use ort::session::{Session, builder::GraphOptimizationLevel};
use ort::value::Tensor;

use super::InferenceBackend;
use super::config::ModelConfig;

pub struct OnnxBackend {
    session: Session,
    num_labels: usize,
}

impl OnnxBackend {
    pub fn load(model_dir: &Path, _config: &ModelConfig) -> Result<Self> {
        let mut candidates: Vec<_> = std::fs::read_dir(model_dir)
            .with_context(|| format!("failed to read {}", model_dir.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().map(|e| e == "onnx").unwrap_or(false))
            .collect();
        candidates.sort();
        candidates.sort_by_key(|p| {
            if p.file_name().map(|n| n == "model.onnx").unwrap_or(false) {
                0
            } else {
                1
            }
        });
        let Some(model_file) = candidates.into_iter().next() else {
            bail!("no .onnx model file in {}", model_dir.display());
        };

        let mut builder = Session::builder()
            .map_err(|e| anyhow::anyhow!("failed to create ONNX session builder: {e}"))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| anyhow::anyhow!("failed to set optimization level: {e}"))?;

        #[cfg(feature = "onnx-cuda")]
        {
            builder = builder
                .with_execution_providers([
                    ort::execution_providers::CUDAExecutionProvider::default().build(),
                ])
                .map_err(|e| anyhow::anyhow!("failed to register CUDA provider: {e}"))?;
        }
        #[cfg(feature = "onnx-directml")]
        {
            builder = builder
                .with_execution_providers([
                    ort::execution_providers::DirectMLExecutionProvider::default().build(),
                ])
                .map_err(|e| anyhow::anyhow!("failed to register DirectML provider: {e}"))?;
        }

        let session = builder
            .commit_from_file(&model_file)
            .map_err(|e| anyhow::anyhow!("failed to load {}: {e}", model_file.display()))?;

        Ok(Self {
            session,
            num_labels: _config.num_labels(),
        })
    }
}

impl InferenceBackend for OnnxBackend {
    fn forward_logits(&mut self, ids: &[u32]) -> Result<Vec<Vec<f32>>> {
        let n = ids.len();
        let shape = [1usize, n];
        let input_ids: Vec<i64> = ids.iter().map(|&i| i as i64).collect();
        let attention_mask: Vec<i64> = vec![1i64; n];

        let input_ids = Tensor::from_array((shape, input_ids))
            .map_err(|e| anyhow::anyhow!("failed to create input tensor: {e}"))?;
        let attention_mask = Tensor::from_array((shape, attention_mask))
            .map_err(|e| anyhow::anyhow!("failed to create mask tensor: {e}"))?;

        let outputs = self
            .session
            .run(ort::inputs!["input_ids" => input_ids, "attention_mask" => attention_mask])
            .map_err(|e| anyhow::anyhow!("ONNX inference failed: {e}"))?;

        let logits = outputs["logits"]
            .try_extract_array::<f32>()
            .map_err(|e| anyhow::anyhow!("failed to extract logits: {e}"))?;

        let flat: Vec<f32> = logits.iter().copied().collect();
        Ok(flat.chunks(self.num_labels).map(|c| c.to_vec()).collect())
    }
}
