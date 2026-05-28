//! Persistence for the `services` table.
//!
//! All queries are plain `sqlx::query` (no compile-time DATABASE_URL needed).
//! The `expected_ports` column is stored as a JSON array; the boundary between
//! Vec<u16> and `String` lives here so callers never see the raw form.

use anyhow::{Context, Result};
use sqlx::SqlitePool;

use crate::models::saved_service::SavedService;
use crate::utils::{ids, time};

#[derive(Debug, Clone)]
pub struct NewSavedService {
    pub label: String,
    pub command: String,
    pub cwd: String,
    pub expected_ports: Vec<u16>,
    pub project_id: Option<String>,
    pub created_from: String,
}

pub async fn list(pool: &SqlitePool) -> Result<Vec<SavedService>> {
    let rows = sqlx::query(
        r#"
        SELECT id, project_id, label, command, cwd, expected_ports,
               pinned, created_from, last_run_at, last_seen_at,
               created_at, updated_at
        FROM services
        ORDER BY pinned DESC, COALESCE(last_seen_at, created_at) DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .context("listing services")?;

    rows.into_iter().map(row_to_saved_service).collect()
}

pub async fn insert(pool: &SqlitePool, input: NewSavedService) -> Result<SavedService> {
    let id = ids::new_id();
    let now = time::iso_now();
    let ports_json =
        serde_json::to_string(&input.expected_ports).context("serializing expected_ports")?;

    sqlx::query(
        r#"
        INSERT INTO services
            (id, project_id, label, command, cwd, expected_ports,
             pinned, created_from, last_run_at, last_seen_at,
             created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, 0, ?, NULL, NULL, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&input.project_id)
    .bind(&input.label)
    .bind(&input.command)
    .bind(&input.cwd)
    .bind(&ports_json)
    .bind(&input.created_from)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await
    .context("inserting service")?;

    fetch_by_id(pool, &id)
        .await?
        .context("inserted service vanished")
}

#[derive(Debug, Clone)]
pub struct ServiceUpdate {
    pub label: String,
    pub command: String,
    pub cwd: String,
    pub expected_ports: Vec<u16>,
}

pub async fn update(pool: &SqlitePool, id: &str, input: ServiceUpdate) -> Result<SavedService> {
    let ports_json =
        serde_json::to_string(&input.expected_ports).context("serializing expected_ports")?;
    let res = sqlx::query(
        r#"
        UPDATE services
        SET label = ?, command = ?, cwd = ?, expected_ports = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(&input.label)
    .bind(&input.command)
    .bind(&input.cwd)
    .bind(&ports_json)
    .bind(time::iso_now())
    .bind(id)
    .execute(pool)
    .await
    .context("updating service")?;

    if res.rows_affected() == 0 {
        anyhow::bail!("no saved service with id {id}");
    }
    fetch_by_id(pool, id)
        .await?
        .context("service vanished after update")
}

pub async fn delete(pool: &SqlitePool, id: &str) -> Result<bool> {
    let res = sqlx::query("DELETE FROM services WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .context("deleting service")?;
    Ok(res.rows_affected() > 0)
}

pub async fn set_pinned(pool: &SqlitePool, id: &str, pinned: bool) -> Result<()> {
    sqlx::query("UPDATE services SET pinned = ?, updated_at = ? WHERE id = ?")
        .bind(pinned as i64)
        .bind(time::iso_now())
        .bind(id)
        .execute(pool)
        .await
        .context("setting pinned")?;
    Ok(())
}

/// Mark a saved service as seen running. Called from the refresh path when a
/// detected service's port matches one of the saved service's expected_ports.
pub async fn touch_last_seen(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("UPDATE services SET last_seen_at = ? WHERE id = ?")
        .bind(time::iso_now())
        .bind(id)
        .execute(pool)
        .await
        .context("touching last_seen_at")?;
    Ok(())
}

async fn fetch_by_id(pool: &SqlitePool, id: &str) -> Result<Option<SavedService>> {
    let maybe = sqlx::query(
        r#"
        SELECT id, project_id, label, command, cwd, expected_ports,
               pinned, created_from, last_run_at, last_seen_at,
               created_at, updated_at
        FROM services
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .context("fetching service by id")?;

    maybe.map(row_to_saved_service).transpose()
}

fn row_to_saved_service(row: sqlx::sqlite::SqliteRow) -> Result<SavedService> {
    use sqlx::Row;
    let ports_json: String = row.try_get("expected_ports")?;
    let expected_ports: Vec<u16> = serde_json::from_str(&ports_json)
        .with_context(|| format!("parsing expected_ports JSON: {ports_json}"))?;
    let pinned: i64 = row.try_get("pinned")?;
    Ok(SavedService {
        id: row.try_get("id")?,
        project_id: row.try_get("project_id")?,
        label: row.try_get("label")?,
        command: row.try_get("command")?,
        cwd: row.try_get("cwd")?,
        expected_ports,
        pinned: pinned != 0,
        created_from: row.try_get("created_from")?,
        last_run_at: row.try_get("last_run_at")?,
        last_seen_at: row.try_get("last_seen_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqliteConnectOptions;
    use std::str::FromStr;

    async fn fresh_pool() -> SqlitePool {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")
            .unwrap()
            .foreign_keys(true);
        let pool = SqlitePool::connect_with(options).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool
    }

    fn input(label: &str, ports: &[u16]) -> NewSavedService {
        NewSavedService {
            label: label.to_string(),
            command: "npm run dev".to_string(),
            cwd: "/Users/me/code/web".to_string(),
            expected_ports: ports.to_vec(),
            project_id: None,
            created_from: "detected".to_string(),
        }
    }

    #[tokio::test]
    async fn insert_then_list_returns_saved_service() {
        let pool = fresh_pool().await;
        let saved = insert(&pool, input("Frontend", &[3000])).await.unwrap();
        assert_eq!(saved.label, "Frontend");
        assert_eq!(saved.expected_ports, vec![3000]);
        assert!(!saved.pinned);

        let all = list(&pool).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, saved.id);
    }

    #[tokio::test]
    async fn pinned_services_sort_first() {
        let pool = fresh_pool().await;
        let _a = insert(&pool, input("A", &[3000])).await.unwrap();
        let b = insert(&pool, input("B", &[3001])).await.unwrap();
        set_pinned(&pool, &b.id, true).await.unwrap();

        let all = list(&pool).await.unwrap();
        assert_eq!(all[0].label, "B", "pinned should sort first");
        assert_eq!(all[1].label, "A");
    }

    #[tokio::test]
    async fn delete_removes_the_row() {
        let pool = fresh_pool().await;
        let saved = insert(&pool, input("X", &[3000])).await.unwrap();
        let removed = delete(&pool, &saved.id).await.unwrap();
        assert!(removed);
        assert!(list(&pool).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn update_overwrites_label_command_ports() {
        let pool = fresh_pool().await;
        let saved = insert(&pool, input("Old", &[3000])).await.unwrap();
        let updated = update(
            &pool,
            &saved.id,
            ServiceUpdate {
                label: "New".into(),
                command: "pnpm dev".into(),
                cwd: "/Users/me/code/new".into(),
                expected_ports: vec![3001, 3002],
            },
        )
        .await
        .unwrap();
        assert_eq!(updated.label, "New");
        assert_eq!(updated.command, "pnpm dev");
        assert_eq!(updated.expected_ports, vec![3001, 3002]);
    }

    #[tokio::test]
    async fn update_missing_id_errors() {
        let pool = fresh_pool().await;
        let err = update(
            &pool,
            "does-not-exist",
            ServiceUpdate {
                label: "X".into(),
                command: "X".into(),
                cwd: "X".into(),
                expected_ports: vec![1],
            },
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("no saved service"));
    }

    #[tokio::test]
    async fn touch_last_seen_updates_timestamp() {
        let pool = fresh_pool().await;
        let saved = insert(&pool, input("X", &[3000])).await.unwrap();
        assert!(saved.last_seen_at.is_none());
        touch_last_seen(&pool, &saved.id).await.unwrap();
        let refreshed = &list(&pool).await.unwrap()[0];
        assert!(refreshed.last_seen_at.is_some());
    }
}
