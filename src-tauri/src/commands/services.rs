use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::app_state::AppState;
use crate::commands::{AppError, AppResult};
use crate::core::editors::{self, EditorInfo};
use crate::core::managed_runtime::{self, ManagedProcessRecord};
use crate::core::service_engine;
use crate::models::saved_service::SavedService;
use crate::models::service::{ServiceBucket, ServiceView};
use crate::storage::repositories::services_repo::{self, NewSavedService, ServiceUpdate};
use crate::tray;
use crate::utils::paths;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDetectedServiceInput {
    pub label: String,
    pub command: String,
    pub cwd: String,
    pub expected_ports: Vec<u16>,
    pub detected_pid: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServiceRunView {
    pub service_id: String,
    pub status: String,
}

#[tauri::command]
pub async fn list_services(
    app: AppHandle,
    state: State<'_, AppState>,
    db: State<'_, SqlitePool>,
) -> AppResult<Vec<ServiceView>> {
    let services = build_services(&state, &db).await?;
    publish_dev_badge(&app, &services);
    Ok(services)
}

#[tauri::command]
pub async fn refresh_services(
    app: AppHandle,
    state: State<'_, AppState>,
    db: State<'_, SqlitePool>,
) -> AppResult<Vec<ServiceView>> {
    let services = build_services(&state, &db).await?;
    publish_dev_badge(&app, &services);
    Ok(services)
}

async fn build_services(
    state: &State<'_, AppState>,
    db: &SqlitePool,
) -> AppResult<Vec<ServiceView>> {
    let detected = detected_services(state)?;
    let saved = services_repo::list(db)
        .await
        .map_err(|e| AppError::Storage(e.to_string()))?;

    // Touch last_seen_at for every saved service whose port is currently
    // bound. Best effort — a failed update shouldn't blank the panel.
    for v in &detected {
        if let Some(port) = v.port {
            if let Some(matched) = saved.iter().find(|s| s.expected_ports.contains(&port)) {
                let _ = services_repo::touch_last_seen(db, &matched.id).await;
            }
        }
    }

    let managed = state.runtime.snapshot();
    Ok(service_engine::merge(detected, &saved, &managed))
}

fn publish_dev_badge(app: &AppHandle, services: &[ServiceView]) {
    let count = services
        .iter()
        .filter(|s| s.bucket == ServiceBucket::Dev)
        .count();
    tray::set_dev_badge(app, count);
}

fn detected_services(state: &State<'_, AppState>) -> AppResult<Vec<ServiceView>> {
    let sockets = state
        .platform
        .list_listening_sockets()
        .map_err(|e| AppError::Platform(e.to_string()))?;

    let enriched = {
        let mut enricher = state
            .enricher
            .lock()
            .map_err(|e| AppError::Other(format!("enricher poisoned: {e}")))?;
        enricher.enrich(&*state.platform, &sockets)
    };

    Ok(service_engine::service_views(&enriched, &state.classifier))
}

#[tauri::command]
pub async fn save_detected_service(
    input: SaveDetectedServiceInput,
    db: State<'_, SqlitePool>,
) -> AppResult<SavedService> {
    if input.label.trim().is_empty() {
        return Err(AppError::InvalidInput("label is required".into()));
    }
    if input.command.trim().is_empty() {
        return Err(AppError::InvalidInput("command is required".into()));
    }
    if input.cwd.trim().is_empty() {
        return Err(AppError::InvalidInput("cwd is required".into()));
    }
    let new = NewSavedService {
        label: input.label.trim().to_string(),
        command: input.command.trim().to_string(),
        cwd: input.cwd.trim().to_string(),
        expected_ports: input.expected_ports,
        project_id: None,
        created_from: "detected".to_string(),
    };
    services_repo::insert(&db, new)
        .await
        .map_err(|e| AppError::Storage(e.to_string()))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServiceInput {
    pub label: String,
    pub command: String,
    pub cwd: String,
    pub expected_ports: Vec<u16>,
}

#[tauri::command]
pub async fn update_saved_service(
    id: String,
    input: UpdateServiceInput,
    db: State<'_, SqlitePool>,
) -> AppResult<SavedService> {
    if input.label.trim().is_empty() {
        return Err(AppError::InvalidInput("label is required".into()));
    }
    if input.command.trim().is_empty() {
        return Err(AppError::InvalidInput("command is required".into()));
    }
    if input.cwd.trim().is_empty() {
        return Err(AppError::InvalidInput("cwd is required".into()));
    }
    services_repo::update(
        &db,
        &id,
        ServiceUpdate {
            label: input.label.trim().to_string(),
            command: input.command.trim().to_string(),
            cwd: input.cwd.trim().to_string(),
            expected_ports: input.expected_ports,
        },
    )
    .await
    .map_err(|e| AppError::Storage(e.to_string()))
}

#[tauri::command]
pub async fn list_saved_services(db: State<'_, SqlitePool>) -> AppResult<Vec<SavedService>> {
    services_repo::list(&db)
        .await
        .map_err(|e| AppError::Storage(e.to_string()))
}

#[tauri::command]
pub async fn unsave_service(id: String, db: State<'_, SqlitePool>) -> AppResult<()> {
    let removed = services_repo::delete(&db, &id)
        .await
        .map_err(|e| AppError::Storage(e.to_string()))?;
    if !removed {
        return Err(AppError::InvalidInput(format!("no saved service: {id}")));
    }
    Ok(())
}

#[tauri::command]
pub async fn set_service_pinned(
    id: String,
    pinned: bool,
    db: State<'_, SqlitePool>,
) -> AppResult<()> {
    services_repo::set_pinned(&db, &id, pinned)
        .await
        .map_err(|e| AppError::Storage(e.to_string()))
}

#[tauri::command]
pub async fn run_service(
    service_id: String,
    state: State<'_, AppState>,
    db: State<'_, SqlitePool>,
) -> AppResult<ManagedProcessRecord> {
    let saved = load_saved(&db, &service_id).await?;
    state
        .runtime
        .spawn(&saved)
        .await
        .map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub async fn stop_service(service_id: String, state: State<'_, AppState>) -> AppResult<()> {
    state
        .runtime
        .stop(&service_id)
        .map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub async fn restart_service(
    service_id: String,
    state: State<'_, AppState>,
    db: State<'_, SqlitePool>,
) -> AppResult<ManagedProcessRecord> {
    let saved = load_saved(&db, &service_id).await?;
    managed_runtime::restart(&state.runtime, &saved)
        .await
        .map_err(|e| AppError::Other(e.to_string()))
}

async fn load_saved(db: &SqlitePool, service_id: &str) -> AppResult<SavedService> {
    let all = services_repo::list(db)
        .await
        .map_err(|e| AppError::Storage(e.to_string()))?;
    all.into_iter()
        .find(|s| s.id == service_id)
        .ok_or_else(|| AppError::InvalidInput(format!("no saved service: {service_id}")))
}

#[tauri::command]
pub async fn open_url(app: AppHandle, url: String) -> AppResult<()> {
    if !is_safe_url(&url) {
        return Err(AppError::InvalidInput(format!(
            "refusing to open URL: {url}"
        )));
    }
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub async fn open_path(app: AppHandle, path: String) -> AppResult<()> {
    if path.trim().is_empty() {
        return Err(AppError::InvalidInput("empty path".into()));
    }
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| AppError::Other(e.to_string()))
}

#[tauri::command]
pub async fn open_in_terminal(path: String) -> AppResult<()> {
    if path.trim().is_empty() {
        return Err(AppError::InvalidInput("empty path".into()));
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .args(["-a", "Terminal"])
            .arg(&path)
            .spawn()
            .map_err(AppError::Io)?;
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        Err(AppError::NotImplemented("open_in_terminal"))
    }
}

#[tauri::command]
pub async fn list_editors() -> AppResult<Vec<EditorInfo>> {
    Ok(editors::detect_editors())
}

#[tauri::command]
pub async fn open_in_editor(editor: String, path: String) -> AppResult<()> {
    if path.trim().is_empty() {
        return Err(AppError::InvalidInput("empty path".into()));
    }
    editors::open_in(&editor, &path).map_err(AppError::Other)
}

#[tauri::command]
pub async fn quit_app(
    stop_managed: bool,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    if stop_managed {
        let snapshot = state.runtime.snapshot();
        for service_id in snapshot.keys() {
            let _ = state.runtime.stop(service_id);
        }
        // Give SIGTERM up to 3 seconds to drain. Most dev servers shut down
        // in <500ms; this is just a ceiling for stubborn ones. The 5-second
        // SIGKILL escalation inside `stop()` is still on its own timer, but
        // we exit before it would have fired — that's intentional, anything
        // still hanging is the user's problem to investigate next session.
        for _ in 0..30 {
            if state.runtime.snapshot().is_empty() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    app.exit(0);
    Ok(())
}

const DEFAULT_LOG_TAIL_BYTES: u64 = 64 * 1024;

#[tauri::command]
pub async fn read_log_tail(path: String, max_bytes: Option<u64>) -> AppResult<String> {
    // Sandbox: only allow reads inside DevDock's own logs directory.
    // Canonicalize to defeat ".."/symlink escape attempts.
    let log_dir = paths::app_data_dir().join("logs");
    let canonical_dir = std::fs::canonicalize(&log_dir).map_err(AppError::Io)?;
    let canonical_target = std::fs::canonicalize(&path)
        .map_err(|e| AppError::InvalidInput(format!("log file not found ({e})")))?;
    if !canonical_target.starts_with(&canonical_dir) {
        return Err(AppError::InvalidInput(
            "refusing to read paths outside the logs directory".into(),
        ));
    }

    let cap = max_bytes.unwrap_or(DEFAULT_LOG_TAIL_BYTES).min(1024 * 1024);
    tail_file(&canonical_target, cap).map_err(AppError::Io)
}

fn tail_file(path: &std::path::Path, max_bytes: u64) -> std::io::Result<String> {
    use std::io::{Read, Seek, SeekFrom};
    let mut file = std::fs::File::open(path)?;
    let size = file.metadata()?.len();
    let start = size.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))?;
    let mut buf = Vec::with_capacity(max_bytes as usize);
    file.read_to_end(&mut buf)?;
    // If we started mid-line, drop the first partial line so the user sees
    // clean output rather than a stray fragment.
    let slice: &[u8] = if start > 0 {
        match buf.iter().position(|&b| b == b'\n') {
            Some(i) => &buf[i + 1..],
            None => &buf,
        }
    } else {
        &buf
    };
    Ok(String::from_utf8_lossy(slice).into_owned())
}

/// Whitelist URL schemes we're willing to launch from a command. The list is
/// intentionally narrow — DevDock should never end up shelling out to
/// arbitrary URI handlers because a service exposed a creative `url`.
fn is_safe_url(url: &str) -> bool {
    matches!(
        url.split_once(':').map(|(s, _)| s),
        Some("http") | Some("https"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_url_only_http_and_https() {
        assert!(is_safe_url("http://127.0.0.1:3000"));
        assert!(is_safe_url("https://example.com"));
        assert!(!is_safe_url("file:///etc/passwd"));
        assert!(!is_safe_url("javascript:alert(1)"));
        assert!(!is_safe_url("custom-scheme:foo"));
        assert!(!is_safe_url(""));
    }
}
