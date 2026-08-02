// Repository Pattern triệt tiêu việc viết SQL rải rác trong API Handlers
// Định nghĩa các Traits Repository và triển khai SqliteRepository an toàn

use aegis_core::{AegisError, ExecutionId, PolicyId, Result, SnapshotId};
use aegis_models::firewall::FirewallPolicy;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

/// Record Audit Event lưu trong database
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditRecord {
    pub id: String,
    pub action: String,
    pub actor: String,
    pub resource: String,
    pub details_json: String,
    pub created_at: String,
}

#[async_trait]
pub trait PolicyRepository: Send + Sync {
    async fn save_policy(&self, policy: &FirewallPolicy, hash: &str) -> Result<()>;
    async fn get_latest_policy(&self) -> Result<Option<FirewallPolicy>>;
}

#[async_trait]
pub trait ExecutionRepository: Send + Sync {
    async fn save_execution(
        &self,
        id: &ExecutionId,
        policy_id: &PolicyId,
        snapshot_id: &SnapshotId,
        state: &str,
        timeout_seconds: u64,
    ) -> Result<()>;
    async fn update_execution_state(
        &self,
        id: &ExecutionId,
        state: &str,
        error: Option<&str>,
    ) -> Result<()>;
}

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn record_audit(
        &self,
        action: &str,
        actor: &str,
        resource: &str,
        details_json: &str,
    ) -> Result<()>;
    async fn list_audits(&self, limit: usize) -> Result<Vec<AuditRecord>>;
}

/// Trait quản lý thông tin các Node trong Cluster (NodeRepository)
#[async_trait]
pub trait NodeRepository: Send + Sync {
    // Đăng ký hoặc cập nhật Heartbeat cho Linux Node (Upsert)
    async fn upsert_node(
        &self,
        hostname: &str,
        ip_address: &str,
        labels: &serde_json::Value,
        version: &str,
    ) -> Result<()>;
    // Cập nhật Heartbeat của Node theo Node ID
    async fn update_node_heartbeat(&self, node_id: uuid::Uuid, status: &str) -> Result<()>;
    // Lấy danh sách tất cả các Nodes trong Cluster
    async fn list_nodes(&self) -> Result<Vec<serde_json::Value>>;
}

/// Triển khai SqliteRepository cho toàn bộ traits
#[derive(Clone)]
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PolicyRepository for SqliteRepository {
    async fn save_policy(&self, policy: &FirewallPolicy, hash: &str) -> Result<()> {
        let json = serde_json::to_string(policy)
            .map_err(|e| AegisError::Storage(format!("Failed to serialize policy: {e}")))?;

        sqlx::query(
            "INSERT INTO policy_versions (id, name, version, policy_hash, content_json, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(policy.metadata.id.as_str())
        .bind(&policy.metadata.name)
        .bind(policy.metadata.version as i64)
        .bind(hash)
        .bind(json)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to insert policy_version: {e}")))?;

        Ok(())
    }

    async fn get_latest_policy(&self) -> Result<Option<FirewallPolicy>> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT content_json FROM policy_versions ORDER BY version DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to query latest policy: {e}")))?;

        if let Some((json_str,)) = row {
            let policy: FirewallPolicy = serde_json::from_str(&json_str)
                .map_err(|e| AegisError::Storage(format!("Failed to parse stored policy: {e}")))?;
            Ok(Some(policy))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl ExecutionRepository for SqliteRepository {
    async fn save_execution(
        &self,
        id: &ExecutionId,
        policy_id: &PolicyId,
        snapshot_id: &SnapshotId,
        state: &str,
        timeout_seconds: u64,
    ) -> Result<()> {
        let now = Utc::now();
        let expires = now + chrono::Duration::seconds(timeout_seconds as i64);

        sqlx::query(
            "INSERT INTO apply_executions (id, policy_id, snapshot_id, state, timeout_seconds, created_at, expires_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(id.as_str())
        .bind(policy_id.as_str())
        .bind(snapshot_id.as_str())
        .bind(state)
        .bind(timeout_seconds as i64)
        .bind(now.to_rfc3339())
        .bind(expires.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to insert apply_execution: {e}")))?;

        Ok(())
    }

    async fn update_execution_state(
        &self,
        id: &ExecutionId,
        state: &str,
        error: Option<&str>,
    ) -> Result<()> {
        sqlx::query("UPDATE apply_executions SET state = ?, error_message = ? WHERE id = ?")
            .bind(state)
            .bind(error)
            .bind(id.as_str())
            .execute(&self.pool)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to update execution state: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl AuditRepository for SqliteRepository {
    async fn record_audit(
        &self,
        action: &str,
        actor: &str,
        resource: &str,
        details_json: &str,
    ) -> Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO audit_events (id, action, actor, resource, details_json, created_at) VALUES (?, ?, ?, ?, ?, ?)"
        )
        .bind(id)
        .bind(action)
        .bind(actor)
        .bind(resource)
        .bind(details_json)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to insert audit event: {e}")))?;

        Ok(())
    }

    async fn list_audits(&self, limit: usize) -> Result<Vec<AuditRecord>> {
        let rows: Vec<(String, String, String, String, String, String)> = sqlx::query_as(
            "SELECT id, action, actor, resource, details_json, created_at FROM audit_events ORDER BY created_at DESC LIMIT ?"
        )
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to list audit events: {e}")))?;

        let records = rows
            .into_iter()
            .map(
                |(id, action, actor, resource, details_json, created_at)| AuditRecord {
                    id,
                    action,
                    actor,
                    resource,
                    details_json,
                    created_at,
                },
            )
            .collect();

        Ok(records)
    }
}
