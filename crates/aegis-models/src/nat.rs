// Model định nghĩa Network Address Translation (NAT)
// Hỗ trợ Masquerade, SNAT, DNAT và Port Forwarding trên nftables

use aegis_core::{PolicyId, RuleId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::firewall::{CidrSpec, PortSpec, TransportProtocol};

/// Model chính cho NAT Policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NatPolicy {
    pub api_version: String,
    pub kind: String,
    pub metadata: NatMetadata,
    #[serde(default)]
    pub masquerade_rules: Vec<MasqueradeRule>,
    #[serde(default)]
    pub snat_rules: Vec<SnatRule>,
    #[serde(default)]
    pub dnat_rules: Vec<DnatRule>,
    #[serde(default)]
    pub port_forward_rules: Vec<PortForwardRule>,
}

/// Metadata cho NAT Policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NatMetadata {
    pub name: String,
    #[serde(default = "PolicyId::new_v4")]
    pub id: PolicyId,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

/// Rule Masquerade (Ẩn địa chỉ IP LAN đằng sau IP WAN động)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MasqueradeRule {
    pub id: RuleId,
    pub out_interface: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_cidr: Option<CidrSpec>,
}

/// Rule Source NAT (Sửa địa chỉ IP nguồn thành IP cố định)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnatRule {
    pub id: RuleId,
    pub out_interface: String,
    pub to_source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_cidr: Option<CidrSpec>,
}

/// Rule Destination NAT (Chuyển hướng IP/Port đích)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DnatRule {
    pub id: RuleId,
    pub in_interface: String,
    pub protocol: TransportProtocol,
    pub external_port: PortSpec,
    pub to_destination: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_port: Option<u16>,
}

/// Rule Port Forwarding chi tiết (Chuyển tiếp cổng ra máy nội bộ/container)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortForwardRule {
    pub id: RuleId,
    pub in_interface: String,
    pub protocol: TransportProtocol,
    pub external_port: PortSpec,
    pub destination_address: String,
    pub destination_port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_cidr: Option<CidrSpec>,
}
