//! AegisNode Firewall Crate
//! Chứa Compilers, Runtime Engine, Transaction Safe Apply, Inspectors, Inventory Collector, Network Backend Detector, Systemd Manager, Combined Change Planner và Blocker Engine.

pub mod blocker;
pub mod change_planner;
pub mod compiler;
pub mod executor_client;
pub mod inspectors;
pub mod inventory_collector;
pub mod network_backend;
pub mod rollout_coordinator;
pub mod runtime;
pub mod systemd_manager;
pub mod transaction;

pub use blocker::{BlockManager, SshDetector};
pub use change_planner::{CombinedChangePlanner, assess_risk};
pub use compiler::{CompiledFirewallPolicy, FirewallCompiler, NatCompiler, NftablesCompiler};
pub use executor_client::{EXECD_SOCKET_PATH, ExecutorClient};
pub use inspectors::{
    DockerExposureReport, DockerInspector, ExposureWarning, RouterManager, SysctlSnapshot,
};
pub use inventory_collector::{
    collect_full_node_inventory, collect_network_interfaces, collect_system_inventory,
};
pub use network_backend::{NetworkBackendDetector, NetworkBackendReport, NetworkBackendType};
pub use rollout_coordinator::RolloutCoordinator;
pub use runtime::{
    ApplyResult, CapabilityDetector, DefaultProcessRunner, FirewallBackend, FirewallState,
    MockProcessRunner, NftCapabilityReport, NftablesRuntimeBackend, ProcessOutput, ProcessRequest,
    ProcessRunner,
};
pub use systemd_manager::{
    PROTECTED_SYSTEM_UNITS, SystemdManager, is_protected_unit, validate_unit_name,
};
pub use transaction::{
    ApplyExecution, ExecutionState, FirewallSnapshot, HealthCheckReport, HealthChecker,
    SafeApplyManager, SnapshotManager,
};
