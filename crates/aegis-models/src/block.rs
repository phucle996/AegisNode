// Model quản lý danh sách địa chỉ IP bị chặn (Blocklist)
// Phục vụ cơ chế chặn thủ công (Manual Block) và chặn tự động (SSH/Nginx Brute-force detector)

use aegis_core::BlockEntryId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Một bản ghi IP bị chặn trong hệ thống AegisNode
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockEntry {
    #[serde(default = "BlockEntryId::new_v4")]
    pub id: BlockEntryId,
    pub ip: String,
    pub source: BlockSource,
    pub reason: BlockReason,
    pub duration: BlockDuration,
    pub status: BlockStatus,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Nguồn phát hiện/yêu cầu chặn IP
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlockSource {
    Manual,
    SshDetector,
    NginxDetector,
    Api,
}

/// Lý do chặn IP
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlockReason {
    BruteForce,
    ManualAdmin,
    MaliciousActivity,
    Other(String),
}

/// Thời hạn hiệu lực của lệnh chặn (Giây hoặc Vĩnh viễn)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlockDuration {
    /// Chặn tạm thời theo số giây quy định
    Temporary(u64),
    /// Chặn vĩnh viễn
    Permanent,
}

/// Trạng thái hiện tại của bản ghi chặn IP
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BlockStatus {
    Active,
    Expired,
    Revoked,
}
