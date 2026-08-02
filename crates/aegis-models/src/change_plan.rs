// Combined Change Plan & Multi-Node Rollout Domain Models cho AegisNode Stage 2
// Định nghĩa cấp độ rủi ro (Risk Levels), Idempotency Keys, Execution Steps & Health Check Probes

use serde::{Deserialize, Serialize};
use uuid::Uuid;

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

/// Kế hoạch Thay đổi Hợp nhất (Combined Change Plan cho Single Node hoặc Multi-Node)
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

/// DTO Báo cáo tiến độ Rollout trên Multi-Node Cluster
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RolloutReport {
    pub rollout_id: Uuid,
    pub status: String,
    pub progress_percent: u32,
    pub error_message: Option<String>,
}
