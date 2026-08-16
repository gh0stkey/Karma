use crate::settings::{self, AppSettings};
use tauri::AppHandle;

#[tauri::command]
pub fn get_app_settings(app: AppHandle) -> AppSettings {
    settings::get_settings(&app)
}

#[tauri::command]
pub fn update_setting(app: AppHandle, key: String, value: String) -> Result<(), String> {
    let parsed: serde_json::Value = serde_json::from_str(&value).map_err(|e| e.to_string())?;
    settings::update_setting(&app, &key, parsed).map_err(|e| e.to_string())?;
    Ok(())
}
