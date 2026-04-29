use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::managers::server_state::{HttpLogEntry, ServerLifecycleStatus, ServerStateManager};

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
        .try_state::<Arc<crate::managers::sidecar::SidecarManager>>()
        .map(|s| s.is_healthy())
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

#[tauri::command]
pub fn get_http_logs(app: AppHandle, limit: Option<usize>) -> Vec<HttpLogEntry> {
    let limit = limit.unwrap_or(100);
    if let Some(server_state) = app.try_state::<Arc<ServerStateManager>>() {
        server_state.get_logs(limit)
    } else {
        Vec::new()
    }
}

#[tauri::command]
pub fn clear_http_logs(app: AppHandle) {
    if let Some(server_state) = app.try_state::<Arc<ServerStateManager>>() {
        server_state.clear_logs();
    }
}

pub(crate) fn emit_server_status(app: &AppHandle) {
    use tauri::Emitter;

    let status = get_server_status(app.clone());
    let _ = app.emit("server-status-changed", status);
}
