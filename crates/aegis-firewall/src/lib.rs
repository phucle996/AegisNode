//! AegisNode Firewall Crate
//! Chứa Compiler dịch policy thành nftables ruleset và Runtime Backend tương tác với kernel.

pub mod backend;
pub mod capability;
pub mod compiler;
pub mod execution;
pub mod health_check;
pub mod nat;
pub mod nftables;
pub mod process_runner;
pub mod safe_apply;
pub mod snapshot;

pub use backend::{ApplyResult, FirewallBackend, FirewallState, NftablesRuntimeBackend};
pub use capability::{CapabilityDetector, NftCapabilityReport};
pub use compiler::{CompiledFirewallPolicy, FirewallCompiler};
pub use execution::{ApplyExecution, ExecutionState};
pub use health_check::{HealthCheckReport, HealthChecker};
pub use nat::NatCompiler;
pub use nftables::NftablesCompiler;
pub use process_runner::{
    DefaultProcessRunner, MockProcessRunner, ProcessOutput, ProcessRequest, ProcessRunner,
};
pub use safe_apply::SafeApplyManager;
pub use snapshot::{FirewallSnapshot, SnapshotManager};
