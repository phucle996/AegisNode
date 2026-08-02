//! AegisNode Firewall Crate
//! Chứa Compilers, Runtime Engine, Transaction Safe Apply, Inspectors và Blocker Engine.

pub mod blocker;
pub mod compiler;
pub mod inspectors;
pub mod runtime;
pub mod transaction;

pub use blocker::{BlockManager, SshDetector};
pub use compiler::{CompiledFirewallPolicy, FirewallCompiler, NatCompiler, NftablesCompiler};
pub use inspectors::{
    DockerExposureReport, DockerInspector, ExposureWarning, RouterManager, SysctlSnapshot,
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
