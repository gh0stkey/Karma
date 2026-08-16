use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::inference::{InferenceEngine, ModelInfo};
use crate::managers::model::{ModelManager, ModelState};

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
    let path = manager.model_dir();
    let dir = if path.is_file() {
        path.parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or(path)
    } else {
        path
    };
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
    let engine = app.state::<Arc<InferenceEngine>>();

    manager.set_loading();

    crate::load_model_with_engine(&engine, &manager).await;

    if manager.get_state().status == "error" {
        Err("Failed to load model".to_string())
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn get_loaded_model_info(app: AppHandle) -> Result<ModelInfo, String> {
    let engine = app.state::<Arc<InferenceEngine>>();
    engine.get_info().map_err(|e| e.to_string())
}
