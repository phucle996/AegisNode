// Cấu hình daemon Local Agent cho AegisNode
// Đọc và validate tập tin cấu hình /etc/aegisnode/agent.yaml với các giá trị mặc định an toàn

use std::path::PathBuf;

use aegis_core::{AegisError, Result};
use serde::{Deserialize, Serialize};

/// Cấu hình tổng thể của AegisNode Agent Daemon
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub firewall: FirewallConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    pub unix_socket: PathBuf,
    pub http: HttpConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpConfig {
    pub enabled: bool,
    pub bind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfig {
    pub database: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallConfig {
    pub policy_file: PathBuf,
    pub rollback_timeout_seconds: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                unix_socket: PathBuf::from("/run/aegisnode/agent.sock"),
                http: HttpConfig {
                    enabled: true,
                    bind: "127.0.0.1:8080".to_string(),
                },
            },
            storage: StorageConfig {
                database: PathBuf::from("/var/lib/aegisnode/aegis.db"),
            },
            firewall: FirewallConfig {
                policy_file: PathBuf::from("/etc/aegisnode/firewall.yaml"),
                rollback_timeout_seconds: 30,
            },
        }
    }
}

impl AgentConfig {
    /// Kiểm tra tính hợp lệ của cấu hình (Path tuyệt đối, HTTP Bind an toàn)
    pub fn validate(&self) -> Result<()> {
        if self.server.http.enabled
            && !self.server.http.bind.starts_with("127.0.0.1")
            && !self.server.http.bind.starts_with("localhost")
        {
            return Err(AegisError::Configuration(
                "Security Warning: Agent HTTP server must bind to localhost/127.0.0.1 by default"
                    .to_string(),
            ));
        }

        if self.firewall.rollback_timeout_seconds == 0 {
            return Err(AegisError::Configuration(
                "Rollback timeout seconds must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Đọc cấu hình từ chuỗi YAML
    pub fn from_yaml(yaml_str: &str) -> Result<Self> {
        let config: AgentConfig = serde_yaml::from_str(yaml_str).map_err(|e| {
            AegisError::Configuration(format!("Failed to parse agent config YAML: {e}"))
        })?;
        config.validate()?;
        Ok(config)
    }
}
