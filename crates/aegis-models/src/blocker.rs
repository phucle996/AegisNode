// Domain Models cho Blocker, Block Entries, Allowlist và SSH Auto-Blocker
// Quản lý thông tin IP bị cấm (Temporary & Permanent) và chính sách dọn dẹp bộ nhớ

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::firewall::CidrSpec;

/// Bản ghi chi tiết một IP đang bị cấm bởi AegisNode Engine
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockEntry {
    pub ip: String,
    pub reason: String,
    pub actor: String,
    pub duration_seconds: Option<u64>,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl BlockEntry {
    /// Kiểm tra bản ghi block đã hết hạn hay chưa
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
    pub ssh_enabled: bool,
    pub threshold: u32,
    pub window_seconds: u64,
    pub block_seconds: u64,
    pub allowlist: Vec<CidrSpec>,
}

impl Default for BlockerConfig {
    fn default() -> Self {
        Self {
            ssh_enabled: true,
            threshold: 10,
            window_seconds: 60,
            block_seconds: 1800,
            allowlist: vec![
                CidrSpec("127.0.0.0/8".to_string()),
                CidrSpec("::1/128".to_string()),
                CidrSpec("10.0.0.0/8".to_string()),
                CidrSpec("172.16.0.0/12".to_string()),
                CidrSpec("192.168.0.0/16".to_string()),
            ],
        }
    }
}
