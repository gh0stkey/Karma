use crate::managers::server_state::ServerStateManager;
use crate::settings::{self, AppSettings};
use std::sync::Arc;
use tauri::{AppHandle, Manager};

#[tauri::command]
pub fn get_app_settings(app: AppHandle) -> AppSettings {
    settings::get_settings(&app)
}

#[tauri::command]
pub fn update_setting(app: AppHandle, key: String, value: String) -> Result<(), String> {
    let parsed: serde_json::Value = serde_json::from_str(&value).map_err(|e| e.to_string())?;
    settings::update_setting(&app, &key, parsed.clone()).map_err(|e| e.to_string())?;

    if key == "server_log_limit" {
        if let (Some(server_state), Some(limit)) =
            (app.try_state::<Arc<ServerStateManager>>(), parsed.as_u64())
        {
            server_state.set_log_limit(limit as usize);
        }
    }

    Ok(())
}
