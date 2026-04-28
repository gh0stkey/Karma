use serde::Serialize;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Instant;

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
    started_at: Mutex<Option<Instant>>,
    request_count: AtomicU64,
    next_log_id: AtomicU64,
    log_store: Mutex<LogStore>,
    shutdown_tx: Mutex<Option<tokio::sync::oneshot::Sender<()>>>,
}

impl ServerStateManager {
    pub fn new(log_limit: usize) -> Self {
        Self {
            running: AtomicBool::new(false),
            started_at: Mutex::new(None),
            request_count: AtomicU64::new(0),
            next_log_id: AtomicU64::new(1),
            log_store: Mutex::new(LogStore {
                logs: VecDeque::with_capacity(log_limit),
                limit: log_limit,
            }),
            shutdown_tx: Mutex::new(None),
        }
    }

    pub fn set_running(&self, running: bool) {
        self.running.store(running, Ordering::SeqCst);
        if running {
            *self.started_at.lock().unwrap() = Some(Instant::now());
        } else {
            *self.started_at.lock().unwrap() = None;
        }
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn uptime_seconds(&self) -> Option<u64> {
        self.started_at
            .lock()
            .unwrap()
            .map(|started| started.elapsed().as_secs())
    }

    pub fn request_count(&self) -> u64 {
        self.request_count.load(Ordering::SeqCst)
    }

    pub fn increment_request_count(&self) {
        self.request_count.fetch_add(1, Ordering::SeqCst);
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
