//! AegisNode Core Crate
//! Cung cấp các kiểu dữ liệu dùng chung (Identifiers), Định nghĩa lỗi chuẩn (Error model),
//! PKI / X.509 Certificate helpers và các tiện ích cơ bản cho hệ thống AegisNode.

pub mod error;
pub mod identifiers;
pub mod pki;

pub use error::{AegisError, Result};
pub use identifiers::*;
pub use pki::{AgentCertificateRecord, EnrollmentToken, PkiManager};
