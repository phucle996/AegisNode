//! AegisNode Models Crate
//! Định nghĩa toàn bộ Domain Objects Core được phân thành 4 miền chính: Security, Firewall, Fleet và Rollout.

pub mod firewall;
pub mod fleet;
pub mod rollout;
pub mod security;

// Re-exports 100% backward compatibility cho toàn hệ thống (Flat types)
pub use firewall::*;
pub use fleet::*;
pub use rollout::*;
pub use security::*;

// Module aliases 100% backward compatibility cho legacy module paths
pub use firewall::advanced as advanced_network;
pub use firewall::nat;
pub use firewall::policy as firewall_policy;
pub use fleet::docker;
pub use fleet::inventory;
pub use fleet::network_profile;
pub use fleet::systemd;
pub use rollout::change_plan;
pub use security::blocker;
pub use security::bundle;
pub use security::rbac;
