// Model đại diện cho thông tin Docker Containers, Networks và Exposure policy
// Phục vụ tính năng Docker Container Discovery & Public Exposure Warning

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::firewall::{CidrSpec, TransportProtocol};

/// Thông tin một Docker Container phát hiện trên máy host
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    // Bổ sung thuộc tính tỉ lệ tiêu thụ CPU (%) từ docker stats
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cpu_perc: Option<String>,
    // Bổ sung thuộc tính bộ nhớ RAM tiêu thụ (Usage / Limit) từ docker stats
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mem_usage: Option<String>,
    #[serde(default)]
    pub networks: Vec<String>,
    #[serde(default)]
    pub published_ports: Vec<PublishedPort>,
    #[serde(default)]
    pub labels: HashMap<String, String>,
}

/// Thông tin Docker Network
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockerNetwork {
    pub id: String,
    pub name: String,
    pub driver: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subnet: Option<String>,
}

/// Thông tin Cổng (Port) được Container publish ra máy host
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishedPort {
    pub host_ip: String,
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: TransportProtocol,
}

/// Mức độ phơi nhiễm (Exposure Level) của Container ra mạng ngoài
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContainerExposure {
    Private,
    PublicUnrestricted,
    PublicRestricted(Vec<CidrSpec>),
}

/// Chính sách quy định mức độ phơi nhiễm đọc từ Docker Labels (`aegis.exposure`)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContainerLabelPolicy {
    pub exposure: ContainerExposure,
    #[serde(default)]
    pub allowed_cidrs: Vec<CidrSpec>,
}
