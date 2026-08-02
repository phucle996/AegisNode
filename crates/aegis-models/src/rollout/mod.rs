//! AegisNode Multi-Node Change Plan & Rollout Domain Models
//! Quản lý NodeChangePlan, RolloutStrategy (Canary/Batch), Health Checks, Risk Assessment và Step Progress Tracking.

pub mod change_plan;

pub use change_plan::{
    BatchConfig, ExecutionStep, HealthCheckSpec, NodeChangePlan, NodeRolloutState,
    NodeRolloutStatus, RiskLevel, RolloutReport, RolloutSpec, RolloutStrategy, StepStatus,
};
