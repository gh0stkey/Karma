use serde::Serialize;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

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

struct LogStore {
    logs: VecDeque<HttpLogEntry>,
    limit: usize,
}

pub struct ServerStateManager {
    running: AtomicBool,
    status: Mutex<ServerLifecycleStatus>,
    next_log_id: AtomicU64,
    log_store: Mutex<LogStore>,
    shutdown_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

impl ServerStateManager {
    pub fn new(log_limit: usize) -> Self {
        Self {
            running: AtomicBool::new(false),
            status: Mutex::new(ServerLifecycleStatus::Stopped),
            next_log_id: AtomicU64::new(1),
            log_store: Mutex::new(LogStore {
                logs: VecDeque::with_capacity(log_limit),
                limit: log_limit,
            }),
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

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn set_log_limit(&self, limit: usize) {
        let mut store = self.log_store.lock().unwrap();
        store.limit = limit;
        while store.logs.len() > limit {
            store.logs.pop_back();
        }
    }

    pub fn add_log(
        &self,
        timestamp: String,
        method: String,
        path: String,
        status: u16,
        latency_ms: f64,
        request_body: Option<String>,
        response_body: Option<String>,
    ) -> HttpLogEntry {
        let entry = HttpLogEntry {
            id: self.next_log_id.fetch_add(1, Ordering::SeqCst),
            timestamp,
            method,
            path,
            status,
            latency_ms,
            request_body,
            response_body,
        };
        let cloned = entry.clone();
        let mut store = self.log_store.lock().unwrap();
        store.logs.push_front(entry);
        while store.logs.len() > store.limit {
            store.logs.pop_back();
        }
        cloned
    }

    pub fn get_logs(&self, limit: usize) -> Vec<HttpLogEntry> {
        let store = self.log_store.lock().unwrap();
        store.logs.iter().take(limit).cloned().collect()
    }

    pub fn clear_logs(&self) {
        self.log_store.lock().unwrap().logs.clear();
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
