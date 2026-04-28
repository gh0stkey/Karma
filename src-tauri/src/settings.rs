use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

const STORE_PATH: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub server_enabled: bool,
    pub server_host: String,
    pub server_port: u16,
    pub server_auto_start: bool,
    pub server_log_limit: u32,
    pub model_path: String,
    pub auto_copy_result: bool,
    pub save_history: bool,
    pub history_limit: u32,
    pub app_language: String,
    pub global_shortcut: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            server_enabled: false,
            server_host: "127.0.0.1".to_string(),
            server_port: 8000,
            server_auto_start: false,
            server_log_limit: 100,
            model_path: String::new(),
            auto_copy_result: false,
            save_history: true,
            history_limit: 1000,
            app_language: "en".to_string(),
            global_shortcut: "command+shift+k".to_string(),
        }
    }
}

macro_rules! read_setting {
    ($store:expr, $key:literal, bool, $default:expr) => {
        $store.get($key).and_then(|v| v.as_bool()).unwrap_or($default)
    };
    ($store:expr, $key:literal, str, $default:expr) => {
        $store.get($key).and_then(|v| v.as_str().map(String::from)).unwrap_or($default)
    };
    ($store:expr, $key:literal, u16, $default:expr) => {
        $store.get($key).and_then(|v| v.as_u64().map(|n| n as u16)).unwrap_or($default)
    };
    ($store:expr, $key:literal, u32, $default:expr) => {
        $store.get($key).and_then(|v| v.as_u64().map(|n| n as u32)).unwrap_or($default)
    };
}

pub fn get_settings(app: &AppHandle) -> AppSettings {
    let store = match app.store(STORE_PATH) {
        Ok(s) => s,
        Err(_) => return AppSettings::default(),
    };

    let defaults = AppSettings::default();

    AppSettings {
        server_enabled: read_setting!(store, "server_enabled", bool, defaults.server_enabled),
        server_host: read_setting!(store, "server_host", str, defaults.server_host),
        server_port: read_setting!(store, "server_port", u16, defaults.server_port),
        server_auto_start: read_setting!(store, "server_auto_start", bool, defaults.server_auto_start),
        server_log_limit: read_setting!(store, "server_log_limit", u32, defaults.server_log_limit),
        model_path: read_setting!(store, "model_path", str, defaults.model_path),
        auto_copy_result: read_setting!(store, "auto_copy_result", bool, defaults.auto_copy_result),
        save_history: read_setting!(store, "save_history", bool, defaults.save_history),
        history_limit: read_setting!(store, "history_limit", u32, defaults.history_limit),
        app_language: read_setting!(store, "app_language", str, defaults.app_language),
        global_shortcut: read_setting!(store, "global_shortcut", str, defaults.global_shortcut),
    }
}

pub fn update_setting(app: &AppHandle, key: &str, value: serde_json::Value) -> anyhow::Result<()> {
    let store = app.store(STORE_PATH)?;
    store.set(key.to_string(), value);
    Ok(())
}
