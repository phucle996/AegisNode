//! AegisNode Firewall Crate
//! Chứa Compiler dịch policy thành nftables ruleset, Runtime Backend, Blocker Engine và SSH Detector.

pub mod backend;
pub mod block_manager;
pub mod capability;
pub mod compiler;
pub mod docker_inspector;
pub mod execution;
pub mod health_check;
pub mod nat;
pub mod nftables;
pub mod process_runner;
pub mod router_manager;
pub mod safe_apply;
pub mod snapshot;
pub mod ssh_detector;

pub use backend::{ApplyResult, FirewallBackend, FirewallState, NftablesRuntimeBackend};
pub use block_manager::BlockManager;
pub use capability::{CapabilityDetector, NftCapabilityReport};
pub use compiler::{CompiledFirewallPolicy, FirewallCompiler};
pub use docker_inspector::{DockerExposureReport, DockerInspector, ExposureWarning};
pub use execution::{ApplyExecution, ExecutionState};
pub use health_check::{HealthCheckReport, HealthChecker};
pub use nat::NatCompiler;
pub use nftables::NftablesCompiler;
pub use process_runner::{
    DefaultProcessRunner, MockProcessRunner, ProcessOutput, ProcessRequest, ProcessRunner,
};
pub use router_manager::{RouterManager, SysctlSnapshot};
pub use safe_apply::SafeApplyManager;
pub use snapshot::{FirewallSnapshot, SnapshotManager};
pub use ssh_detector::SshDetector;
