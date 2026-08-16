use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};

use crate::inference::{InferenceEngine, RedactionResult};
use crate::managers::history::HistoryManager;

#[tauri::command]
pub async fn redact_text(app: AppHandle, text: String) -> Result<RedactionResult, String> {
    let engine = app
        .try_state::<Arc<InferenceEngine>>()
        .ok_or("Model not loaded")?;

    let engine = engine.inner().clone();
    let text_clone = text.clone();

    let result = tokio::task::spawn_blocking(move || engine.redact(&text_clone))
        .await
        .map_err(|e| format!("Task join error: {}", e))?
        .map_err(|e| e.to_string())?;

    let settings = crate::settings::get_settings(&app);
    if settings.save_history {
        if let Some(history) = app.try_state::<Arc<HistoryManager>>() {
            match history.add_entry(&result, settings.history_limit) {
                Ok(entry) => {
                    let _ = app.emit("history-entry-added", &entry);
                }
                Err(e) => {
                    log::warn!("Failed to save history entry: {}", e);
                }
            }
        }
    }

    Ok(result)
}
