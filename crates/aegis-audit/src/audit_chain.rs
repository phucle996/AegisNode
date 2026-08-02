//! Cryptographic Audit Trail Hash Chain & Integrity Verification (Phase 25 Audit & Recovery)
//! Cung cấp cấu trúc AuditChainRecord dạng Merkle-linked list và bộ kiểm tra toàn vẹn chống sửa đổi dữ liệu (Tamper Detection).

use aegis_core::AegisError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Hằng số khởi tạo mã băm cho bản ghi Audit đầu tiên (Genesis Hash)
pub const GENESIS_AUDIT_HASH: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

/// Bản ghi Nhật ký Kiểm toán liên kết chuỗi băm (AuditChainRecord)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditChainRecord {
    /// ID duy nhất của bản ghi Audit (UUID v4)
    pub id: String,

    /// Định danh của đối tác thực hiện (User ID / Agent ID)
    pub actor_id: String,

    /// Hành động thực hiện (VD: POLICY_APPLY, ROLLBACK, RBAC_APPROVE)
    pub action: String,

    /// Tài nguyên mục tiêu (VD: policy:policy-web, node:node-01)
    pub resource: String,

    /// ID của Node xảy ra sự kiện (tùy chọn)
    pub node_id: Option<String>,

    /// ID của Execution Step (tùy chọn)
    pub execution_id: Option<String>,

    /// Mã băm SHA-256 của tài nguyên trước khi thay đổi
    pub before_hash: Option<String>,

    /// Mã băm SHA-256 của tài nguyên sau khi thay đổi
    pub after_hash: Option<String>,

    /// Kết quả thực hiện (SUCCESS / FAILURE / DENIED)
    pub result: String,

    /// Thời điểm ghi nhận sự kiện (ISO 8601 string)
    pub timestamp: String,

    /// Số thứ tự liên tục trong chuỗi Audit (1-indexed)
    pub sequence_number: u64,

    /// Mã băm event_hash của bản ghi sự kiện ngay trước đó (Merkle Link)
    pub prev_event_hash: String,

    /// Mã băm cryptographic hash đại diện cho bản ghi sự kiện hiện tại
    pub event_hash: String,
}

impl AuditChainRecord {
    /// Tính toán mã băm SHA-256 duy nhất cho bản ghi sự kiện hiện tại
    pub fn compute_event_hash(&self) -> String {
        let mut hasher = Sha256::new();

        let canonical_str = format!(
            "{}:{}:{}:{}:{}:{}:{}:{}:{}",
            self.sequence_number,
            self.prev_event_hash,
            self.id,
            self.actor_id,
            self.action,
            self.resource,
            self.result,
            self.timestamp,
            self.after_hash.as_deref().unwrap_or("")
        );

        hasher.update(canonical_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Bộ Kiểm tra Tính Toàn vẹn Chuỗi băm Audit (AuditChainVerifier)
pub struct AuditChainVerifier;

impl AuditChainVerifier {
    /// Xác thực toàn bộ chuỗi nhật ký kiểm toán (Audit Trail)
    /// Trả về Ok(()) nếu hợp lệ hoặc Err(AegisError::Validation) nếu phát hiện dữ liệu bị sửa đổi (Tampered)
    pub fn verify_chain_integrity(records: &[AuditChainRecord]) -> Result<(), AegisError> {
        if records.is_empty() {
            return Ok(());
        }

        let mut expected_prev_hash = GENESIS_AUDIT_HASH.to_string();

        for (index, record) in records.iter().enumerate() {
            // 1. Kiểm tra liên kết băm prev_event_hash với bản ghi trước đó
            if record.prev_event_hash != expected_prev_hash {
                return Err(AegisError::Validation(format!(
                    "Phát hiện gián đoạn chuỗi Audit tại bản ghi index {} (ID: {}): prev_event_hash không khớp (Nhận: {}, Kỳ vọng: {})",
                    index, record.id, record.prev_event_hash, expected_prev_hash
                )));
            }

            // 2. Tính toán lại event_hash và so sánh với event_hash lưu trữ
            let computed_hash = record.compute_event_hash();
            if record.event_hash != computed_hash {
                return Err(AegisError::Validation(format!(
                    "Phát hiện dữ liệu Audit bị chỉnh sửa (Tampered) tại bản ghi index {} (ID: {}): event_hash không khớp (Nhận: {}, Tính toán: {})",
                    index, record.id, record.event_hash, computed_hash
                )));
            }

            // Cập nhật expected_prev_hash cho vòng lặp tiếp theo
            expected_prev_hash = record.event_hash.clone();
        }

        Ok(())
    }
}
