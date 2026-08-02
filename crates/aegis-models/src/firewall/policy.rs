// Model định nghĩa Firewall Policy, Rules, Actions, Protocols và Specs
// Phục vụ công đoạn validate, compile nftables và an toàn hệ thống

use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;

use aegis_core::{AegisError, PolicyId, Result, RuleId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// API Version được hỗ trợ duy nhất ở MVP Stage
pub const SUPPORTED_API_VERSION: &str = "aegisnode.io/v1";

/// Kind được hỗ trợ duy nhất cho Firewall Policy
pub const SUPPORTED_FIREWALL_KIND: &str = "FirewallPolicy";

/// Model chính biểu diễn Firewall Policy tải từ YAML/JSON
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallPolicy {
    pub api_version: String,
    pub kind: String,
    pub metadata: PolicyMetadata,
    pub defaults: FirewallDefaults,
    #[serde(default)]
    pub rules: Vec<FirewallRule>,
}

impl FirewallPolicy {
    /// Kiểm tra tính hợp lệ về phiên bản Schema Version và Kind
    pub fn validate_schema_version(&self) -> Result<()> {
        if self.api_version != SUPPORTED_API_VERSION {
            return Err(AegisError::Validation(format!(
                "Unsupported apiVersion: '{}'. Expected: '{}'",
                self.api_version, SUPPORTED_API_VERSION
            )));
        }
        if self.kind != SUPPORTED_FIREWALL_KIND {
            return Err(AegisError::Validation(format!(
                "Unsupported kind: '{}'. Expected: '{}'",
                self.kind, SUPPORTED_FIREWALL_KIND
            )));
        }
        Ok(())
    }
}

/// Metadata chứa thông tin tên, định danh và nhãn của Policy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyMetadata {
    pub name: String,
    #[serde(default = "PolicyId::new_v4")]
    pub id: PolicyId,
    #[serde(default)]
    pub version: u64,
    #[serde(default)]
    pub labels: HashMap<String, String>,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
}

/// Cấu hình mặc định cho các chain Input, Output và Forward
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallDefaults {
    pub input: FirewallAction,
    pub output: FirewallAction,
    pub forward: FirewallAction,
}

/// Hướng di chuyển của packet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FirewallDirection {
    Input,
    Output,
    Forward,
}

/// Hành xử áp dụng cho rule match
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FirewallAction {
    Accept,
    Drop,
    Reject,
}

/// Giao thức truyền tải (Transport Protocol)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransportProtocol {
    Tcp,
    Udp,
    Icmp,
    Icmpv6,
}

/// Trạng thái kết nối (Connection Tracking / Conntrack State)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    New,
    Established,
    Related,
    Invalid,
}

/// Vai trò của card mạng
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterfaceRole {
    Wan,
    Lan,
    Management,
}

/// Chọn interface theo Vai trò hoặc Tên cụ thể
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InterfaceSelector {
    Role(InterfaceRole),
    Name(String),
}

/// Định nghĩa một Rule lọc Firewall chi tiết
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallRule {
    pub id: RuleId,
    pub direction: FirewallDirection,
    pub action: FirewallAction,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub protocol: Option<TransportProtocol>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub connection_states: Vec<ConnectionState>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_cidrs: Vec<CidrSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub destination_cidrs: Vec<CidrSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_ports: Vec<PortSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub destination_ports: Vec<PortSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interfaces: Vec<InterfaceSelector>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rate_limit: Option<RateLimit>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_options: Option<LogOptions>,
}

/// Specification cho Port: Single Port (80), Port Range (8000-8080)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PortSpec {
    Single(u16),
    Range(u16, u16),
}

impl PortSpec {
    /// Kiểm tra Port nằm trong dải hợp lệ 1..=65535
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Single(p) => {
                if *p == 0 {
                    return Err(AegisError::Validation(
                        "Port 0 is invalid. Port must be between 1 and 65535".to_string(),
                    ));
                }
            }
            Self::Range(start, end) => {
                if *start == 0 || *end == 0 {
                    return Err(AegisError::Validation(
                        "Port 0 is invalid. Port must be between 1 and 65535".to_string(),
                    ));
                }
                if start > end {
                    return Err(AegisError::Validation(format!(
                        "Invalid port range: {start}-{end}. Start port must be <= end port",
                    )));
                }
            }
        }
        Ok(())
    }
}

