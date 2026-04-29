use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

fn trigger_open_redact(handle: &AppHandle) {
    if let Some(win) = handle.get_webview_window("main") {
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
    let _ = handle.emit("show-redactor", ());
}

pub fn register_shortcut(app: &AppHandle, shortcut_str: &str) {
    if shortcut_str.is_empty() {
        return;
    }

    let handle = app.clone();
    if let Err(e) =
        app.global_shortcut()
            .on_shortcut(shortcut_str, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    trigger_open_redact(&handle);
                }
            })
    {
        log::warn!(
            "Failed to register global shortcut '{}': {}",
            shortcut_str,
            e
        );
    }
}

#[tauri::command]
pub async fn update_global_shortcut(
    app: AppHandle,
    old_shortcut: String,
    new_shortcut: String,
) -> Result<(), String> {
    if !old_shortcut.is_empty() {
        let _ = app.global_shortcut().unregister(old_shortcut.as_str());
    }

    if !new_shortcut.is_empty() {
        let handle = app.clone();
        app.global_shortcut()
            .on_shortcut(new_shortcut.as_str(), move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    trigger_open_redact(&handle);
                }
            })
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}
