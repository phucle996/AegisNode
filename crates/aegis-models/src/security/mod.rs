//! AegisNode Security & Access Control Domain Models
//! Tập hợp các models về Blocker (Dynamic IP Sets), RBAC Roles/Permissions và Signed Policy Bundles.

pub mod blocker;
pub mod bundle;
pub mod rbac;

pub use blocker::{BlockEntry, BlockerConfig};
pub use bundle::SignedPolicyBundle;
pub use rbac::{AccessScope, ApprovalRecord, Permission, Role, UserSubject};
