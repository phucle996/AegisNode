//! AegisNode Fleet & Node Runtime Domain Models
//! Quản lý Node Inventory, Network Profiles, Systemd Units và Container Exposure.

pub mod docker;
pub mod inventory;
pub mod network_profile;
pub mod systemd;

pub use docker::{
    ContainerExposure, ContainerLabelPolicy, DockerContainer, DockerNetwork, PublishedPort,
};
pub use inventory::{
    NetworkInterfaceInfo, NodeInventoryPayload, RuntimeInventory, SystemInventory,
};
pub use network_profile::{
    AddressConfig, DnsConfig, InterfaceProfile, InterfaceRole, NetworkProfile, RouteConfig,
};
pub use systemd::{
    JournalLogEntry, ServiceOpRequest, ServiceOpResult, ServiceOperation, ServiceUnitStatus,
};
