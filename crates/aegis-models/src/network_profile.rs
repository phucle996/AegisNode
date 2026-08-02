// Network Management Domain Models cho AegisNode Stage 2
// Định nghĩa vai trò giao diện mạng (WAN, LAN, MANAGEMENT, STORAGE) và cấu hình Network Profile

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Vai trò đại diện của Giao diện mạng (Interface Role cho Microsegmentation)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InterfaceRole {
    /// Card mạng kết nối Internet công cộng (WAN)
    Wan,
    /// Card mạng kết nối nội bộ (LAN)
    Lan,
    /// Kênh quản trị và điều khiển mTLS với Controller (MANAGEMENT)
    Management,
    /// Kênh truyền tải dữ liệu lưu trữ / Replication (STORAGE)
    Storage,
    /// Chưa gắn vai trò cụ thể
    #[default]
    Unspecified,
}

/// Cấu hình địa chỉ IP (DHCP hoặc Static IPv4/IPv6)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AddressConfig {
    pub dhcp: bool,
    pub ipv4_cidr: Option<String>,
    pub ipv6_cidr: Option<String>,
}

/// Cấu hình tuyến đường Static Route
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RouteConfig {
    pub destination_cidr: String,
    pub gateway: String,
    pub metric: u32,
}

/// Cấu hình hệ thống phân giải tên miền (DNS)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DnsConfig {
    pub nameservers: Vec<String>,
    pub search_domains: Vec<String>,
}

/// Profile cấu hình cho từng giao diện mạng cụ thể
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InterfaceProfile {
    pub name: String,
    pub role: InterfaceRole,
    pub address: AddressConfig,
    pub routes: Vec<RouteConfig>,
    pub mtu: u32,
}

/// Profile cấu hình mạng tổng thể của Node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkProfile {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub interfaces: Vec<InterfaceProfile>,
    pub dns: DnsConfig,
}

impl Default for NetworkProfile {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "default_profile".to_string(),
            description: "Default Network Profile".to_string(),
            interfaces: vec![],
            dns: DnsConfig::default(),
        }
    }
}
