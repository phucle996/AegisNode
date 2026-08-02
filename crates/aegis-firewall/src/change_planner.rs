// Combined Change Planner Engine cho AegisNode Stage 2
// Tạo chuỗi các bước thực thi từ Snapshot -> Temp Management Rule -> Stage -> Activate -> Health Check Probe -> Confirm

use aegis_core::Result;
use aegis_models::change_plan::{ExecutionStep, NodeChangePlan, RiskLevel, StepStatus};

/// Đánh giá cấp độ rủi ro dựa trên giao diện và cấu hình bị tác động
pub fn assess_risk(
    has_network_change: bool,
    affects_management_iface: bool,
    has_firewall_change: bool,
) -> RiskLevel {
    if affects_management_iface {
        RiskLevel::Critical
    } else if has_network_change {
        RiskLevel::High
    } else if has_firewall_change {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    }
}

/// Planner Engine sinh thứ tự thực thi chuẩn và thứ tự Rollback ngược lại
#[derive(Debug, Clone, Default)]
pub struct CombinedChangePlanner;

impl CombinedChangePlanner {
    pub fn new() -> Self {
        Self
    }

    /// Sinh chuỗi 6 bước thực thi tuần tự an toàn tuyệt đối cho Node
    pub fn plan_execution(&self, plan: &mut NodeChangePlan) -> Result<()> {
        let steps = vec![
            ExecutionStep {
                step_id: "step_01_snapshot".to_string(),
                name: "Snapshot Current System State".to_string(),
                action: "Take system snapshot of firewall, network & services".to_string(),
                status: StepStatus::Pending,
            },
            ExecutionStep {
                step_id: "step_02_temp_allow".to_string(),
                name: "Install Temporary Management Allow Rule".to_string(),
                action: "Bypass temporary allow rule for mTLS management interface".to_string(),
                status: StepStatus::Pending,
            },
            ExecutionStep {
                step_id: "step_03_stage".to_string(),
                name: "Stage Candidate Configuration".to_string(),
                action: "Write candidate rules to staging directory".to_string(),
                status: StepStatus::Pending,
            },
            ExecutionStep {
                step_id: "step_04_activate".to_string(),
                name: "Activate Candidate Rules".to_string(),
                action: "Atomic swap candidate rules into active runtime".to_string(),
                status: StepStatus::Pending,
            },
            ExecutionStep {
                step_id: "step_05_health_check".to_string(),
                name: "Probe Health Checks".to_string(),
                action: "Probe gateway, DNS and Controller mTLS connectivity".to_string(),
                status: StepStatus::Pending,
            },
            ExecutionStep {
                step_id: "step_06_confirm".to_string(),
                name: "Confirm Rollout Completion".to_string(),
                action: "Clean up temporary rules & commit final state".to_string(),
                status: StepStatus::Pending,
            },
        ];

        plan.steps = steps;
        Ok(())
    }
}
