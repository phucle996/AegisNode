//! AegisNode Config Crate
//! Quản lý việc đọc, ghi và validate các file cấu hình YAML/JSON của Daemon và Agent.

use serde::{Deserialize, Serialize};

/// Cấu hình tổng quan Daemon AegisNode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub socket_path: String,
    pub http_bind: String,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            socket_path: "/run/aegisnode/agent.sock".to_string(),
            http_bind: "127.0.0.1:8080".to_string(),
        }
    }
}
