// Model định nghĩa hệ thống lỗi phân loại chuẩn cho AegisNode
// Đảm bảo tính nhất quán giữa API HTTP response, CLI exit code và Audit Event

use thiserror::Error;

/// Kết quả mặc định trả về từ các hàm trong AegisNode
pub type Result<T> = std::result::Result<T, AegisError>;

/// Danh mục lỗi được phân loại rõ ràng trong toàn bộ hệ thống AegisNode
#[derive(Debug, Error)]
pub enum AegisError {
    /// Lỗi validate định dạng cấu hình, policy hoặc dữ liệu đầu vào
    #[error("Validation error: {0}")]
    Validation(String),

    /// Lỗi cấu hình hệ thống hoặc thiếu file cấu hình
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Lỗi liên quan đến cơ sở dữ liệu (SQLite / PostgreSQL)
    #[error("Storage error: {0}")]
    Storage(String),

    /// Lỗi thao tác hoặc thực thi lệnh nftables/firewall
    #[error("Firewall error: {0}")]
    Firewall(String),

    /// Lỗi thiếu quyền hạn thực thi (VD: cần root/CAP_NET_ADMIN)
    #[error("Permission denied: {0}")]
    Permission(String),

    /// Lỗi không tìm thấy tài nguyên (Node, Policy, Rule, Container,...)
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Lỗi xung đột tài nguyên hoặc xung đột transaction đang apply
    #[error("Resource conflict: {0}")]
    Conflict(String),

    /// Lỗi hết thời gian chờ (Timeout) trong quá trình apply/rollback policy
    #[error("Operation timeout: {0}")]
    Timeout(String),

    /// Lỗi nội bộ hệ thống không lường trước
    #[error("Internal error: {0}")]
    Internal(String),
}

impl AegisError {
    /// Chuyển đổi lỗi sang mã lỗi CLI Exit Code tương ứng
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Validation(_) => 2,
            Self::Permission(_) => 3,
            Self::NotFound(_) => 4,
            Self::Conflict(_) => 5,
            Self::Timeout(_) => 6,
            Self::Configuration(_) | Self::Storage(_) | Self::Firewall(_) | Self::Internal(_) => 1,
        }
    }
}
