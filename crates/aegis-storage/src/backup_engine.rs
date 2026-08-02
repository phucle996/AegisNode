//! Automated Backup Export & Disaster Recovery Engine (Phase 25 Audit & Recovery)
//! Đóng gói bản sao lưu mã hóa BackupSnapshot và kiểm tra tính toàn vẹn Checksum trước khi thực hiện Phục hồi sau thảm họa.

use aegis_core::AegisError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Phiên bản Backup Schema được hỗ trợ hiện tại
pub const CURRENT_BACKUP_SCHEMA_VERSION: &str = "v1.0.0";

/// Cấu trúc Bản sao lưu Hệ thống (BackupSnapshot)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSnapshot {
    /// Phiên bản Schema của bản sao lưu
    pub version: String,

    /// Thời điểm khởi tạo bản sao lưu (ISO 8601 string)
    pub created_at: String,

    /// Chuỗi mã băm SHA-256 Checksum đại diện cho toàn bộ nội dung dữ liệu
    pub checksum: String,

    /// Dữ liệu JSON mã hóa danh sách Firewall Policies
    pub policies_json: String,

    /// Dữ liệu JSON mã hóa danh sách Nodes
    pub nodes_json: String,

    /// Dữ liệu JSON mã hóa danh sách Audit Logs
    pub audit_logs_json: String,
}

impl BackupSnapshot {
    /// Tính toán mã băm SHA-256 Checksum cho nội dung BackupSnapshot
    pub fn compute_checksum(&self) -> String {
        let mut hasher = Sha256::new();
        let canonical_str = format!(
            "{}:{}:{}:{}",
            self.version, self.policies_json, self.nodes_json, self.audit_logs_json
        );
        hasher.update(canonical_str.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Động cơ Quản lý Sao lưu & Phục hồi (BackupEngine)
pub struct BackupEngine;

impl BackupEngine {
    /// Tạo một bản sao lưu BackupSnapshot mới từ dữ liệu JSON
    pub fn create_backup(
        policies_json: String,
        nodes_json: String,
        audit_logs_json: String,
        created_at: String,
    ) -> BackupSnapshot {
        let mut snapshot = BackupSnapshot {
            version: CURRENT_BACKUP_SCHEMA_VERSION.to_string(),
            created_at,
            checksum: "".to_string(),
            policies_json,
            nodes_json,
            audit_logs_json,
        };

        snapshot.checksum = snapshot.compute_checksum();
        snapshot
    }

    /// Xác thực tính hợp lệ của Bản sao lưu trước khi tiến hành Phục hồi hệ thống
    pub fn verify_backup(snapshot: &BackupSnapshot) -> Result<(), AegisError> {
        // 1. Kiểm tra Schema Version
        if snapshot.version != CURRENT_BACKUP_SCHEMA_VERSION {
            return Err(AegisError::Validation(format!(
                "Phiên bản Backup Schema không tương thích (Nhận: {}, Kỳ vọng: {})",
                snapshot.version, CURRENT_BACKUP_SCHEMA_VERSION
            )));
        }

        // 2. Tính toán lại SHA-256 Checksum và so sánh với Checksum lưu trữ
        let expected_checksum = snapshot.compute_checksum();
        if snapshot.checksum != expected_checksum {
            return Err(AegisError::Validation(format!(
                "Bản sao lưu BackupSnapshot bị hư hỏng hoặc chỉnh sửa dữ liệu (Checksum mismatch! Nhận: {}, Tính toán: {})",
                snapshot.checksum, expected_checksum
            )));
        }

        Ok(())
    }
}
