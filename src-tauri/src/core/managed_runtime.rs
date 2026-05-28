//! Tracks services DevDock itself has launched.
//!
//! Each spawn produces a `ManagedProcessRecord` keyed by saved-service id.
//! The `Child` itself is moved into a tokio task that `.wait()`s on it and
//! removes the record when the process exits, so the registry stays an
//! accurate view of what's actually live.
//!
//! Process group semantics: every spawn becomes its own group leader via
//! `process_group(0)`. `kill(-pid, SIGTERM)` then signals the entire tree,
//! which is what we want for `npm run dev` style scripts that fork helpers.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::models::saved_service::SavedService;
use crate::utils::{ids, paths, time};

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("service is already running")]
    AlreadyRunning,
    #[error("service is not running")]
    NotRunning,
    #[error("spawned process exited before reporting a pid")]
    NoPid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ManagedProcessRecord {
    pub run_id: String,
    pub service_id: String,
    pub root_pid: u32,
    pub started_at_unix: u64,
    pub log_path: String,
}

#[derive(Clone, Default)]
pub struct ManagedRuntime {
    runs: Arc<Mutex<HashMap<String, ManagedProcessRecord>>>,
}

impl ManagedRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_running(&self, service_id: &str) -> bool {
        self.runs.lock().unwrap().contains_key(service_id)
    }

    pub fn snapshot(&self) -> HashMap<String, ManagedProcessRecord> {
        self.runs.lock().unwrap().clone()
    }

    /// Spawn a saved service. Returns immediately once the process is up;
    /// monitoring (and cleanup on exit) happens in a background task.
    pub async fn spawn(&self, saved: &SavedService) -> Result<ManagedProcessRecord, RuntimeError> {
        if self.is_running(&saved.id) {
            return Err(RuntimeError::AlreadyRunning);
        }

        let logs_dir = paths::app_data_dir().join("logs");
        std::fs::create_dir_all(&logs_dir)?;
        let run_id = ids::new_id();
        let log_path = logs_dir.join(format!("{run_id}.log"));

        let log_file = std::fs::File::create(&log_path)?;
        let log_err = log_file.try_clone()?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let mut cmd = Command::new(&shell);
        // -i (interactive) makes zsh/bash source the user's rc files so PATH
        // matches what they get in Terminal. -c runs the command and exits.
        cmd.args(["-i", "-c", &saved.command])
            .current_dir(&saved.cwd)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::from(log_file))
            .stderr(std::process::Stdio::from(log_err));
        // 0 = become own process group leader. kill(-pid, ...) then targets
        // every descendant (vite -> esbuild workers, etc.).
        #[cfg(unix)]
        cmd.process_group(0);

        let mut child = cmd.spawn()?;
        let root_pid = child.id().ok_or(RuntimeError::NoPid)?;

        let record = ManagedProcessRecord {
            run_id,
            service_id: saved.id.clone(),
            root_pid,
            started_at_unix: time::now().timestamp() as u64,
            log_path: log_path.to_string_lossy().to_string(),
        };

        // Track before spawning the wait task so a fast-exit doesn't race the
        // insertion.
        self.runs
            .lock()
            .unwrap()
            .insert(saved.id.clone(), record.clone());

        let runs = self.runs.clone();
        let service_id = saved.id.clone();
        let run_id_for_log = record.run_id.clone();
        tokio::spawn(async move {
            let status = child.wait().await;
            runs.lock().unwrap().remove(&service_id);
            match status {
                Ok(s) => tracing::info!(
                    service_id,
                    run_id = run_id_for_log,
                    code = ?s.code(),
                    "managed process exited"
                ),
                Err(err) => tracing::warn!(
                    service_id,
                    run_id = run_id_for_log,
                    error = %err,
                    "wait() on managed process failed"
                ),
            }
        });

        Ok(record)
    }

    /// Request a graceful stop. Sends SIGTERM to the process group; the wait
    /// task running in the background notices the exit and unregisters.
    pub fn stop(&self, service_id: &str) -> Result<(), RuntimeError> {
        let record = self
            .runs
            .lock()
            .unwrap()
            .get(service_id)
            .cloned()
            .ok_or(RuntimeError::NotRunning)?;
        send_group_signal(record.root_pid as i32, Signal::Term);
        // Best-effort SIGKILL escalation after a grace period in case the
        // process ignores SIGTERM. Runs in the background; we don't await it.
        let runs = self.runs.clone();
        let sid = service_id.to_string();
        let pid = record.root_pid as i32;
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            if runs.lock().unwrap().contains_key(&sid) {
                send_group_signal(pid, Signal::Kill);
            }
        });
        Ok(())
    }

    pub fn log_path(&self, service_id: &str) -> Option<PathBuf> {
        self.runs
            .lock()
            .unwrap()
            .get(service_id)
            .map(|r| PathBuf::from(&r.log_path))
    }
}

#[allow(dead_code)]
enum Signal {
    Term,
    Kill,
}

#[cfg(unix)]
fn send_group_signal(root_pid: i32, sig: Signal) {
    let signum = match sig {
        Signal::Term => libc::SIGTERM,
        Signal::Kill => libc::SIGKILL,
    };
    // Negative pid → entire process group.
    unsafe {
        libc::kill(-root_pid, signum);
    }
}

#[cfg(not(unix))]
fn send_group_signal(_pid: i32, _sig: Signal) {
    // Windows support belongs to a later milestone via Job Objects.
}

/// Convenience wrapper: stop the service, wait briefly for the registry to
/// clear, then spawn fresh. Designed for the Restart button.
pub async fn restart(
    rt: &ManagedRuntime,
    saved: &SavedService,
) -> Result<ManagedProcessRecord, RuntimeError> {
    if rt.is_running(&saved.id) {
        let _ = rt.stop(&saved.id);
        // Poll up to ~3s for the wait task to see exit. This is more reliable
        // than a fixed sleep, especially for cleanly-shutting servers.
        for _ in 0..30 {
            if !rt.is_running(&saved.id) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    rt.spawn(saved).await
}

#[allow(dead_code)]
fn log_dir() -> PathBuf {
    paths::app_data_dir().join("logs")
}

#[allow(dead_code)]
pub fn log_dir_path() -> &'static Path {
    // Intentionally unused — exposed for future log-viewer hookup.
    static LOG_DIR: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    LOG_DIR.get_or_init(|| paths::app_data_dir().join("logs"))
}
