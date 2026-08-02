// Shared State cho Axum HTTP & Unix Socket Server Handlers (Local Agent)
// Chứa toàn bộ shared dependencies: SafeApplyManager, BlockManager, SqliteRepository, AgentConfig

use std::sync::Arc;
use tokio::sync::Mutex;

use aegis_config::AgentConfig;
use aegis_firewall::{BlockManager, CapabilityDetector, SafeApplyManager};
use aegis_storage::{PolicyRepository, SqliteRepository};

/// Structure chứa toàn bộ Shared State cho Axum Web Framework (Local Agent Mode)
#[derive(Clone)]
pub struct AppState {
    /// Quản lý Safe Apply với cơ chế transaction + auto-rollback
    pub safe_apply: Arc<SafeApplyManager>,
    /// Quản lý danh sách IP bị chặn động qua nftables set
    pub block_manager: Arc<Mutex<BlockManager>>,
    /// Phát hiện khả năng hỗ trợ hệ thống (kernel modules, nftables)
    pub capability_detector: Arc<CapabilityDetector>,
    /// Repository lưu trữ SQLite local
    pub repository: Arc<SqliteRepository>,
    /// Cấu hình Agent
    pub config: Arc<AgentConfig>,
    /// Phiên bản policy hiện hành (dùng cho version tracking)
    pub current_version: Arc<Mutex<u64>>,
}

impl AppState {
    pub fn new(
        safe_apply: Arc<SafeApplyManager>,
        block_manager: Arc<Mutex<BlockManager>>,
        capability_detector: Arc<CapabilityDetector>,
        repository: Arc<SqliteRepository>,
        config: Arc<AgentConfig>,
    ) -> Self {
        // Khởi tạo AppState với cờ phiên bản mặc định
        Self {
            safe_apply,
            block_manager,
            capability_detector,
            repository,
            config,
            current_version: Arc::new(Mutex::new(0)),
        }
    }

    /// Khởi tạo AppState và khôi phục phiên bản Policy mới nhất từ SQLite CSDL local
    pub async fn init_with_persisted_version(
        safe_apply: Arc<SafeApplyManager>,
        block_manager: Arc<Mutex<BlockManager>>,
        capability_detector: Arc<CapabilityDetector>,
        repository: Arc<SqliteRepository>,
        config: Arc<AgentConfig>,
    ) -> Self {
        // Truy vấn bản ghi policy mới nhất từ SQLite CSDL
        let initial_version = if let Ok(Some(latest)) = repository.get_latest_policy().await {
            latest.metadata.version as u64
        } else {
            0
        };

        Self {
            safe_apply,
            block_manager,
            capability_detector,
            repository,
            config,
            current_version: Arc::new(Mutex::new(initial_version)),
        }
    }
}
