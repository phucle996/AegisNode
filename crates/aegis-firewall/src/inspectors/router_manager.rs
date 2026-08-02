// Quản lý Router Mode & Sysctl IP Forwarding cho AegisNode
// Đọc/ghi /proc/sys/net/ipv4/ip_forward và /proc/sys/net/ipv6/conf/all/forwarding
// Tự động lưu SysctlSnapshot trước khi thay đổi và khôi phục khi Rollback

use aegis_core::Result;
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

    /// Thiết lập bật/tắt IP Forwarding trên Kernel
    pub async fn set_ip_forwarding(enable: bool) -> Result<SysctlSnapshot> {
        let snapshot = Self::read_sysctl_forwarding().await?;
        let target_val = if enable { "1" } else { "0" };

        let _ = tokio::fs::write("/proc/sys/net/ipv4/ip_forward", target_val).await;
        let _ = tokio::fs::write("/proc/sys/net/ipv6/conf/all/forwarding", target_val).await;

        Ok(snapshot)
    }

    /// Khôi phục lại giá trị Sysctl từ Snapshot khi Rollback
    pub async fn restore_sysctl(snapshot: &SysctlSnapshot) -> Result<()> {
        let _ = tokio::fs::write("/proc/sys/net/ipv4/ip_forward", &snapshot.old_ipv4_forward).await;
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
