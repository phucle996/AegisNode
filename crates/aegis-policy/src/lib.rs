//! AegisNode Policy Engine Crate
//! Kiểm tra hợp lệ (Validation), phát hiện rủi ro bảo mật (Semantic checks),
//! chuẩn hóa (Normalizer), tính toán Hash deterministic và Phân quyền RBAC / 2-Person Approval.

pub mod hasher;
pub mod normalizer;
pub mod rbac_engine;
pub mod report;
pub mod validator;

pub use hasher::PolicyHasher;
pub use normalizer::PolicyNormalizer;
pub use rbac_engine::{ApprovalWorkflowValidator, RbacEngine};
pub use report::{ValidationIssue, ValidationReport, ValidationSeverity};
pub use validator::PolicyValidator;
