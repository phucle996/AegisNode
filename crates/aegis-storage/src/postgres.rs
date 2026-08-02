// PostgreSQL Repository Layer cho AegisNode Controller Server (`aegisnode server`)
// Phục vụ lưu trữ tập trung Multi-Node, chống Race Condition qua Optimistic Locking, mTLS Certificates & Node Inventories

use std::time::Duration;

use aegis_core::pki::AgentCertificateRecord;
use aegis_core::{AegisError, Result};
use aegis_models::inventory::NodeInventoryPayload;
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

    /// Cập nhật Heartbeat của Node theo Node ID
    pub async fn update_node_heartbeat(&self, node_id: Uuid, status: &str) -> Result<()> {
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

    /// Lưu Enrollment Token vào PostgreSQL
    pub async fn insert_enrollment_token(
        &self,
        token_hash: &str,
        max_usages: i32,
        ttl_minutes: i64,
    ) -> Result<Uuid> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO enrollment_tokens (id, token_hash, max_usages, current_usages, expires_at)
            VALUES ($1, $2, $3, 0, CURRENT_TIMESTAMP + ($4 || ' minutes')::INTERVAL)
            "#,
        )
        .bind(id)
        .bind(token_hash)
        .bind(max_usages)
        .bind(ttl_minutes.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to insert enrollment token: {e}")))?;

        Ok(id)
    }

    /// Tiêu thụ One-Time Enrollment Token (Atomic Usage Increment)
    pub async fn consume_enrollment_token(&self, token_hash: &str) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE enrollment_tokens
            SET current_usages = current_usages + 1
            WHERE token_hash = $1 AND revoked = FALSE AND expires_at > CURRENT_TIMESTAMP AND current_usages < max_usages
            RETURNING id
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to consume enrollment token: {e}")))?;

        Ok(result.is_some())
    }

    /// Lưu trữ Agent Certificate vào PostgreSQL
    pub async fn save_agent_certificate(&self, cert: &AgentCertificateRecord) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO agent_certificates (serial_number, node_id, machine_id, hostname, cert_pem, issued_at, expires_at, revoked)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (serial_number) DO UPDATE SET
                cert_pem = EXCLUDED.cert_pem,
                expires_at = EXCLUDED.expires_at,
                revoked = EXCLUDED.revoked
            "#,
        )
        .bind(&cert.serial_number)
        .bind(cert.node_id)
        .bind(&cert.machine_id)
        .bind(&cert.hostname)
        .bind(&cert.cert_pem)
        .bind(cert.issued_at)
        .bind(cert.expires_at)
        .bind(cert.revoked)
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to save agent certificate: {e}")))?;

        Ok(())
    }

    /// Cập nhật hoặc chèn mới Node Inventory (System, Network & Runtime) vào PostgreSQL
    pub async fn upsert_node_inventory(
        &self,
        node_id: Uuid,
        payload: &NodeInventoryPayload,
    ) -> Result<()> {
        let runtime_json = serde_json::to_value(&payload.runtime).map_err(|e| {
            AegisError::Storage(format!("Failed to serialize runtime inventory: {e}"))
        })?;

        sqlx::query(
            r#"
            INSERT INTO node_inventories (node_id, os_name, os_version, kernel_version, cpu_cores, total_memory_mb, free_memory_mb, uptime_seconds, machine_id, agent_version, runtime_summary, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, CURRENT_TIMESTAMP)
            ON CONFLICT (node_id) DO UPDATE SET
                os_name = EXCLUDED.os_name,
                os_version = EXCLUDED.os_version,
                kernel_version = EXCLUDED.kernel_version,
                cpu_cores = EXCLUDED.cpu_cores,
                total_memory_mb = EXCLUDED.total_memory_mb,
                free_memory_mb = EXCLUDED.free_memory_mb,
                uptime_seconds = EXCLUDED.uptime_seconds,
                machine_id = EXCLUDED.machine_id,
                agent_version = EXCLUDED.agent_version,
                runtime_summary = EXCLUDED.runtime_summary,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(node_id)
        .bind(&payload.system.os_name)
        .bind(&payload.system.os_version)
        .bind(&payload.system.kernel_version)
        .bind(payload.system.cpu_cores as i32)
        .bind(payload.system.total_memory_mb as i64)
        .bind(payload.system.free_memory_mb as i64)
        .bind(payload.system.uptime_seconds as i64)
        .bind(&payload.system.machine_id)
        .bind(&payload.system.agent_version)
        .bind(runtime_json)
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to upsert node inventory: {e}")))?;

        for iface in &payload.network_interfaces {
            sqlx::query(
                r#"
                INSERT INTO node_network_interfaces (node_id, interface_name, mac_address, mtu, operstate, ipv4_addresses, ipv6_addresses, rx_bytes, tx_bytes, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, CURRENT_TIMESTAMP)
                ON CONFLICT (node_id, interface_name) DO UPDATE SET
                    mac_address = EXCLUDED.mac_address,
                    mtu = EXCLUDED.mtu,
                    operstate = EXCLUDED.operstate,
                    ipv4_addresses = EXCLUDED.ipv4_addresses,
                    ipv6_addresses = EXCLUDED.ipv6_addresses,
                    rx_bytes = EXCLUDED.rx_bytes,
                    tx_bytes = EXCLUDED.tx_bytes,
                    updated_at = CURRENT_TIMESTAMP
                "#,
            )
            .bind(node_id)
            .bind(&iface.name)
            .bind(&iface.mac_address)
            .bind(iface.mtu as i32)
            .bind(&iface.operstate)
            .bind(&iface.ipv4_addresses)
            .bind(&iface.ipv6_addresses)
            .bind(iface.rx_bytes as i64)
            .bind(iface.tx_bytes as i64)
            .execute(&self.pool)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to upsert network interface: {e}")))?;
        }

        Ok(())
    }
}
