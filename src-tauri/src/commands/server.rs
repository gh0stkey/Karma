use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::managers::server_state::{ServerLifecycleStatus, ServerStateManager};

#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
    pub running: bool,
    pub status: ServerLifecycleStatus,
    pub host: String,
    pub port: u16,
    pub model_loaded: bool,
}

#[tauri::command]
pub fn get_server_status(app: AppHandle) -> ServerStatus {
    let settings = crate::settings::get_settings(&app);

    let (running, status) = if let Some(server_state) = app.try_state::<Arc<ServerStateManager>>() {
        (server_state.is_running(), server_state.status())
    } else {
        (false, ServerLifecycleStatus::Stopped)
    };

    let model_loaded = app
        .try_state::<Arc<crate::inference::InferenceEngine>>()
        .map(|e| e.is_model_loaded())
        .unwrap_or(false);

    ServerStatus {
        running,
        status,
        host: settings.server_host,
        port: settings.server_port,
        model_loaded,
    }
}

#[tauri::command]
pub async fn start_server(app: AppHandle) -> Result<(), String> {
    let server_state = app.state::<Arc<ServerStateManager>>();
    if server_state.is_active() {
        return Err("Server is already active".to_string());
    }

    server_state.set_starting();
    emit_server_status(&app);

    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        crate::start_http_server(&handle).await;
    });

    Ok(())
}

#[tauri::command]
pub fn stop_server(app: AppHandle) -> Result<(), String> {
    let server_state = app.state::<Arc<ServerStateManager>>();
    if !server_state.is_running() {
        return Err("Server is not running".to_string());
    }

    if server_state.trigger_shutdown() {
        server_state.set_stopping();
        emit_server_status(&app);
        Ok(())
    } else {
        Err("Failed to send shutdown signal".to_string())
    }
}

pub(crate) fn emit_server_status(app: &AppHandle) {
    use tauri::Emitter;

    let status = get_server_status(app.clone());
    let _ = app.emit("server-status-changed", status);
}

/// Open the HTTP access log file in the system file manager, creating an empty
/// file first so the button works before any request has been logged.
#[tauri::command]
pub fn open_http_log(app: AppHandle) -> Result<(), String> {
    let path = ServerStateManager::ensure_log_file(&app).map_err(|e| e.to_string())?;
    opener::reveal(&path).map_err(|e| e.to_string())
}
