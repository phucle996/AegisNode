// Policy Hasher tính toán SHA-256 Hash chuẩn hóa (Deterministic Policy Hash)
// Dùng để phát hiện thay đổi Policy (Drift Detection), Snapshot Metadata và Reconciliation

use aegis_models::firewall::FirewallPolicy;
use sha2::{Digest, Sha256};

use crate::normalizer::PolicyNormalizer;

/// Struct Hasher chịu trách nhiệm sinh Deterministic SHA-256 Hash
pub struct PolicyHasher;

impl PolicyHasher {
    /// Chuẩn hóa Policy và tính toán SHA-256 Hash định dạng Hex (64 ký tự)
    pub fn compute_hash(policy: &FirewallPolicy) -> String {
        // 1. Đưa Policy về dạng Normalized Canonical Form
        let normalized = PolicyNormalizer::normalize(policy);

        // 2. Serialize sang JSON string chuẩn để băm
        let json_canonical =
            serde_json::to_string(&normalized).unwrap_or_else(|_| format!("{policy:?}"));

        // 3. Tính toán SHA-256 Digest
        let mut hasher = Sha256::new();
        hasher.update(json_canonical.as_bytes());
        let result = hasher.finalize();

        // 4. Trả về Hex string
        format!("{:x}", result)
    }
}
