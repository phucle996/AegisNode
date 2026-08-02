//! AegisNode Firewall & Network Engine Domain Models
//! Quản lý FirewallPolicy, NatPolicy, Advanced Network Profiles (Bonding, VRF, SynProxy).

pub mod advanced;
pub mod nat;
pub mod policy;

pub use advanced::{BondMode, BondingProfile, SynProxyConfig, VrfProfile};
pub use nat::{DnatRule, MasqueradeRule, NatMetadata, NatPolicy, PortForwardRule, SnatRule};
pub use policy::{
    CidrSpec, ConnectionState, FirewallAction, FirewallDefaults, FirewallDirection, FirewallPolicy,
    FirewallRule, InterfaceSelector, PolicyMetadata, PortSpec, SUPPORTED_API_VERSION,
    SUPPORTED_FIREWALL_KIND, TransportProtocol,
};
