//! Domain Models cho Blocker, Block Entries, Allowlist và SSH Auto-Blocker
//! Quản lý thông tin IP bị cấm (Temporary & Permanent) và chính sách dọn dẹp bộ nhớ.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::firewall::CidrSpec;

/// Bản ghi chi tiết một IP đang bị cấm bởi AegisNode Engine
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockEntry {
    /// Địa chỉ IP bị khóa
    pub ip: String,
    /// Lý do thực hiện khóa IP
    pub reason: String,
    /// Tác nhân yêu cầu khóa IP ("api", "ssh-detector", ...)
    pub actor: String,
    /// Thời hạn khóa tính bằng giây (None = vĩnh viễn)
    pub duration_seconds: Option<u64>,
    /// Thời điểm khởi tạo lệnh khóa
    pub created_at: DateTime<Utc>,
    /// Thời điểm hết hạn của lệnh khóa
    pub expires_at: Option<DateTime<Utc>>,
}

impl BlockEntry {
    /// Kiểm tra bản ghi block đã hết hạn hay chưa dựa trên thời gian hiện tại
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.expires_at {
            Utc::now() > exp
        } else {
            false
        }
    }
}

/// Cấu hình tổng thể cho Blocker & SSH Auto-Detector
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockerConfig {
    /// Bật/Tắt trình tự động phát hiện tấn công SSH Brute-force
    pub ssh_enabled: bool,
    /// Ngưỡng số lần đăng nhập thất bại tối đa trước khi khóa
    pub threshold: u32,
    /// Cửa sổ thời gian tính số lần thất bại (giây)
    pub window_seconds: u64,
    /// Thời gian khóa tự động mặc định (giây)
    pub block_seconds: u64,
    /// Danh sách CIDR được phép (Allowlist), không bao giờ bị khóa
    pub allowlist: Vec<CidrSpec>,
}

impl Default for BlockerConfig {
    fn default() -> Self {
        Self {
            ssh_enabled: true,
            threshold: 10,
            window_seconds: 60,
            block_seconds: 1800,
            // Mặc định chỉ cho phép Loopback, KHÔNG hardcode các dải IP nội bộ
            allowlist: vec![
                CidrSpec("127.0.0.0/8".to_string()),
                CidrSpec("::1/128".to_string()),
            ],
        }
    }
}
