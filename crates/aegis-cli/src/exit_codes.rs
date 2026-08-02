// Chuẩn hóa Machine-Readable Exit Codes cho CLI `aegisctl`
// Giúp cho Shell Scripts và CI/CD Automation dễ dàng phân loại kết quả thực thi

use aegis_core::AegisError;

pub const SUCCESS: i32 = 0;
pub const GENERIC_FAILURE: i32 = 1;
pub const VALIDATION_FAILURE: i32 = 2;
pub const PERMISSION_DENIED: i32 = 3;
pub const NOT_FOUND: i32 = 4;
pub const CONFLICT: i32 = 5;
pub const TIMEOUT_OR_ROLLBACK: i32 = 6;

/// Chuyển đổi một AegisError thành mã Exit Code tương ứng
pub fn exit_code_for_error(err: &AegisError) -> i32 {
    match err {
        AegisError::Validation(_) => VALIDATION_FAILURE,
        AegisError::Permission(_) => PERMISSION_DENIED,
        AegisError::NotFound(_) => NOT_FOUND,
        AegisError::Conflict(_) => CONFLICT,
        AegisError::Timeout(_) => TIMEOUT_OR_ROLLBACK,
        AegisError::Firewall(msg) if msg.contains("roll") || msg.contains("Rollback") => {
            TIMEOUT_OR_ROLLBACK
        }
        _ => GENERIC_FAILURE,
    }
}
