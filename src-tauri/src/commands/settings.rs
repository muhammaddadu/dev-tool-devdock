use std::collections::HashMap;

use sqlx::SqlitePool;
use tauri::State;

use crate::commands::{AppError, AppResult};
use crate::storage::repositories::settings_repo;

#[tauri::command]
pub async fn get_setting(key: String, db: State<'_, SqlitePool>) -> AppResult<Option<String>> {
    settings_repo::get(&db, &key)
        .await
        .map_err(|e| AppError::Storage(e.to_string()))
}

#[tauri::command]
pub async fn set_setting(key: String, value: String, db: State<'_, SqlitePool>) -> AppResult<()> {
    if key.trim().is_empty() {
        return Err(AppError::InvalidInput("setting key is required".into()));
    }
    settings_repo::set(&db, &key, &value)
        .await
        .map_err(|e| AppError::Storage(e.to_string()))
}

#[tauri::command]
pub async fn list_settings(db: State<'_, SqlitePool>) -> AppResult<HashMap<String, String>> {
    settings_repo::list(&db)
        .await
        .map_err(|e| AppError::Storage(e.to_string()))
}
