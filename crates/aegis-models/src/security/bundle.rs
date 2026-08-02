//! Signed Policy Bundle Data Models (Phase 22 Cryptographic Integrity & Anti-Replay)
//! Định nghĩa cấu trúc SignedPolicyBundle chứa Chữ ký số Ed25519, SHA-256 Checksum, Target Node ID và Sequence Number.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use crate::firewall::FirewallPolicy;
use crate::fleet::NetworkProfile;

/// Gói Policy đã được đóng gói và ký số bởi Controller (SignedPolicyBundle)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedPolicyBundle {
    /// ID duy nhất của đợt Bundle (UUID v4)
    pub bundle_id: String,

    /// ID của Node mục tiêu được phép áp dụng Policy (Khóa Target Node)
    pub target_node_id: String,

    /// Phiên bản Policy (VD: v1.4.2)
    pub policy_version: String,

    /// Số thứ tự Sequence đơn điệu tăng dần (Chống Replay Attack)
    pub sequence_number: u64,

    /// Thời điểm Controller phát hành Bundle (ISO 8601 string)
    pub issued_at: String,

    /// Thời điểm hết hạn của Bundle (ISO 8601 string)
    pub expires_at: String,

    /// Định danh của Controller phát hành
    pub controller_id: String,

    /// Mã băm SHA-256 checksum của toàn bộ nội dung Payload
    pub payload_checksum: String,

    /// Cấu hình Firewall Policy (tùy chọn)
    pub firewall_policy: Option<FirewallPolicy>,

    /// Cấu hình Network Profile (tùy chọn)
    pub network_profile: Option<NetworkProfile>,

    /// Chuỗi hex biểu diễn chữ ký số Ed25519 (64-byte signature)
    pub signature_hex: String,
}

impl SignedPolicyBundle {
    /// Tính toán mã băm SHA-256 của nội dung Payload (FirewallPolicy + NetworkProfile)
    pub fn compute_payload_checksum(&self) -> String {
        let mut hasher = Sha256::new();

        // 1. Hash phần Firewall Policy nếu có
        if let Some(ref fw) = self.firewall_policy {
            if let Ok(fw_json) = serde_json::to_string(fw) {
                hasher.update(fw_json.as_bytes());
            }
        }

        // 2. Hash phần Network Profile nếu có
        if let Some(ref net) = self.network_profile {
            if let Ok(net_json) = serde_json::to_string(net) {
                hasher.update(net_json.as_bytes());
            }
        }

        // 3. Trả về mã SHA-256 dưới dạng chuỗi Hexadecimal
        format!("{:x}", hasher.finalize())
    }

    /// Trích xuất chuỗi Byte chuẩn hóa cần ký số / xác thực (Signing Data Buffer)
    pub fn signing_bytes(&self) -> Vec<u8> {
        let canonical_str = format!(
            "{}:{}:{}:{}:{}:{}:{}",
            self.bundle_id,
            self.target_node_id,
            self.policy_version,
            self.sequence_number,
            self.issued_at,
            self.expires_at,
            self.payload_checksum
        );
        canonical_str.into_bytes()
    }
}
