use serde::Serialize;
use tauri::State;

use crate::app_state::AppState;
use crate::commands::{AppError, AppResult};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScanSuggestion {
    pub label: String,
    pub command: String,
    pub cwd: String,
    pub expected_ports: Vec<u16>,
    pub confidence: String,
    pub reason: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectScanResult {
    pub project_root: String,
    pub suggestions: Vec<ProjectScanSuggestion>,
}

#[tauri::command]
pub async fn scan_project(
    _path: String,
    _state: State<'_, AppState>,
) -> AppResult<ProjectScanResult> {
    Err(AppError::NotImplemented("scan_project"))
}
