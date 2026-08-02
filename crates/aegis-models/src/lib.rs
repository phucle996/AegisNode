//! AegisNode Models Crate
//! Định nghĩa các domain objects core: FirewallPolicy, DockerContainer, BlockerEntry, Inventory, NetworkProfile, Systemd, ChangePlan...

pub mod blocker;
pub mod change_plan;
pub mod docker;
pub mod firewall;
pub mod inventory;
pub mod nat;
pub mod network_profile;
pub mod rbac;
pub mod systemd;

pub use rbac::{AccessScope, ApprovalRecord, Permission, Role, UserSubject};

pub use blocker::{BlockEntry, BlockerConfig};
pub use change_plan::{
    BatchConfig, ExecutionStep, HealthCheckSpec, NodeChangePlan, NodeRolloutState,
    NodeRolloutStatus, RiskLevel, RolloutReport, RolloutSpec, RolloutStrategy, StepStatus,
};
pub use docker::{
    ContainerExposure, ContainerLabelPolicy, DockerContainer, DockerNetwork, PublishedPort,
};
pub use firewall::{
    CidrSpec, FirewallDefaults, FirewallDirection, FirewallPolicy, FirewallRule, PortSpec,
    TransportProtocol,
};
pub use inventory::{
    NetworkInterfaceInfo, NodeInventoryPayload, RuntimeInventory, SystemInventory,
};
pub use nat::NatPolicy;
pub use network_profile::{
    AddressConfig, DnsConfig, InterfaceProfile, InterfaceRole, NetworkProfile, RouteConfig,
};
pub use systemd::{
    JournalLogEntry, ServiceOpRequest, ServiceOpResult, ServiceOperation, ServiceUnitStatus,
};
