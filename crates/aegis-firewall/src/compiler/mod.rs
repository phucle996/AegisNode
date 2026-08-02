// Module Domain: Compiler (Biên dịch FirewallPolicy & NatPolicy sang nftables script)

pub mod nat;
pub mod nftables;

use aegis_core::Result;
use aegis_models::firewall::FirewallPolicy;
use chrono::{DateTime, Utc};
pub use nat::NatCompiler;
pub use nftables::NftablesCompiler;
use serde::{Deserialize, Serialize};

/// Kết quả đầu ra của quá trình Biên dịch Policy sang nftables Script
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompiledFirewallPolicy {
    /// Chuỗi kịch bản nftables (nft script) hoàn chỉnh sẵn sàng để apply
    pub nft_script: String,

    /// SHA-256 Hash đại diện cho nội dung Policy (từ PolicyHasher)
    pub policy_hash: String,

    /// Thời điểm biên dịch policy
    pub generated_at: DateTime<Utc>,

    /// Danh sách các Table nftables thuộc quyền quản lý của AegisNode
    pub managed_tables: Vec<String>,

    /// Danh sách các rule được tự động sinh (VD: loopback allow, established allow)
    pub auto_generated_rules: Vec<String>,
}

/// Trait định nghĩa giao diện Compiler cho Firewall Policy
pub trait FirewallCompiler: Send + Sync {
    /// Biên dịch một FirewallPolicy đã validate thành CompiledFirewallPolicy
    fn compile(&self, policy: &FirewallPolicy) -> Result<CompiledFirewallPolicy>;
}
