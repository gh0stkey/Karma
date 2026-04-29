mod commands;
mod managers;
mod server;
mod settings;
mod shortcut;

use managers::history::HistoryManager;
use managers::model::ModelManager;
use managers::server_state::ServerStateManager;
use managers::sidecar::SidecarManager;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_log::{Builder as LogBuilder, Target, TargetKind};

fn resolve_sidecar_path(_app: &tauri::AppHandle) -> std::path::PathBuf {
    let exe_suffix = std::env::consts::EXE_SUFFIX;

    #[cfg(debug_assertions)]
    {
        let triple = env!("TAURI_ENV_TARGET_TRIPLE");
        let dev_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("binaries")
            .join(format!("opf-mlx-{triple}{exe_suffix}"));
        if dev_path.exists() {
            log::info!("Sidecar (dev): {}", dev_path.display());
            return dev_path;
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        let exe = exe.canonicalize().unwrap_or(exe);
        if let Some(exe_dir) = exe.parent() {
            let prod_path = exe_dir.join(format!("opf-mlx{exe_suffix}"));
            if prod_path.exists() {
                log::info!("Sidecar (prod): {}", prod_path.display());
                return prod_path;
            }
        }
    }

    let fallback = std::path::PathBuf::from(format!("opf-mlx{exe_suffix}"));
    log::error!("Sidecar binary not found!");
    fallback
}

pub(crate) async fn load_model_via_sidecar(
    sidecar: &Arc<SidecarManager>,
    manager: &Arc<ModelManager>,
) {
    let model_dir = manager.model_dir();
    let model_path = model_dir.to_string_lossy().to_string();
    let sidecar_clone = sidecar.clone();

    let load_result = tokio::task::spawn_blocking(move || sidecar_clone.load(&model_path)).await;

    match load_result {
        Ok(Ok(info)) => {
            manager.set_loaded();
            log::info!("Model loaded via sidecar: {}", info.name);
        }
        Ok(Err(e)) => {
            manager.set_error();
            log::warn!("Failed to load model via sidecar: {}", e);
        }
        Err(e) => {
            manager.set_error();
            log::error!("Model loading task panicked: {}", e);
        }
    }
}

fn initialize_managers(app: &tauri::AppHandle) {
    let settings = settings::get_settings(app);

    let server_state = Arc::new(ServerStateManager::new(settings.server_log_limit as usize));
    app.manage(server_state.clone());

    let model_manager =
        Arc::new(ModelManager::new(app).expect("Failed to initialize model manager"));

    let history_manager =
        Arc::new(HistoryManager::new(app).expect("Failed to initialize history manager"));

    app.manage(model_manager.clone());
    app.manage(history_manager.clone());

    let handle = app.clone();
    let manager = model_manager.clone();
    let sidecar_path = resolve_sidecar_path(app);

    tauri::async_runtime::spawn(async move {
        let sidecar_path_clone = sidecar_path.clone();
        let spawn_result =
            tokio::task::spawn_blocking(move || SidecarManager::spawn(&sidecar_path_clone)).await;

        match spawn_result {
            Ok(Ok(sidecar)) => {
                let sidecar = Arc::new(sidecar);
                handle.manage(sidecar.clone());
                log::info!("MLX sidecar spawned successfully");

                if manager.is_available() {
                    load_model_via_sidecar(&sidecar, &manager).await;
                }
            }
            Ok(Err(e)) => {
                log::error!("Failed to spawn sidecar: {}", e);
            }
            Err(e) => {
                log::error!("Sidecar spawn task panicked: {}", e);
            }
        }
    });
}

pub(crate) async fn start_http_server(app: &tauri::AppHandle) {
    let settings = settings::get_settings(app);
    let server_state = app.state::<Arc<ServerStateManager>>();

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
            return;
        }
    };

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    server_state.set_shutdown_handle(shutdown_tx);
    server_state.set_running(true);

    {
        use tauri::Emitter;
        let _ = app.emit("server-status-changed", ());
    }

    let serve_result = axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        })
        .await;

    if let Err(e) = serve_result {
        log::error!("HTTP server error: {}", e);
    }

    server_state.set_running(false);
    log::info!("HTTP server stopped");

    {
        use tauri::Emitter;
        let _ = app.emit("server-status-changed", ());
    }
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
            commands::server::start_server,
            commands::server::stop_server,
            commands::server::get_http_logs,
            commands::server::clear_http_logs,
            commands::history::get_history_entries,
            commands::history::delete_history_entry,
            commands::history::clear_all_history,
            shortcut::update_global_shortcut,
        ])
        .setup(|app| {
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
