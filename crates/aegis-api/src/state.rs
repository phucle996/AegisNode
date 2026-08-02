// Shared State cho Axum HTTP & Unix Socket Server Handlers

use std::sync::Arc;

use aegis_config::AgentConfig;
use aegis_firewall::{CapabilityDetector, SafeApplyManager};
use aegis_storage::SqliteRepository;

/// Structure chứa toàn bộ Shared State cho Axum Web Framework
#[derive(Clone)]
pub struct AppState {
    pub safe_apply_manager: Arc<SafeApplyManager>,
    pub capability_detector: Arc<CapabilityDetector>,
    pub repository: Arc<SqliteRepository>,
    pub config: Arc<AgentConfig>,
}

impl AppState {
    pub fn new(
        safe_apply_manager: Arc<SafeApplyManager>,
        capability_detector: Arc<CapabilityDetector>,
        repository: Arc<SqliteRepository>,
        config: Arc<AgentConfig>,
    ) -> Self {
        Self {
            safe_apply_manager,
            capability_detector,
            repository,
            config,
        }
    }
}
