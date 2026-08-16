use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

pub const QUICK_LABEL: &str = "quick";

static QUICK_PINNED: AtomicBool = AtomicBool::new(false);

/// Frontmost app at the moment the panel is summoned, so hiding the panel can
/// hand focus back instead of letting macOS fall through to Karma's own main
/// window. Zero when the shortcut fired from Karma itself.
#[cfg(target_os = "macos")]
static PREV_APP_PID: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

#[cfg(target_os = "macos")]
fn remember_front_app() {
    use objc2_app_kit::NSWorkspace;
    let front = NSWorkspace::sharedWorkspace().frontmostApplication();
    let pid = front.map(|a| a.processIdentifier()).unwrap_or(0);
    let own = std::process::id() as i32;
    PREV_APP_PID.store(if pid == own { 0 } else { pid }, Ordering::Relaxed);
}

#[cfg(target_os = "macos")]
fn restore_prev_app() {
    use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
    let pid = PREV_APP_PID.swap(0, Ordering::Relaxed);
    if pid == 0 {
        return;
    }
    if let Some(app) = NSRunningApplication::runningApplicationWithProcessIdentifier(pid) {
        app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
    }
}

/// Toggle the quick-redact panel: hide when visible, otherwise create (once),
/// center, focus, and push the current clipboard text into it.
pub fn toggle_quick_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(QUICK_LABEL) {
        if win.is_visible().unwrap_or(false) {
            hide_quick(app);
            return;
        }
        present(app, &win);
        return;
    }

    match WebviewWindowBuilder::new(app, QUICK_LABEL, WebviewUrl::App("index.html".into()))
        .title("Karma Quick Redact")
        .inner_size(440.0, 520.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()
    {
        Ok(win) => {
            let handle = app.clone();
            win.on_window_event(move |event| match event {
                tauri::WindowEvent::Focused(false) => {
                    if !QUICK_PINNED.load(Ordering::Relaxed) {
                        hide_quick(&handle);
                    }
                }
                // The native close button hides instead of destroying, so the
                // webview state survives the next shortcut invocation.
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    hide_quick(&handle);
                }
                _ => {}
            });
            present(app, &win);
        }
        Err(e) => log::warn!("Failed to create quick redact window: {e}"),
    }
}

/// Hide the panel and hand focus back to the app that was frontmost when the
/// shortcut fired. Without this macOS falls through to Karma's main window as
/// the next key window. When the panel was summoned from Karma itself there
/// is no recorded app and focus settles naturally.
fn hide_quick(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(QUICK_LABEL) {
        let _ = w.hide();
    }
    #[cfg(target_os = "macos")]
    restore_prev_app();
}

/// Frontend close affordances (Esc) route through the same hide logic.
#[tauri::command]
pub fn hide_quick_window(app: AppHandle) {
    hide_quick(&app);
}

fn present(app: &AppHandle, win: &WebviewWindow) {
    #[cfg(target_os = "macos")]
    remember_front_app();
    let _ = win.center();
    let _ = win.show();
    let _ = win.set_focus();
    let clipboard = arboard::Clipboard::new()
        .ok()
        .and_then(|mut cb| cb.get_text().ok())
        .filter(|t| !t.trim().is_empty());
    let _ = app.emit_to(QUICK_LABEL, "quick-shown", clipboard);
}

/// Pin state set by the frontend pin button; a pinned panel stays open on blur.
#[tauri::command]
pub fn set_quick_pinned(pinned: bool) {
    QUICK_PINNED.store(pinned, Ordering::Relaxed);
}

/// Write text to the clipboard. The webview's navigator.clipboard is rejected
/// without a user gesture (auto-copy after background redact), so the quick
/// panel copies through the OS directly.
#[tauri::command]
pub fn copy_text(text: String) -> Result<(), String> {
    arboard::Clipboard::new()
        .and_then(|mut cb| cb.set_text(text))
        .map_err(|e| e.to_string())
}

/// Clipboard text for the first panel show, where the webview missed the
/// `quick-shown` event because it was still loading.
#[tauri::command]
pub fn quick_clipboard() -> Option<String> {
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut cb| cb.get_text().ok())
        .filter(|t| !t.trim().is_empty())
}
