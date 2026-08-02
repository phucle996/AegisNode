// Trạng thái Giao dịch Apply (Apply Execution State Machine) cho AegisNode
// Theo dõi toàn bộ vòng đời của một thay đổi Policy từ khởi tạo đến xác nhận hoặc Rollback

use aegis_core::{ExecutionId, PolicyId, SnapshotId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Định nghĩa các trạng thái của Apply Execution State Machine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionState {
    Created,
    Validated,
    Snapshotted,
    AppliedPendingConfirmation,
    Committed,
    RollingBack,
    RolledBack,
    Failed,
}

impl ExecutionState {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Committed | Self::RolledBack | Self::Failed)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::AppliedPendingConfirmation)
    }
}

/// Dữ liệu chi tiết của một đợt thực thi Apply Execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyExecution {
    pub execution_id: ExecutionId,
    pub policy_id: PolicyId,
    pub snapshot_id: SnapshotId,
    pub state: ExecutionState,
    pub timeout_seconds: u64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub error_message: Option<String>,
}

impl ApplyExecution {
    pub fn new(
        execution_id: ExecutionId,
        policy_id: PolicyId,
        snapshot_id: SnapshotId,
        timeout_seconds: u64,
    ) -> Self {
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(timeout_seconds as i64);

        Self {
            execution_id,
            policy_id,
            snapshot_id,
            state: ExecutionState::Created,
            timeout_seconds,
            created_at: now,
            expires_at,
            error_message: None,
        }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}
