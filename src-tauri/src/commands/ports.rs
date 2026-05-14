use tauri::State;

use crate::app_state::AppState;
use crate::commands::{AppError, AppResult};

#[tauri::command]
pub async fn kill_detected_process(pid: u32, state: State<'_, AppState>) -> AppResult<()> {
    if pid == 0 {
        return Err(AppError::InvalidInput("pid 0 is not a valid target".into()));
    }
    // Belt-and-braces: refuse to signal our own process. The frontend already
    // hides Kill on services we manage, but a stray bug in classification
    // shouldn't be able to take DevDock down.
    if pid == std::process::id() {
        return Err(AppError::InvalidInput(
            "refusing to kill DevDock itself".into(),
        ));
    }
    state
        .platform
        .terminate_process(pid)
        .map_err(|e| AppError::Platform(e.to_string()))
}
