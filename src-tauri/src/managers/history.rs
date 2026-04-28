use anyhow::Result;
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::managers::sidecar::RedactionResult;

#[derive(Debug, Clone, Serialize)]
pub struct HistoryEntry {
    pub id: i64,
    pub timestamp: String,
    pub input_text: String,
    pub redacted_text: String,
    pub detected_spans: Vec<crate::managers::sidecar::DetectedSpan>,
    pub summary: std::collections::HashMap<String, u32>,
    pub latency_ms: f64,
}

pub struct HistoryManager {
    db: Mutex<Connection>,
}

impl HistoryManager {
    pub fn new(app: &AppHandle) -> Result<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .expect("Failed to get app data dir");
        std::fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("history.db");
        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL DEFAULT (datetime('now')),
                input_text TEXT NOT NULL,
                redacted_text TEXT NOT NULL,
                detected_spans TEXT NOT NULL DEFAULT '[]',
                summary TEXT NOT NULL DEFAULT '{}',
                latency_ms REAL NOT NULL DEFAULT 0
            );
            CREATE INDEX IF NOT EXISTS idx_history_timestamp ON history(timestamp DESC);",
        )?;

        Ok(Self {
            db: Mutex::new(conn),
        })
    }

    pub fn add_entry(&self, result: &RedactionResult, history_limit: u32) -> Result<HistoryEntry> {
        let db = self.db.lock().unwrap();
        let spans_json = serde_json::to_string(&result.detected_spans)?;
        let summary_json = serde_json::to_string(&result.summary)?;

        db.execute(
            "INSERT INTO history (input_text, redacted_text, detected_spans, summary, latency_ms) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                result.text,
                result.redacted_text,
                spans_json,
                summary_json,
                result.latency_ms,
            ],
        )?;
        let id = db.last_insert_rowid();

        if history_limit > 0 {
            db.execute(
                "DELETE FROM history WHERE id NOT IN (SELECT id FROM history ORDER BY id DESC LIMIT ?1)",
                rusqlite::params![history_limit],
            )?;
        }

        Ok(HistoryEntry {
            id,
            timestamp: chrono::Utc::now().to_rfc3339(),
            input_text: result.text.clone(),
            redacted_text: result.redacted_text.clone(),
            detected_spans: result.detected_spans.clone(),
            summary: result.summary.clone(),
            latency_ms: result.latency_ms,
        })
    }

    pub fn get_entries(
        &self,
        cursor: Option<i64>,
        limit: u32,
    ) -> Result<(Vec<HistoryEntry>, bool)> {
        let db = self.db.lock().unwrap();
        let fetch_limit = limit + 1;

        let mut stmt = if let Some(cursor) = cursor {
            let mut s = db.prepare(
                "SELECT id, timestamp, input_text, redacted_text, detected_spans, summary, latency_ms
                 FROM history WHERE id < ?1 ORDER BY id DESC LIMIT ?2",
            )?;
            let rows = s.query_map(rusqlite::params![cursor, fetch_limit], Self::map_row)?;
            rows.collect::<Result<Vec<_>, _>>()?
        } else {
            let mut s = db.prepare(
                "SELECT id, timestamp, input_text, redacted_text, detected_spans, summary, latency_ms
                 FROM history ORDER BY id DESC LIMIT ?1",
            )?;
            let rows = s.query_map(rusqlite::params![fetch_limit], Self::map_row)?;
            rows.collect::<Result<Vec<_>, _>>()?
        };

        let has_more = stmt.len() > limit as usize;
        if has_more {
            stmt.truncate(limit as usize);
        }

        Ok((stmt, has_more))
    }

    pub fn delete_entry(&self, id: i64) -> Result<()> {
        let db = self.db.lock().unwrap();
        db.execute("DELETE FROM history WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        let db = self.db.lock().unwrap();
        db.execute("DELETE FROM history", [])?;
        Ok(())
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<HistoryEntry> {
        let spans_json: String = row.get(4)?;
        let summary_json: String = row.get(5)?;

        Ok(HistoryEntry {
            id: row.get(0)?,
            timestamp: row.get(1)?,
            input_text: row.get(2)?,
            redacted_text: row.get(3)?,
            detected_spans: serde_json::from_str(&spans_json).unwrap_or_default(),
            summary: serde_json::from_str(&summary_json).unwrap_or_default(),
            latency_ms: row.get(6)?,
        })
    }
}
