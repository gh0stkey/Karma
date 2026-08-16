use anyhow::{Context, Result};
use serde::Serialize;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tauri::{AppHandle, Manager};

const HTTP_LOG_FILE_NAME: &str = "http.log";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ServerLifecycleStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct HttpLogEntry {
    pub id: u64,
    pub timestamp: String,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub latency_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_body: Option<String>,
}

pub struct ServerStateManager {
    running: AtomicBool,
    status: Mutex<ServerLifecycleStatus>,
    next_log_id: AtomicU64,
    shutdown_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

pub fn http_log_path(app: &AppHandle) -> Result<std::path::PathBuf> {
    let log_dir = app
        .path()
        .app_log_dir()
        .context("Failed to get app log dir")?;
    fs::create_dir_all(&log_dir)?;
    Ok(log_dir.join(HTTP_LOG_FILE_NAME))
}

pub fn append_http_log(app: &AppHandle, entry: &HttpLogEntry) -> Result<()> {
    let log_path = http_log_path(app)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;
    serde_json::to_writer(&mut file, entry)?;
    writeln!(file)?;
    Ok(())
}

impl ServerStateManager {
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            status: Mutex::new(ServerLifecycleStatus::Stopped),
            next_log_id: AtomicU64::new(1),
            shutdown_tx: Mutex::new(None),
        }
    }

    pub fn status(&self) -> ServerLifecycleStatus {
        *self.status.lock().unwrap()
    }

    pub fn set_starting(&self) {
        self.running.store(false, Ordering::SeqCst);
        *self.status.lock().unwrap() = ServerLifecycleStatus::Starting;
    }

    pub fn set_running(&self) {
        self.running.store(true, Ordering::SeqCst);
        *self.status.lock().unwrap() = ServerLifecycleStatus::Running;
    }

    pub fn set_stopping(&self) {
        *self.status.lock().unwrap() = ServerLifecycleStatus::Stopping;
    }

    pub fn set_stopped(&self) {
        self.running.store(false, Ordering::SeqCst);
        *self.status.lock().unwrap() = ServerLifecycleStatus::Stopped;
    }

    pub fn set_error(&self) {
        self.running.store(false, Ordering::SeqCst);
        *self.status.lock().unwrap() = ServerLifecycleStatus::Error;
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status(),
            ServerLifecycleStatus::Starting
                | ServerLifecycleStatus::Running
                | ServerLifecycleStatus::Stopping
        )
    }

    /// Ensure the HTTP log file exists (creating parent dirs if needed) so it
    /// can be revealed in the file manager before any request was logged.
    pub fn ensure_log_file(app: &AppHandle) -> anyhow::Result<std::path::PathBuf> {
        let path = http_log_path(app)?;
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(path)
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn next_log_id(&self) -> u64 {
        self.next_log_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn set_shutdown_handle(&self, tx: tokio::sync::oneshot::Sender<()>) {
        *self.shutdown_tx.lock().unwrap() = Some(tx);
    }

    pub fn trigger_shutdown(&self) -> bool {
        if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
            let _ = tx.send(());
            true
        } else {
            false
        }
    }
}
