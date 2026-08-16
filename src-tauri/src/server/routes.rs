use axum::{
    Router,
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{Json, Response},
    routing::{get, post},
};
use std::sync::Arc;
use tauri::Manager;
use tower_http::cors::CorsLayer;

use super::schemas::{HealthResponse, RedactRequest, RedactionResult};
use crate::inference::InferenceEngine;
use crate::managers::server_state::{HttpLogEntry, ServerStateManager, append_http_log};

pub struct ApiState {
    pub server_state: Arc<ServerStateManager>,
    pub app_handle: tauri::AppHandle,
}

pub fn create_router(state: Arc<ApiState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/redact", post(redact))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            http_logging_middleware,
        ))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn http_logging_middleware(
    State(state): State<Arc<ApiState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let start = std::time::Instant::now();

    let (parts, body) = req.into_parts();
    let req_bytes = axum::body::to_bytes(body, 1024 * 64)
        .await
        .unwrap_or_default();
    let request_body = if req_bytes.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&req_bytes).to_string())
    };
    let req = Request::from_parts(parts, Body::from(req_bytes));

    let response = next.run(req).await;
    let latency_ms = start.elapsed().as_secs_f64() * 1000.0;
    let status = response.status().as_u16();

    let (resp_parts, resp_body) = response.into_parts();
    let resp_bytes = axum::body::to_bytes(resp_body, 1024 * 64)
        .await
        .unwrap_or_default();
    let response_body = if resp_bytes.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&resp_bytes).to_string())
    };

    let entry = HttpLogEntry {
        id: state.server_state.next_log_id(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        method,
        path,
        status,
        latency_ms: (latency_ms * 100.0).round() / 100.0,
        request_body,
        response_body,
    };

    if let Err(e) = append_http_log(&state.app_handle, &entry) {
        log::warn!("Failed to persist HTTP log entry: {}", e);
    }

    Response::from_parts(resp_parts, Body::from(resp_bytes))
}

async fn health(State(state): State<Arc<ApiState>>) -> Json<HealthResponse> {
    let model_loaded = state
        .app_handle
        .try_state::<Arc<InferenceEngine>>()
        .map(|e| e.is_model_loaded())
        .unwrap_or(false);
    Json(HealthResponse {
        status: "ok".to_string(),
        model_loaded,
    })
}

async fn redact(
    State(state): State<Arc<ApiState>>,
    Json(req): Json<RedactRequest>,
) -> Result<Json<RedactionResult>, (StatusCode, String)> {
    let engine = state
        .app_handle
        .try_state::<Arc<InferenceEngine>>()
        .ok_or((
            StatusCode::SERVICE_UNAVAILABLE,
            "Inference engine not available".to_string(),
        ))?;

    let engine = engine.inner().clone();
    let text = req.text;

    let result = tokio::task::spawn_blocking(move || engine.redact(&text))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}
