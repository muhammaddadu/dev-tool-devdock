//! Persistence for the `settings` key/value table. Used for things that
//! need to outlive an app restart but don't deserve their own schema —
//! refresh interval, denylist tweaks, etc.

use std::collections::HashMap;

use anyhow::{Context, Result};
use sqlx::{Row, SqlitePool};

use crate::utils::time;

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await
        .with_context(|| format!("reading setting {key}"))?;
    Ok(row.map(|r| r.get::<String, _>("value")))
}

pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO settings (key, value, updated_at)
        VALUES (?, ?, ?)
        ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at
        "#,
    )
    .bind(key)
    .bind(value)
    .bind(time::iso_now())
    .execute(pool)
    .await
    .with_context(|| format!("writing setting {key}"))?;
    Ok(())
}

pub async fn list(pool: &SqlitePool) -> Result<HashMap<String, String>> {
    let rows = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(pool)
        .await
        .context("listing settings")?;
    Ok(rows
        .into_iter()
        .map(|r| (r.get::<String, _>("key"), r.get::<String, _>("value")))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqliteConnectOptions;
    use std::str::FromStr;

    async fn fresh_pool() -> SqlitePool {
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::from_str("sqlite::memory:")
                .unwrap()
                .foreign_keys(true),
        )
        .await
        .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    #[tokio::test]
    async fn set_then_get_returns_value() {
        let pool = fresh_pool().await;
        assert!(get(&pool, "missing").await.unwrap().is_none());
        set(&pool, "theme", "dark").await.unwrap();
        assert_eq!(get(&pool, "theme").await.unwrap().as_deref(), Some("dark"));
    }

    #[tokio::test]
    async fn set_overwrites_existing_value() {
        let pool = fresh_pool().await;
        set(&pool, "interval", "1500").await.unwrap();
        set(&pool, "interval", "3000").await.unwrap();
        assert_eq!(
            get(&pool, "interval").await.unwrap().as_deref(),
            Some("3000"),
        );
    }

    #[tokio::test]
    async fn list_returns_all_settings() {
        let pool = fresh_pool().await;
        set(&pool, "a", "1").await.unwrap();
        set(&pool, "b", "2").await.unwrap();
        let all = list(&pool).await.unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all.get("a"), Some(&"1".to_string()));
        assert_eq!(all.get("b"), Some(&"2".to_string()));
    }
}
