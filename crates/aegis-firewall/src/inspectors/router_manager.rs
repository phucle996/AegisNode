// Quản lý Router Mode & Sysctl IP Forwarding cho AegisNode
// Đọc/ghi /proc/sys/net/ipv4/ip_forward và /proc/sys/net/ipv6/conf/all/forwarding
// Tự động lưu SysctlSnapshot trước khi thay đổi và khôi phục khi Rollback

use aegis_core::{AegisError, Result};
use serde::{Deserialize, Serialize};

/// Bản chụp trạng thái giá trị Sysctl trước khi bật Router Mode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SysctlSnapshot {
    pub old_ipv4_forward: String,
    pub old_ipv6_forward: String,
}

/// Trình quản lý Router Mode & Sysctl Kernel Parameters
pub struct RouterManager;

impl RouterManager {
    pub fn new() -> Self {
        Self
    }

    /// Đọc giá trị IP Forwarding hiện tại của Linux Kernel
    pub async fn read_sysctl_forwarding() -> Result<SysctlSnapshot> {
        let v4_val = tokio::fs::read_to_string("/proc/sys/net/ipv4/ip_forward")
            .await
            .unwrap_or_else(|_| "0\n".to_string())
            .trim()
            .to_string();

        let v6_val = tokio::fs::read_to_string("/proc/sys/net/ipv6/conf/all/forwarding")
            .await
            .unwrap_or_else(|_| "0\n".to_string())
            .trim()
            .to_string();

        Ok(SysctlSnapshot {
            old_ipv4_forward: v4_val,
            old_ipv6_forward: v6_val,
        })
    }

    /// Thiết lập bật/tắt IP Forwarding trên Kernel và kiểm tra lỗi ghi sysctl an toàn
    pub async fn set_ip_forwarding(enable: bool) -> Result<SysctlSnapshot> {
        let snapshot = Self::read_sysctl_forwarding().await?;
        let target_val = if enable { "1" } else { "0" };

        // Ghi giá trị IPv4 forwarding vào kernel sysctl procfs
        tokio::fs::write("/proc/sys/net/ipv4/ip_forward", target_val)
            .await
            .map_err(|e| {
                AegisError::Firewall(format!("Không thể ghi sysctl IPv4 forwarding: {e}"))
            })?;

        // Ghi giá trị IPv6 forwarding vào kernel sysctl procfs
        tokio::fs::write("/proc/sys/net/ipv6/conf/all/forwarding", target_val)
            .await
            .map_err(|e| {
                AegisError::Firewall(format!("Không thể ghi sysctl IPv6 forwarding: {e}"))
            })?;

        Ok(snapshot)
    }

    /// Khôi phục lại giá trị Sysctl từ Snapshot khi Rollback
    pub async fn restore_sysctl(snapshot: &SysctlSnapshot) -> Result<()> {
        // Khôi phục lại giá trị IPv4 forwarding cũ
        let _ = tokio::fs::write("/proc/sys/net/ipv4/ip_forward", &snapshot.old_ipv4_forward).await;
        // Khôi phục lại giá trị IPv6 forwarding cũ
        let _ = tokio::fs::write(
            "/proc/sys/net/ipv6/conf/all/forwarding",
            &snapshot.old_ipv6_forward,
        )
        .await;
        Ok(())
    }
}

impl Default for RouterManager {
    fn default() -> Self {
        Self::new()
    }
}
