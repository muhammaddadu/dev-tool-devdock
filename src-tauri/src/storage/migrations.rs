//! Embedded SQLx migrations. Loaded from `src-tauri/migrations/`.

pub static INITIAL_SCHEMA: &str = include_str!("../../migrations/0001_initial.sql");
