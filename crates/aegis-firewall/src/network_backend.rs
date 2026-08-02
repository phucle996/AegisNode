// Network Backend Detector & Adapter Engine cho AegisNode Linux Node
// Phát hiện tự động trình quản lý mạng (NetworkManager, systemd-networkd) và hỗ trợ chế độ Read-Only Fallback an toàn

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Trình quản lý mạng được phát hiện trên Linux Node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NetworkBackendType {
    /// Quản lý mạng qua NetworkManager (`nmcli` / D-Bus)
    NetworkManager,
    /// Quản lý mạng qua systemd-networkd (`/etc/systemd/network/*.network`)
    SystemdNetworkd,
    /// Chế độ Safe Fallback (Read-Only) khi gặp trình quản lý mạng lạ hoặc không có quyền root
    #[default]
    ReadOnly,
}

/// Trạng thái báo cáo của Network Backend Detector
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkBackendReport {
    pub backend_type: NetworkBackendType,
    pub is_writable: bool,
    pub detected_unit: String,
}

/// Engine nhận diện tự động trình quản lý mạng Linux
#[derive(Debug, Clone, Default)]
pub struct NetworkBackendDetector;

impl NetworkBackendDetector {
    pub fn new() -> Self {
        Self
    }

    /// Phát hiện trình quản lý mạng đang hoạt động trên HĐH Linux
    pub fn detect(&self) -> NetworkBackendReport {
        // 1. Kiểm tra NetworkManager active
        if Path::new("/run/NetworkManager/NetworkManager.pid").exists()
            || Path::new("/usr/bin/nmcli").exists()
        {
            return NetworkBackendReport {
                backend_type: NetworkBackendType::NetworkManager,
                is_writable: true,
                detected_unit: "NetworkManager.service".to_string(),
            };
        }

        // 2. Kiểm tra systemd-networkd active
        if Path::new("/run/systemd/netif/state").exists()
            || Path::new("/etc/systemd/network").exists()
        {
            return NetworkBackendReport {
                backend_type: NetworkBackendType::SystemdNetworkd,
                is_writable: true,
                detected_unit: "systemd-networkd.service".to_string(),
            };
        }

        // 3. Safe Fallback: Chuyển sang Read-Only để đảm bảo an toàn tuyệt đối
        NetworkBackendReport {
            backend_type: NetworkBackendType::ReadOnly,
            is_writable: false,
            detected_unit: "none (read-only mode)".to_string(),
        }
    }
}
