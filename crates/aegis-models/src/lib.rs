//! AegisNode Models Crate
//! Định nghĩa các domain objects core: FirewallPolicy, DockerContainer, BlockerEntry, NatPolicy...

pub mod blocker;
pub mod docker;
pub mod firewall;
pub mod nat;

pub use blocker::{BlockEntry, BlockerConfig};
pub use docker::{
    ContainerExposure, ContainerLabelPolicy, DockerContainer, DockerNetwork, PublishedPort,
};
pub use firewall::{
    CidrSpec, FirewallDefaults, FirewallDirection, FirewallPolicy, FirewallRule, PortSpec,
    TransportProtocol,
};
pub use nat::NatPolicy;
