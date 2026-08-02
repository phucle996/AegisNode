// Node Inventory Models DTO cho AegisNode Stage 2
// Định nghĩa cấu trúc dữ liệu cho System, Network và Runtime Inventory của các Linux Node

use serde::{Deserialize, Serialize};

/// Thông tin System Inventory (Hệ thống & Phần cứng)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SystemInventory {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_cores: u32,
    pub total_memory_mb: u64,
    pub free_memory_mb: u64,
    pub uptime_seconds: u64,
    pub machine_id: String,
    pub agent_version: String,
}

/// Thông tin giao diện mạng (Network Interface Inventory)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceInfo {
    pub name: String,
    pub mac_address: String,
    pub permanent_mac: String,
    pub mtu: u32,
    pub operstate: String,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub gateway: Option<String>,
}

/// Thông tin Runtime Inventory (Trạng thái Firewall, Docker & Services)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInventory {
    pub firewall_hash: String,
    pub rule_count: u32,
    pub active_blocks_count: u32,
    pub docker_containers_count: u32,
    pub systemd_services_running: u32,
    pub backend_name: String,
}

/// Payload tổng hợp thông tin Inventory gửi từ Linux Agent Node lên Controller
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NodeInventoryPayload {
    pub system: SystemInventory,
    pub network_interfaces: Vec<NetworkInterfaceInfo>,
    pub runtime: RuntimeInventory,
}
