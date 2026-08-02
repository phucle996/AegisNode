//! AegisNode Firewall Crate
//! Chứa Compilers, Runtime Engine, Transaction Safe Apply, Inspectors, Inventory Collector và Blocker Engine.

pub mod blocker;
pub mod compiler;
pub mod inspectors;
pub mod inventory_collector;
pub mod runtime;
pub mod transaction;

pub use blocker::{BlockManager, SshDetector};
pub use compiler::{CompiledFirewallPolicy, FirewallCompiler, NatCompiler, NftablesCompiler};
pub use inspectors::{
    DockerExposureReport, DockerInspector, ExposureWarning, RouterManager, SysctlSnapshot,
};
pub use inventory_collector::{
    collect_full_node_inventory, collect_network_interfaces, collect_system_inventory,
};
pub use runtime::{
    ApplyResult, CapabilityDetector, DefaultProcessRunner, FirewallBackend, FirewallState,
    MockProcessRunner, NftCapabilityReport, NftablesRuntimeBackend, ProcessOutput, ProcessRequest,
    ProcessRunner,
};
pub use transaction::{
    ApplyExecution, ExecutionState, FirewallSnapshot, HealthCheckReport, HealthChecker,
    SafeApplyManager, SnapshotManager,
};
