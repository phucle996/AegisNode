// PostgreSQL Repository Layer cho AegisNode Controller Server (`aegisnode server`)
// Phục vụ lưu trữ tập trung Multi-Node, chống Race Condition qua Optimistic Locking

use std::time::Duration;

use aegis_core::{AegisError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// DTO biểu diễn thông tin Node đăng ký trong PostgreSQL
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct NodeRecord {
    pub id: Uuid,
    pub hostname: String,
    pub ip_address: String,
    pub status: String,
    pub labels: serde_json::Value,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

/// DTO biểu diễn API Token trong Controller
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ApiTokenRecord {
    pub id: Uuid,
    pub name: String,
    pub token_hash: String,
    pub scopes: Vec<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Controller PostgreSQL Repository Interface
#[derive(Clone)]
pub struct PgRepository {
    pool: Pool<Postgres>,
}

impl PgRepository {
    /// Khởi tạo PostgreSQL Connection Pool với timeout và max connections
    pub async fn connect(url: &str, max_connections: u32, timeout_sec: u64) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(timeout_sec))
            .connect(url)
            .await
            .map_err(|e| {
                AegisError::Storage(format!("Failed to connect to PostgreSQL at '{url}': {e}"))
            })?;

        Ok(Self { pool })
    }

    /// Khởi tạo instance từ PgPool sẵn có
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }

    /// Trả về tham chiếu tới inner pool
    pub fn pool(&self) -> &Pool<Postgres> {
        &self.pool
    }

    /// Đăng ký hoặc cập nhật Heartbeat cho Linux Node (Upsert)
    pub async fn upsert_node(
        &self,
        hostname: &str,
        ip_address: &str,
        labels: &serde_json::Value,
        version: &str,
    ) -> Result<NodeRecord> {
        let record = sqlx::query_as::<_, NodeRecord>(
            r#"
            INSERT INTO nodes (hostname, ip_address, status, labels, version, last_seen_at)
            VALUES ($1, $2, 'ONLINE', $3, $4, CURRENT_TIMESTAMP)
            ON CONFLICT (id) DO UPDATE SET
                hostname = EXCLUDED.hostname,
                ip_address = EXCLUDED.ip_address,
                status = 'ONLINE',
                labels = EXCLUDED.labels,
                version = EXCLUDED.version,
                last_seen_at = CURRENT_TIMESTAMP
            RETURNING id, hostname, ip_address, status, labels, version, created_at, last_seen_at
            "#,
        )
        .bind(hostname)
        .bind(ip_address)
        .bind(labels)
        .bind(version)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to upsert node: {e}")))?;

        Ok(record)
    }

    /// Lấy danh sách tất cả các Nodes trong Cluster
    pub async fn list_nodes(&self) -> Result<Vec<NodeRecord>> {
        let nodes = sqlx::query_as::<_, NodeRecord>(
            r#"
            SELECT id, hostname, ip_address, status, labels, version, created_at, last_seen_at
            FROM nodes
            ORDER BY last_seen_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to list nodes: {e}")))?;

        Ok(nodes)
    }

    /// Tạo API Token mới cho Admin
    pub async fn create_api_token(&self, name: &str, token_hash: &str) -> Result<ApiTokenRecord> {
        let token = sqlx::query_as::<_, ApiTokenRecord>(
            r#"
            INSERT INTO api_tokens (name, token_hash, scopes)
            VALUES ($1, $2, ARRAY['read', 'write'])
            RETURNING id, name, token_hash, scopes, expires_at, created_at
            "#,
        )
        .bind(name)
        .bind(token_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to create API token: {e}")))?;

        Ok(token)
    }

    /// Xác thực tính hợp lệ của API Token
    pub async fn verify_api_token(&self, token_hash: &str) -> Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT 1 FROM api_tokens
            WHERE token_hash = $1 AND (expires_at IS NULL OR expires_at > CURRENT_TIMESTAMP)
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to verify token: {e}")))?;

        Ok(row.is_some())
    }
}
