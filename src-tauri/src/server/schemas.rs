use serde::{Deserialize, Serialize};

pub use crate::inference::RedactionResult;

#[derive(Debug, Deserialize)]
pub struct RedactRequest {
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub model_loaded: bool,
}
