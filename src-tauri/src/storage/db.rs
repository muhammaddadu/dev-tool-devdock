//! SQLite handle setup.
//!
//! Initialised once at app startup. The connection pool is `Manage`d via
//! Tauri so commands can pull it via `State<'_, SqlitePool>`.

use std::path::Path;

use anyhow::{Context, Result};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

/// Open (or create) the SQLite database at `path`, run pending migrations,
/// and return a connection pool.
pub async fn open(path: &Path) -> Result<SqlitePool> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("creating data directory {}", parent.display())
        })?;
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        // Enable WAL for fast concurrent reads alongside the writer. Reduces
        // tail latency on the UI when a background task is touching the DB.
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        // foreign_keys defaults off in SQLite. We rely on `ON DELETE CASCADE`
        // for services_runs and ai_suggestions, so turn it on.
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(options)
        .await
        .with_context(|| format!("opening sqlite at {}", path.display()))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("running embedded migrations")?;

    Ok(pool)
}
