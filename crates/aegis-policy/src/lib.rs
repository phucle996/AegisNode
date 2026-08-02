//! AegisNode Policy Engine Crate
//! Kiểm tra hợp lệ (Validation), phát hiện rủi ro bảo mật (Semantic checks),
//! chuẩn hóa (Normalizer) và tính toán Hash deterministic (Hasher).

pub mod hasher;
pub mod normalizer;
pub mod report;
pub mod validator;

pub use hasher::PolicyHasher;
pub use normalizer::PolicyNormalizer;
pub use report::{ValidationIssue, ValidationReport, ValidationSeverity};
pub use validator::PolicyValidator;
