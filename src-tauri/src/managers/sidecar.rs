use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;

const IPC_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidecarModelInfo {
    pub name: String,
    pub architecture: String,
    pub num_labels: i32,
    pub hidden_size: i64,
    pub vocab_size: i64,
    pub max_position_embeddings: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedSpan {
    pub label: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub placeholder: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionResult {
    pub schema_version: u32,
    pub text: String,
    pub redacted_text: String,
    pub detected_spans: Vec<DetectedSpan>,
    pub summary: HashMap<String, u32>,
    pub latency_ms: f64,
}

pub struct SidecarManager {
    child: Mutex<Option<Child>>,
    stdin: Mutex<Option<std::process::ChildStdin>>,
    stdout: Mutex<Option<BufReader<std::process::ChildStdout>>>,
    next_id: AtomicU64,
    model_info: Mutex<Option<SidecarModelInfo>>,
}

impl SidecarManager {
    pub fn spawn(binary_path: &std::path::Path) -> Result<Self> {
        log::info!("Spawning sidecar: {}", binary_path.display());

        let mut command = Command::new(binary_path);
        command
            .env("PYTHONUTF8", "1")
            .env("PYTHONIOENCODING", "utf-8");

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("Failed to spawn sidecar: {}", binary_path.display()))?;

        let child_stdin = child.stdin.take().context("Failed to get sidecar stdin")?;
        let child_stdout = child
            .stdout
            .take()
            .context("Failed to get sidecar stdout")?;
        let mut reader = BufReader::new(child_stdout);

        let mut line = String::new();
        reader
            .read_line(&mut line)
            .context("Failed to read sidecar ready signal")?;

        let ready: serde_json::Value =
            serde_json::from_str(line.trim()).context("Invalid ready JSON from sidecar")?;

        if ready.get("ready").and_then(|v| v.as_bool()) != Some(true) {
            anyhow::bail!("Sidecar did not send ready signal, got: {}", line.trim());
        }

        log::info!("Sidecar ready");

        Ok(Self {
            child: Mutex::new(Some(child)),
            stdin: Mutex::new(Some(child_stdin)),
            stdout: Mutex::new(Some(reader)),
            next_id: AtomicU64::new(1),
            model_info: Mutex::new(None),
        })
    }

    fn call(&self, cmd: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let req_id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let msg = serde_json::json!({
            "id": req_id,
            "cmd": cmd,
            "params": params,
        });

        let msg_str = serde_json::to_string(&msg)? + "\n";

        {
            let mut stdin_guard = self
                .stdin
                .lock()
                .map_err(|e| anyhow::anyhow!("stdin lock poisoned: {}", e))?;
            let stdin = stdin_guard
                .as_mut()
                .context("Sidecar stdin not available")?;
            stdin
                .write_all(msg_str.as_bytes())
                .context("Failed to write to sidecar stdin")?;
            stdin.flush().context("Failed to flush sidecar stdin")?;
        }

        let read_result = {
            let mut stdout_guard = self
                .stdout
                .lock()
                .map_err(|e| anyhow::anyhow!("stdout lock poisoned: {}", e))?;
            let reader = stdout_guard
                .as_mut()
                .context("Sidecar stdout not available")?;

            let deadline = std::time::Instant::now() + IPC_TIMEOUT;

            loop {
                if std::time::Instant::now() > deadline {
                    break Err(anyhow::anyhow!(
                        "Sidecar IPC timeout after {}s for command '{}'",
                        IPC_TIMEOUT.as_secs(),
                        cmd
                    ));
                }

                let mut line = String::new();
                let n = reader
                    .read_line(&mut line)
                    .context("Failed to read sidecar response")?;
                if n == 0 {
                    break Err(anyhow::anyhow!("Sidecar process closed stdout (crashed?)"));
                }
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    break Ok(parsed);
                }
                log::debug!("Sidecar non-JSON output: {}", trimmed);
            }
        };

        let resp = read_result?;

        if let Some(err) = resp.get("error").and_then(|v| v.as_str()) {
            anyhow::bail!("Sidecar error: {}", err);
        }

        Ok(resp)
    }

    pub fn load(&self, model_path: &str) -> Result<SidecarModelInfo> {
        let resp = self.call("load", serde_json::json!({ "model_path": model_path }))?;

        let info: SidecarModelInfo = serde_json::from_value(
            resp.get("info")
                .cloned()
                .context("Missing 'info' in load response")?,
        )?;

        *self.model_info.lock().unwrap() = Some(info.clone());
        Ok(info)
    }

    pub fn get_info(&self) -> Result<SidecarModelInfo> {
        if let Some(info) = self.model_info.lock().unwrap().clone() {
            return Ok(info);
        }

        let resp = self.call("info", serde_json::json!({}))?;
        let info: SidecarModelInfo = serde_json::from_value(resp)?;
        *self.model_info.lock().unwrap() = Some(info.clone());
        Ok(info)
    }

    pub fn redact(&self, text: &str) -> Result<RedactionResult> {
        let resp = self.call("redact", serde_json::json!({ "text": text }))?;
        let result: RedactionResult = serde_json::from_value(resp)?;
        Ok(result)
    }

    pub fn is_healthy(&self) -> bool {
        self.call("health", serde_json::json!({}))
            .and_then(|resp| {
                Ok(resp
                    .get("status")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "ok")
                    .unwrap_or(false))
            })
            .unwrap_or(false)
    }

    pub fn kill(&self) {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(ref mut child) = *guard {
                let _ = child.kill();
                let _ = child.wait();
                log::info!("Sidecar process killed");
            }
            *guard = None;
        }
        if let Ok(mut s) = self.stdin.lock() {
            *s = None;
        }
        if let Ok(mut s) = self.stdout.lock() {
            *s = None;
        }
    }
}

impl Drop for SidecarManager {
    fn drop(&mut self) {
        self.kill();
    }
}
