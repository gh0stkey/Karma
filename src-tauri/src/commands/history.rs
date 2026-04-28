use serde::Serialize;
use std::sync::Arc;
use tauri::{AppHandle, Manager};

use crate::managers::history::{HistoryEntry, HistoryManager};

#[derive(Serialize)]
pub struct HistoryPage {
    pub entries: Vec<HistoryEntry>,
    pub has_more: bool,
}

#[tauri::command]
pub fn get_history_entries(
    app: AppHandle,
    cursor: Option<i64>,
    limit: u32,
) -> Result<HistoryPage, String> {
    let manager = app.state::<Arc<HistoryManager>>();
    let (entries, has_more) = manager
        .get_entries(cursor, limit)
        .map_err(|e| e.to_string())?;
    Ok(HistoryPage { entries, has_more })
}

#[tauri::command]
pub fn delete_history_entry(app: AppHandle, id: i64) -> Result<(), String> {
    let manager = app.state::<Arc<HistoryManager>>();
    manager.delete_entry(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn clear_all_history(app: AppHandle) -> Result<(), String> {
    let manager = app.state::<Arc<HistoryManager>>();
    manager.clear_all().map_err(|e| e.to_string())
}
