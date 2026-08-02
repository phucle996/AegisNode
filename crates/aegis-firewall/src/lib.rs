//! AegisNode Firewall Crate
//! Chứa Compiler dịch policy thành nftables ruleset và Runtime Backend tương tác với kernel.

pub mod backend;
pub mod capability;
pub mod compiler;
pub mod nat;
pub mod nftables;
pub mod process_runner;
pub mod snapshot;

pub use backend::{ApplyResult, FirewallBackend, FirewallState, NftablesRuntimeBackend};
pub use capability::{CapabilityDetector, NftCapabilityReport};
pub use compiler::{CompiledFirewallPolicy, FirewallCompiler};
pub use nat::NatCompiler;
pub use nftables::NftablesCompiler;
pub use process_runner::{
    DefaultProcessRunner, MockProcessRunner, ProcessOutput, ProcessRequest, ProcessRunner,
};
pub use snapshot::{FirewallSnapshot, SnapshotManager};