impl Serialize for PortSpec {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Single(p) => serializer.serialize_u16(*p),
            Self::Range(start, end) => serializer.serialize_str(&format!("{start}-{end}")),
        }
    }
}

impl<'de> Deserialize<'de> for PortSpec {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum PortValue {
            Int(u32),
            Str(String),
        }

        let value = PortValue::deserialize(deserializer)?;
        match value {
            PortValue::Int(p) => {
                if !(1..=65535).contains(&p) {
                    return Err(serde::de::Error::custom(format!(
                        "Port number {p} out of range 1..65535"
                    )));
                }
                Ok(PortSpec::Single(p as u16))
            }
            PortValue::Str(s) => {
                if let Ok(p) = s.parse::<u32>() {
                    if !(1..=65535).contains(&p) {
                        return Err(serde::de::Error::custom(format!(
                            "Port number {p} out of range 1..65535"
                        )));
                    }
                    return Ok(PortSpec::Single(p as u16));
                }
                let parts: Vec<&str> = s.split('-').collect();
                if parts.len() == 2 {
                    let start = parts[0]
                        .trim()
                        .parse::<u32>()
                        .map_err(serde::de::Error::custom)?;
                    let end = parts[1]
                        .trim()
                        .parse::<u32>()
                        .map_err(serde::de::Error::custom)?;
                    if !(1..=65535).contains(&start) || !(1..=65535).contains(&end) {
                        return Err(serde::de::Error::custom(
                            "Port in range out of range 1..65535".to_string(),
                        ));
                    }
                    if start > end {
                        return Err(serde::de::Error::custom(format!(
                            "Invalid range start {start} > end {end}"
                        )));
                    }
                    return Ok(PortSpec::Range(start as u16, end as u16));
                }
                Err(serde::de::Error::custom(format!(
                    "Invalid port spec: '{s}'"
                )))
            }
        }
    }
}

/// Specification đại diện cho dải địa chỉ IP (CIDR format: 192.168.1.0/24 hoặc 10.0.0.1/32)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CidrSpec(pub String);

impl CidrSpec {
    /// Validate định dạng CIDR IPv4 hoặc IPv6
    pub fn validate(&self) -> Result<()> {
        let s = self.0.trim();
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(AegisError::Validation(format!(
                "Invalid CIDR string: '{s}'. Must be in IP/prefix format"
            )));
        }
        let ip_str = parts[0];
        let prefix_str = parts[1];

        let ip = IpAddr::from_str(ip_str).map_err(|_| {
            AegisError::Validation(format!("Invalid IP address in CIDR: '{ip_str}'"))
        })?;

        let prefix = prefix_str.parse::<u8>().map_err(|_| {
            AegisError::Validation(format!("Invalid CIDR prefix length: '{prefix_str}'"))
        })?;

        match ip {
            IpAddr::V4(_) => {
                if prefix > 32 {
                    return Err(AegisError::Validation(format!(
                        "IPv4 CIDR prefix /{prefix} out of range (0..32)"
                    )));
                }
            }
            IpAddr::V6(_) => {
                if prefix > 128 {
                    return Err(AegisError::Validation(format!(
                        "IPv6 CIDR prefix /{prefix} out of range (0..128)"
                    )));
                }
            }
        }
        Ok(())
    }
}

impl Serialize for CidrSpec {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for CidrSpec {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let spec = CidrSpec(s);
        spec.validate().map_err(serde::de::Error::custom)?;
        Ok(spec)
    }
}

/// Cấu hình Rate Limit (Giới hạn tần suất lưu lượng)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    pub packets_per_second: u32,
    pub burst: u32,
}

/// Cấu hình Logging cho Rule
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogOptions {
    pub enabled: bool,
    #[serde(default)]
    pub prefix: String,
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub rate_limit: Option<RateLimit>,
}

fn default_log_level() -> String {
    "info".to_string()
}
