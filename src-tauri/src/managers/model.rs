use anyhow::{Context, Result};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
pub struct ModelState {
    pub status: String,
    pub model_path: String,
}

impl Default for ModelState {
    fn default() -> Self {
        Self {
            status: "not_found".to_string(),
            model_path: String::new(),
        }
    }
}

pub struct ModelManager {
    app: AppHandle,
    state: Arc<Mutex<ModelState>>,
    model_dir: Mutex<PathBuf>,
}

impl ModelManager {
    pub fn new(app: &AppHandle) -> Result<Self> {
        let settings = crate::settings::get_settings(app);
        let model_dir = PathBuf::from(&settings.model_path);

        let status = if Self::check_model_files(&model_dir) {
            "loading"
        } else {
            "not_found"
        };

        let state = ModelState {
            status: status.to_string(),
            model_path: settings.model_path.clone(),
        };

        Ok(Self {
            app: app.clone(),
            state: Arc::new(Mutex::new(state)),
            model_dir: Mutex::new(model_dir),
        })
    }

    fn check_model_files(dir: &PathBuf) -> bool {
        if !dir.join("config.json").exists() || !dir.join("tokenizer.json").exists() {
            return false;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry
                    .path()
                    .extension()
                    .map(|e| e == "safetensors")
                    .unwrap_or(false)
                {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_state(&self) -> ModelState {
        self.state.lock().unwrap().clone()
    }

    fn emit_state(&self) {
        let state = self.get_state();
        let _ = self.app.emit("model-state-changed", &state);
    }

    pub fn set_model_path(&self, path: &str) -> Result<()> {
        let new_dir = PathBuf::from(path);

        let status = if Self::check_model_files(&new_dir) {
            "ready"
        } else {
            "not_found"
        };

        {
            let mut s = self.state.lock().unwrap();
            s.model_path = path.to_string();
            s.status = status.to_string();
        }

        *self.model_dir.lock().unwrap() = new_dir;

        crate::settings::update_setting(
            &self.app,
            "model_path",
            serde_json::Value::String(path.to_string()),
        )?;

        self.emit_state();
        Ok(())
    }

    pub fn delete_model(&self) -> Result<()> {
        let model_dir = self.model_dir.lock().unwrap().clone();

        if model_dir.exists() {
            std::fs::remove_dir_all(&model_dir)
                .with_context(|| format!("Failed to delete {}", model_dir.display()))?;
            log::info!("Deleted model directory: {}", model_dir.display());
        }

        {
            let mut s = self.state.lock().unwrap();
            s.status = "not_found".to_string();
        }
        self.emit_state();

        Ok(())
    }

    pub fn is_available(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.status == "ready" || state.status == "loaded" || state.status == "loading"
    }

    pub fn model_dir(&self) -> PathBuf {
        self.model_dir.lock().unwrap().clone()
    }

    pub fn set_loaded(&self) {
        let mut s = self.state.lock().unwrap();
        s.status = "loaded".to_string();
        drop(s);
        self.emit_state();
    }

    pub fn set_loading(&self) {
        let mut s = self.state.lock().unwrap();
        s.status = "loading".to_string();
        drop(s);
        self.emit_state();
    }

    pub fn set_error(&self) {
        let mut s = self.state.lock().unwrap();
        s.status = "error".to_string();
        drop(s);
        self.emit_state();
    }
}
