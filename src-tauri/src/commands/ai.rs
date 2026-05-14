use serde::{Deserialize, Serialize};
use tauri::State;

use crate::app_state::AppState;
use crate::commands::projects::ProjectScanResult;
use crate::commands::{AppError, AppResult};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProviderView {
    pub id: String,
    pub display_name: String,
    pub available: bool,
    pub version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiProjectAnalysisRequest {
    pub provider_id: String,
    pub project_root: String,
}

#[tauri::command]
pub async fn detect_ai_providers(_state: State<'_, AppState>) -> AppResult<Vec<AiProviderView>> {
    Ok(Vec::new())
}

#[tauri::command]
pub async fn analyze_project_with_ai(
    _input: AiProjectAnalysisRequest,
    _state: State<'_, AppState>,
) -> AppResult<ProjectScanResult> {
    Err(AppError::NotImplemented("analyze_project_with_ai"))
}
