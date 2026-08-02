// Module Domain: Runtime (Tương tác trực tiếp nhân Kernel, Process Runner & System Capability Detection)

pub mod backend;
pub mod capability;
pub mod process_runner;

pub use backend::{ApplyResult, FirewallBackend, FirewallState, NftablesRuntimeBackend};
pub use capability::{CapabilityDetector, NftCapabilityReport};
pub use process_runner::{
    DefaultProcessRunner, MockProcessRunner, ProcessOutput, ProcessRequest, ProcessRunner,
};
