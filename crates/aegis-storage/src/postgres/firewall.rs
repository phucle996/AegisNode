// Quản lý bản ghi Luật Tường lửa OS Kernel thực tế gửi từ Agent Node vào PostgreSQL

use aegis_core::{AegisError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::PgRepository;

/// DTO biểu diễn 1 luật Firewall thực tế từ OS Kernel
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct LiveFirewallRuleRecord {
    pub id: Uuid,
    pub node_id: Uuid,
    pub chain: String,
    pub rule_id: String,
    pub protocol: String,
    pub src_cidr: String,
    pub dst_cidr: String,
    pub port_spec: String,
    pub action: String,
    pub packets: i64,
    pub bytes: i64,
    pub updated_at: DateTime<Utc>,
}

impl PgRepository {
    /// Upsert luật tường lửa thực tế thu thập từ OS Kernel vào PostgreSQL
    pub async fn upsert_node_firewall_rule(
        &self,
        node_id: Uuid,
        chain: &str,
        rule_id: &str,
        protocol: &str,
        src_cidr: &str,
        dst_cidr: &str,
        port_spec: &str,
        action: &str,
        packets: i64,
        bytes: i64,
    ) -> Result<()> {
        // Thực thi câu lệnh SQL Upsert nguyên tử cập nhật bộ đếm gói tin và dung lượng bytes
        sqlx::query(
            r#"
            INSERT INTO node_firewall_rules (node_id, chain, rule_id, protocol, src_cidr, dst_cidr, port_spec, action, packets, bytes, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, CURRENT_TIMESTAMP)
            ON CONFLICT (node_id, chain, rule_id) DO UPDATE SET
                protocol = EXCLUDED.protocol,
                src_cidr = EXCLUDED.src_cidr,
                dst_cidr = EXCLUDED.dst_cidr,
                port_spec = EXCLUDED.port_spec,
                action = EXCLUDED.action,
                packets = EXCLUDED.packets,
                bytes = EXCLUDED.bytes,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(node_id)
        .bind(chain)
        .bind(rule_id)
        .bind(protocol)
        .bind(src_cidr)
        .bind(dst_cidr)
        .bind(port_spec)
        .bind(action)
        .bind(packets)
        .bind(bytes)
        .execute(&self.pool)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to upsert node firewall rule: {e}")))?;

        Ok(())
    }

    /// Truy vấn danh sách luật tường lửa OS Kernel thực tế của 1 Node (hoặc toàn bộ Fleet)
    pub async fn list_live_firewall_rules(
        &self,
        node_id: Option<Uuid>,
    ) -> Result<Vec<LiveFirewallRuleRecord>> {
        // Nếu có node_id thì lọc theo node_id, ngược lại lấy toàn bộ
        let rows = if let Some(nid) = node_id {
            sqlx::query_as::<_, LiveFirewallRuleRecord>(
                r#"
                SELECT id, node_id, chain, rule_id, protocol, src_cidr, dst_cidr, port_spec, action, packets, bytes, updated_at
                FROM node_firewall_rules
                WHERE node_id = $1
                ORDER BY chain ASC, updated_at DESC
                "#,
            )
            .bind(nid)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, LiveFirewallRuleRecord>(
                r#"
                SELECT id, node_id, chain, rule_id, protocol, src_cidr, dst_cidr, port_spec, action, packets, bytes, updated_at
                FROM node_firewall_rules
                ORDER BY updated_at DESC
                "#,
            )
            .fetch_all(&self.pool)
            .await
        }
        .map_err(|e| AegisError::Storage(format!("Failed to list live firewall rules: {e}")))?;

        Ok(rows)
    }
}
