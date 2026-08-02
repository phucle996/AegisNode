//! AegisNode Configuration Crate
//! Quản lý tập tin cấu hình agent.yaml và controller.yaml cho nền tảng AegisNode.

pub mod agent_config;
pub mod controller_config;

pub use agent_config::{AgentConfig, FirewallConfig, HttpConfig, ServerConfig, StorageConfig};
pub use controller_config::{
    ControllerConfig, ControllerDatabaseConfig, ControllerServerConfig, ControllerTlsConfig,
};
