use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::managers::model::{ModelManager, ModelState};
use crate::managers::sidecar::{SidecarManager, SidecarModelInfo};

#[tauri::command]
pub fn get_model_state(app: AppHandle) -> Result<ModelState, String> {
    let manager = app.state::<Arc<ModelManager>>();
    Ok(manager.get_state())
}

#[tauri::command]
pub fn delete_model(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<Arc<ModelManager>>();
    manager.delete_model().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn open_model_folder(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<Arc<ModelManager>>();
    let dir = manager.model_dir();
    opener::open(dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_model_path(app: AppHandle, path: String) -> Result<(), String> {
    let manager = app.state::<Arc<ModelManager>>();
    manager.set_model_path(&path).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reload_model(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<Arc<ModelManager>>();

    let sidecar = match app.try_state::<Arc<SidecarManager>>() {
        Some(s) => s.inner().clone(),
        None => {
            log::error!("reload_model: sidecar not available");
            return Err("Sidecar not running".to_string());
        }
    };

    manager.set_loading();

    crate::load_model_via_sidecar(&sidecar, &manager).await;

    if manager.get_state().status == "error" {
        Err("Failed to load model".to_string())
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn get_loaded_model_info(app: AppHandle) -> Result<SidecarModelInfo, String> {
    let sidecar = app
        .try_state::<Arc<SidecarManager>>()
        .ok_or_else(|| "Sidecar not running".to_string())?;

    sidecar.get_info().map_err(|e| e.to_string())
}
