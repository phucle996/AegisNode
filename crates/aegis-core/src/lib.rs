//! AegisNode Core Crate
//! Cung cấp các kiểu dữ liệu dùng chung (Identifiers), Định nghĩa lỗi chuẩn (Error model)
//! và các tiện ích cơ bản cho toàn bộ các module trong hệ thống.

pub mod error;
pub mod identifiers;

pub use error::{AegisError, Result};
pub use identifiers::*;
