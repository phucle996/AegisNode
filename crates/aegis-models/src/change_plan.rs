// Combined Change Plan & Multi-Node Rollout Domain Models cho AegisNode Stage 2
// Định nghĩa cấp độ rủi ro (Risk Levels), Idempotency Keys, Execution Steps,
// Health Check Probes, Rollout Strategies, Batch Config & Node-level Status Tracking

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Risk Level ──────────────────────────────────────────────────────────────

/// Cấp độ rủi ro của Kế hoạch Thay đổi (Risk Assessment)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

// ─── Step Status ─────────────────────────────────────────────────────────────

/// Trạng thái thực thi từng bước trong Change Plan
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StepStatus {
    #[default]
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

/// DTO biểu diễn một bước thực thi chi tiết
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionStep {
    pub step_id: String,
    pub name: String,
    pub action: String,
    pub status: StepStatus,
}

// ─── Health Check ────────────────────────────────────────────────────────────

/// Cấu hình kiểm tra sức khỏe mạng sau khi áp dụng cấu hình (Health Check Probe)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthCheckSpec {
    pub probe_gateway: bool,
    pub probe_dns: bool,
    pub controller_url: Option<String>,
    pub timeout_seconds: u64,
}

impl Default for HealthCheckSpec {
    fn default() -> Self {
        Self {
            probe_gateway: true,
            probe_dns: true,
            controller_url: None,
            timeout_seconds: 30,
        }
    }
}

// ─── Single-Node Change Plan ──────────────────────────────────────────────────

/// Kế hoạch Thay đổi Hợp nhất (Combined Change Plan cho Single Node)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeChangePlan {
    pub id: Uuid,
    pub idempotency_key: String,
    pub target_node_id: Uuid,
    pub risk_level: RiskLevel,
    pub steps: Vec<ExecutionStep>,
    pub health_check: HealthCheckSpec,
}

impl Default for NodeChangePlan {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            idempotency_key: format!("plan_{}", Uuid::new_v4().simple()),
            target_node_id: Uuid::new_v4(),
            risk_level: RiskLevel::Low,
            steps: vec![],
            health_check: HealthCheckSpec::default(),
        }
    }
}

// ─── Rollout Strategy & Config ────────────────────────────────────────────────

/// Chiến lược phân phối Rollout trên Multi-Node Cluster
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RolloutStrategy {
    /// Admin xác nhận thủ công từng node — tuyệt đối kiểm soát
    Manual,
    /// Push đến tất cả nodes đồng thời — nhanh nhất, rủi ro cao nhất
    AllAtOnce,
    /// 1 node canary đầu tiên, health gate OK mới mở rộng ra fleet
    #[default]
    Canary,
    /// batch_size nodes mỗi lần, tuân thủ max_unavailable
    Batch,
}

/// Cấu hình chi tiết cho chiến lược Batch Rollout
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchConfig {
    /// Số node mỗi batch
    pub batch_size: usize,
    /// Số node tối đa được phép không hoạt động đồng thời trong fleet
    pub max_unavailable: usize,
    /// Thời gian chờ (giây) giữa các batch để hệ thống ổn định
    pub pause_between_batches_secs: u64,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 1,
            max_unavailable: 1,
            pause_between_batches_secs: 30,
        }
    }
}

// ─── Node Rollout Status ──────────────────────────────────────────────────────

/// Trạng thái Rollout của từng Node trong fleet
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NodeRolloutState {
    #[default]
    Pending,
    Running,
    Succeeded,
    Failed,
    /// Node offline trong quá trình rollout — sẽ reconcile khi quay lại online
    OfflineSkipped,
    /// Admin bỏ qua node này có ghi vết Audit
    SkippedWithAudit,
    /// Node đã được rollback về version trước
    RolledBack,
}

/// Trạng thái chi tiết của từng node trong một Rollout
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeRolloutStatus {
    pub node_id: Uuid,
    pub state: NodeRolloutState,
    pub current_step: Option<String>,
    pub error_message: Option<String>,
}

// ─── Rollout Spec (Multi-Node) ────────────────────────────────────────────────

/// Thông số điều phối Rollout trên Multi-Node Cluster
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RolloutSpec {
    pub rollout_id: Uuid,
    pub idempotency_key: String,
    pub strategy: RolloutStrategy,
    pub batch_config: BatchConfig,
    /// Phần trăm node thất bại tối đa trước khi dừng toàn fleet
    pub failure_threshold_percent: u32,
    /// Danh sách Node IDs mục tiêu theo thứ tự ưu tiên
    pub targets: Vec<Uuid>,
    pub health_check: HealthCheckSpec,
    pub risk_level: RiskLevel,
}

impl Default for RolloutSpec {
    fn default() -> Self {
        Self {
            rollout_id: Uuid::new_v4(),
            idempotency_key: format!("rollout_{}", Uuid::new_v4().simple()),
            strategy: RolloutStrategy::Canary,
            batch_config: BatchConfig::default(),
            failure_threshold_percent: 20,
            targets: vec![],
            health_check: HealthCheckSpec::default(),
            risk_level: RiskLevel::Medium,
        }
    }
}

// ─── Rollout Report ───────────────────────────────────────────────────────────

/// DTO báo cáo tiến độ tổng thể Rollout trên fleet
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RolloutReport {
    pub rollout_id: Uuid,
    pub status: String,
    pub strategy: String,
    pub progress_percent: u32,
    pub total_nodes: usize,
    pub succeeded_nodes: usize,
    pub failed_nodes: usize,
    pub pending_nodes: usize,
    pub error_message: Option<String>,
    pub node_statuses: Vec<NodeRolloutStatus>,
}
