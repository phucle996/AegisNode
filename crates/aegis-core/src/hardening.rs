//! Security Hardening & File Permission Guards (Phase 27 Production Hardening)
//! Cung cấp các phương thức kiểm tra giới hạn Payload size cap và quyền tệp tin an toàn.

use crate::error::AegisError;
use std::path::Path;

/// Giới hạn dung lượng tối đa cho Payload API (10 MegaBytes)
pub const MAX_API_PAYLOAD_SIZE_BYTES: usize = 10 * 1024 * 1024;

/// Bộ Quản lý Gia cố Bảo mật (SecurityHardening)
pub struct SecurityHardening;

impl SecurityHardening {
    /// Kiểm tra giới hạn dung lượng Payload nhận được (Payload Size Cap)
    pub fn validate_payload_size(payload_bytes_len: usize) -> Result<(), AegisError> {
        if payload_bytes_len > MAX_API_PAYLOAD_SIZE_BYTES {
            return Err(AegisError::Validation(format!(
                "Dung lượng Payload ({payload_bytes_len} bytes) vượt quá giới hạn an toàn cho phép ({MAX_API_PAYLOAD_SIZE_BYTES} bytes)"
            )));
        }
        Ok(())
    }

    /// Kiểm tra quyền tệp tin trên hệ thống Linux (File Permissions Guard)
    pub fn validate_file_permissions(path: &Path, expected_mode: u32) -> Result<(), AegisError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let metadata = std::fs::metadata(path).map_err(|e| {
                AegisError::Internal(format!("Lỗi đọc thông tin file metadata '{:?}': {e}", path))
            })?;

            let current_mode = metadata.permissions().mode() & 0o777;

            if current_mode != expected_mode {
                return Err(AegisError::Permission(format!(
                    "Quyền tệp tin '{:?}' không đạt tiêu chuẩn an toàn (Hiện tại: {:o}, Kỳ vọng: {:o})",
                    path, current_mode, expected_mode
                )));
            }
        }

        let _ = path;
        let _ = expected_mode;
        Ok(())
    }
}
