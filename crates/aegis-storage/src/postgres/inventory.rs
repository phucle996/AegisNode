// Quản lý System Inventory và Network Interfaces trong PostgreSQL

use aegis_core::{AegisError, Result};
use aegis_models::inventory::NodeInventoryPayload;
use uuid::Uuid;

use super::PgRepository;

impl PgRepository {
    /// Cập nhật hoặc chèn mới Node Inventory (System, Network & Runtime) vào PostgreSQL trong Transaction nguyên tử
    pub async fn upsert_node_inventory(
        &self,
        node_id: Uuid,
        payload: &NodeInventoryPayload,
    ) -> Result<()> {
        let runtime_json = serde_json::to_value(&payload.runtime).map_err(|e| {
            AegisError::Storage(format!("Failed to serialize runtime inventory: {e}"))
        })?;

        // Khởi tạo Database Transaction để thực hiện cập nhật nguyên tử cho System và Interfaces
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to begin inventory transaction: {e}")))?;

        // 1. Ghi thông tin System & Runtime Inventory vào bảng node_inventories
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
        .execute(&mut *tx)
        .await
        .map_err(|e| AegisError::Storage(format!("Failed to upsert node inventory: {e}")))?;

        // 2. Ghi danh sách Network Interfaces trong cùng transaction
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
            .execute(&mut *tx)
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to upsert network interface: {e}")))?;
        }

        // Commit transaction ghi nhận toàn bộ dữ liệu inventory nguyên tử
        tx.commit()
            .await
            .map_err(|e| AegisError::Storage(format!("Failed to commit inventory transaction: {e}")))?;

        Ok(())
    }
}
