// Quản lý kết nối cơ sở dữ liệu SQLite và tự động thực thi DDLS Migrations
// Tạo 7 bảng core: policy_versions, apply_executions, snapshots, audit_events, block_entries, agent_state, settings

use std::path::Path;

use aegis_core::{AegisError, Result};
use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

/// Khởi tạo SQLite Connection Pool và tự động chạy DDL Migrations
pub async fn init_sqlite_pool(db_path: &Path) -> Result<SqlitePool> {
    // Tạo parent directory nếu chưa có
    if let Some(parent) = db_path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
    }

    let opts = SqliteConnectOptions::new()
        .filename(db_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await
        .map_err(|e| {
            AegisError::Storage(format!("Failed to connect to SQLite at '{db_path:?}': {e}"))
        })?;

    run_migrations(&pool).await?;

    Ok(pool)
}

/// Khởi tạo SQLite Connection Pool in-memory cho Testing
pub async fn init_in_memory_pool() -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to connect in-memory SQLite: {e}")))?;

    run_migrations(&pool).await?;

    Ok(pool)
}

/// Thực thi DDL Migrations tự động tạo 7 bảng dữ liệu chuẩn
async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    let migration_sql = r#"
CREATE TABLE IF NOT EXISTS policy_versions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version INTEGER NOT NULL,
    policy_hash TEXT NOT NULL,
    content_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS apply_executions (
    id TEXT PRIMARY KEY,
    policy_id TEXT NOT NULL,
    snapshot_id TEXT NOT NULL,
    state TEXT NOT NULL,
    timeout_seconds INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    error_message TEXT
);

CREATE TABLE IF NOT EXISTS snapshots (
    id TEXT PRIMARY KEY,
    policy_hash TEXT NOT NULL,
    ruleset_content TEXT NOT NULL,
    checksum_sha256 TEXT NOT NULL,
    reason TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY,
    action TEXT NOT NULL,
    actor TEXT NOT NULL,
    resource TEXT NOT NULL,
    details_json TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS block_entries (
    id TEXT PRIMARY KEY,
    target TEXT NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS agent_state (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
"#;

    sqlx::query(migration_sql)
        .execute(pool)
        .await
        .map_err(|e| {
            AegisError::Storage(format!("Failed to execute SQLite DDL migrations: {e}"))
        })?;

    Ok(())
}
