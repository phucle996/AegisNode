// Controller Configuration Schema cho `aegisnode server`
// Phục vụ cấu hình cho nền tảng quản trị tập trung Multi-Node Cloud Native HA Cluster

use std::fs;
use std::path::Path;

use aegis_core::{AegisError, Result};
use serde::{Deserialize, Serialize};

/// Cấu hình tổng thể của AegisNode Controller Server
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControllerConfig {
    pub version: String,
    pub server: ControllerServerConfig,
    pub database: ControllerDatabaseConfig,
    #[serde(default)]
    pub tls: ControllerTlsConfig,
}

/// Cấu hình HTTP Server & JWT Secret cho Controller
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControllerServerConfig {
    pub host: String,
    pub port: u16,
    pub auth_secret: String,
    #[serde(default = "default_session_ttl")]
    pub session_ttl_seconds: u64,
}

fn default_session_ttl() -> u64 {
    86400 // 24 hours default
}

/// Cấu hình kết nối PostgreSQL High Availability Cluster
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControllerDatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_seconds: u64,
}

fn default_max_connections() -> u32 {
    20
}

fn default_connect_timeout() -> u64 {
    10
}

/// Cấu hình mTLS (Mutual TLS v1.3) và PKI/CA cho Node Enrollment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ControllerTlsConfig {
    pub enabled: bool,
    pub ca_cert_path: Option<String>,
    pub ca_key_path: Option<String>,
    pub server_cert_path: Option<String>,
    pub server_key_path: Option<String>,
}

impl ControllerConfig {
    /// Nạp cấu hình Controller từ file YAML
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| {
            AegisError::Configuration(format!(
                "Failed to read controller config file '{:?}': {}",
                path.as_ref(),
                e
            ))
        })?;

        let config: ControllerConfig = serde_yaml::from_str(&content).map_err(|e| {
            AegisError::Configuration(format!("Failed to parse controller YAML config: {e}"))
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Kiểm tra tính hợp lệ của cấu hình Controller
    pub fn validate(&self) -> Result<()> {
        if self.server.port == 0 {
            return Err(AegisError::Validation(
                "Controller server port must be > 0".to_string(),
            ));
        }
        if self.database.url.trim().is_empty() {
            return Err(AegisError::Validation(
                "Controller database url cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            server: ControllerServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8443,
                auth_secret: "aegisnode-default-secret-key-change-in-production".to_string(),
                session_ttl_seconds: 86400,
            },
            database: ControllerDatabaseConfig {
                url: "postgresql://postgres:postgres@localhost:5432/aegisnode".to_string(),
                max_connections: 20,
                connect_timeout_seconds: 10,
            },
            tls: ControllerTlsConfig::default(),
        }
    }
}
