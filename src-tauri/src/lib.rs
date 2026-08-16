mod commands;
pub mod inference;
mod managers;
mod quick_window;
mod server;
mod settings;
mod shortcut;

use inference::InferenceEngine;
use managers::history::HistoryManager;
use managers::model::ModelManager;
use managers::server_state::ServerStateManager;
use std::sync::Arc;
use tauri::{
    Manager, WindowEvent,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
};
use tauri_plugin_log::{Builder as LogBuilder, Target, TargetKind};

const TRAY_OPEN_ID: &str = "open";
const TRAY_QUIT_ID: &str = "quit";

pub(crate) async fn load_model_with_engine(
    engine: &Arc<InferenceEngine>,
    manager: &Arc<ModelManager>,
) {
    let model_dir = manager.model_dir();
    let engine_clone = engine.clone();

    let load_result =
        tokio::task::spawn_blocking(move || engine_clone.load_model(&model_dir)).await;

    match load_result {
        Ok(Ok(info)) => {
            manager.set_loaded();
            log::info!("Model loaded: {}", info.name);
        }
        Ok(Err(e)) => {
            manager.set_error();
            log::warn!("Failed to load model: {}", e);
        }
        Err(e) => {
            manager.set_error();
            log::error!("Model loading task panicked: {}", e);
        }
    }
}

fn initialize_managers(app: &tauri::AppHandle) {
    let server_state = Arc::new(ServerStateManager::new());
    app.manage(server_state.clone());

    let model_manager =
        Arc::new(ModelManager::new(app).expect("Failed to initialize model manager"));

    let history_manager =
        Arc::new(HistoryManager::new(app).expect("Failed to initialize history manager"));

    app.manage(model_manager.clone());
    app.manage(history_manager.clone());

    let engine = Arc::new(InferenceEngine::new());
    app.manage(engine.clone());

    let manager = model_manager.clone();

    tauri::async_runtime::spawn(async move {
        if manager.is_available() {
            load_model_with_engine(&engine, &manager).await;
        }
    });
}

pub(crate) fn show_main_window(app: &tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    let _ = app.set_dock_visibility(true);

    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

fn initialize_tray(app: &tauri::App) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, TRAY_OPEN_ID, "Open Karma", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, TRAY_QUIT_ID, "Quit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&open, &separator, &quit])?;
    let icon = app.default_window_icon().cloned();

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Karma")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            if event.id() == TRAY_OPEN_ID {
                show_main_window(app);
            } else if event.id() == TRAY_QUIT_ID {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => show_main_window(tray.app_handle()),
            _ => {}
        });

    if let Some(icon) = icon {
        tray = tray.icon(icon);
    }

    tray.build(app)?;
    Ok(())
}

pub(crate) async fn start_http_server(app: &tauri::AppHandle) {
    let settings = settings::get_settings(app);
    let server_state = app.state::<Arc<ServerStateManager>>();

    server_state.set_starting();
    commands::server::emit_server_status(app);

    let api_state = Arc::new(server::routes::ApiState {
        server_state: server_state.inner().clone(),
        app_handle: app.clone(),
    });

    let router = server::routes::create_router(api_state);
    let addr = format!("{}:{}", settings.server_host, settings.server_port);

    log::info!("Starting HTTP API server on {}", addr);

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            log::error!("Failed to bind HTTP server to {}: {}", addr, e);
            server_state.set_error();
            commands::server::emit_server_status(app);
            return;
        }
    };

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    server_state.set_shutdown_handle(shutdown_tx);
    server_state.set_running();
    commands::server::emit_server_status(app);

    let serve_result = axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        })
        .await;

    if let Err(e) = serve_result {
        log::error!("HTTP server error: {}", e);
        server_state.set_error();
        commands::server::emit_server_status(app);
        return;
    }

    server_state.set_stopped();
    log::info!("HTTP server stopped");
    commands::server::emit_server_status(app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            LogBuilder::new()
                .level(log::LevelFilter::Info)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("karma".into()),
                    }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = window.hide();

                    #[cfg(target_os = "macos")]
                    let _ = window.app_handle().set_dock_visibility(false);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::settings::get_app_settings,
            commands::settings::update_setting,
            commands::redact::redact_text,
            commands::model::get_model_state,
            commands::model::delete_model,
            commands::model::open_model_folder,
            commands::model::set_model_path,
            commands::model::reload_model,
            commands::model::get_loaded_model_info,
            commands::server::get_server_status,
            commands::server::open_http_log,
            quick_window::copy_text,
            quick_window::hide_quick_window,
            quick_window::quick_clipboard,
            quick_window::set_quick_pinned,
            commands::server::start_server,
            commands::server::stop_server,
            commands::history::get_history_entries,
            commands::history::delete_history_entry,
            commands::history::clear_all_history,
            shortcut::update_global_shortcut,
        ])
        .setup(|app| {
            initialize_tray(app)?;

            let handle = app.handle().clone();
            initialize_managers(&handle);

            let settings = settings::get_settings(&handle);
            shortcut::register_shortcut(&handle, &settings.global_shortcut);

            if settings.server_enabled && settings.server_auto_start {
                let handle_for_server = handle.clone();
                tauri::async_runtime::spawn(async move {
                    start_http_server(&handle_for_server).await;
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Karma");
}
