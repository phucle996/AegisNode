// Quản lý Node Records, Heartbeat và Node Labels trong PostgreSQL

use aegis_core::{AegisError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::PgRepository;

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

impl PgRepository {
    /// Đăng ký hoặc cập nhật Heartbeat cho Linux Node (Upsert)
    pub async fn upsert_node(
        &self,
        hostname: &str,
        ip_address: &str,
        labels: &serde_json::Value,
        version: &str,
    ) -> Result<NodeRecord> {
        // Thực thi SQL Upsert Node trong CSDL
        let record = sqlx::query_as::<_, NodeRecord>(
            r#"
            INSERT INTO nodes (hostname, ip_address, status, labels, version, last_seen_at)
            VALUES ($1, $2, 'ONLINE', $3, $4, CURRENT_TIMESTAMP)
            ON CONFLICT (hostname) DO UPDATE SET
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

    /// Cập nhật Heartbeat của Node theo Node ID
    pub async fn update_node_heartbeat(&self, node_id: Uuid, status: &str) -> Result<()> {
        // Cập nhật trạng thái và thời gian last_seen_at của Node
        sqlx::query(
            r#"
            UPDATE nodes
            SET status = $1, last_seen_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(status)
        .bind(node_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to update node heartbeat: {e}")))?;

        Ok(())
    }

    /// Lấy danh sách tất cả các Nodes trong Cluster
    pub async fn list_nodes(&self) -> Result<Vec<NodeRecord>> {
        // Lấy danh sách toàn bộ các Node bản ghi sắp xếp giảm dần theo thời gian
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

    /// Cập nhật nhãn (labels) của Node theo Node ID trong PostgreSQL
    pub async fn update_node_labels(&self, node_id: Uuid, labels: &serde_json::Value) -> Result<()> {
        // Thực thi câu lệnh SQL UPDATE cập nhật trường labels và last_seen_at
        sqlx::query(
            r#"
            UPDATE nodes
            SET labels = $1, last_seen_at = CURRENT_TIMESTAMP
            WHERE id = $2
            "#,
        )
        .bind(labels)
        .bind(node_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to update node labels: {e}")))?;

        Ok(())
    }
}
