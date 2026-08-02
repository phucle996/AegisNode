//! AegisNode Models Crate
//! Định nghĩa các domain objects core: FirewallPolicy, DockerContainer, BlockerEntry, Inventory, NetworkProfile...

pub mod blocker;
pub mod docker;
pub mod firewall;
pub mod inventory;
pub mod nat;
pub mod network_profile;

pub use blocker::{BlockEntry, BlockerConfig};
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
