//! AegisNode Audit Crate
//! Quản lý Lịch sử thao tác (Audit Log) và Chuỗi băm Kiểm toán mã hóa chống chỉnh sửa dữ liệu (Audit Hash Chain).

pub mod audit_chain;

pub use audit_chain::{AuditChainRecord, AuditChainVerifier, GENESIS_AUDIT_HASH};
