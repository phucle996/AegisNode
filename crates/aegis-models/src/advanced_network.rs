//! Advanced Networking Data Models (Phase 24 Enterprise Bonding, VRF & SYN Flood)
//! Định nghĩa các cấu trúc BondingProfile, VrfProfile và SynProxyConfig.

use serde::{Deserialize, Serialize};

/// Chế độ Network Interface Bonding được hỗ trợ
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BondMode {
    /// Active-Backup (mode 1): Một slave chạy chính, slave còn lại dự phòng
    ActiveBackup,
    /// 802.3ad LACP (mode 4): Gộp băng thông chuẩn IEEE 802.3ad Link Aggregation
    Lacp8023ad,
}

/// Cấu hình Network Interface Bonding (BondingProfile)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondingProfile {
    /// Tên giao diện Bond (VD: bond0)
    pub bond_name: String,
    /// Chế độ Bonding
    pub mode: BondMode,
    /// Danh sách các giao diện card mạng thành viên (Slaves)
    pub slaves: Vec<String>,
    /// Tần số kiểm tra link MII (ms), mặc định 100ms
    pub miimon_ms: u32,
    /// Card mạng ưu tiên làm Primary slave trong chế độ ActiveBackup
    pub primary_slave: Option<String>,
}

/// Cấu hình Virtual Routing and Forwarding (VrfProfile)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VrfProfile {
    /// Tên của giao diện VRF (VD: vrf-prod)
    pub vrf_name: String,
    /// ID của bảng định tuyến Routing Table dành riêng cho VRF (VD: 100)
    pub table_id: u32,
    /// Danh sách giao diện card mạng được gắn vào VRF này
    pub interfaces: Vec<String>,
}

/// Cấu hình Chống Tấn công SYN Flood Protection (SynProxyConfig)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynProxyConfig {
    /// Maximum Segment Size (MSS)
    pub mss: u16,
    /// Window Scale factor
    pub wscale: u8,
    /// Giới hạn tần số SYN request per second per IP (VD: 100)
    pub syn_rate_limit: u32,
}
