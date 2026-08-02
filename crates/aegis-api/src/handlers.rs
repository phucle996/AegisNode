// API Handlers cho Axum Web Framework (HTTP Server & Unix Socket IPC)
// Xử lý các yêu cầu RESTful và chuyển giao cho Repository Layer / SafeApplyManager / Blocker

use std::sync::Arc;

use aegis_core::ExecutionId;
use aegis_firewall::{
    BlockManager, DockerExposureReport, DockerInspector, RouterManager, SysctlSnapshot,
};
use aegis_models::blocker::{BlockEntry, BlockerConfig};
use aegis_models::firewall::FirewallPolicy;
use aegis_policy::PolicyValidator;
use aegis_storage::{AuditRepository, PolicyRepository};
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use crate::state::AppState;

/// Status Response Payload
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub status: &'static str,
    pub version: &'static str,
    pub capability: aegis_firewall::NftCapabilityReport,
}

/// Request Payload cho Apply API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyRequest {
    pub policy: FirewallPolicy,
    pub rollback_timeout_seconds: Option<u64>,
}

/// Request Payload cho Confirm / Rollback API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionActionRequest {
    pub execution_id: ExecutionId,
}

/// Request Payload cho Router Forwarding API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouterForwardingRequest {
    pub enabled: bool,
}

/// Request Payload cho Blocker Add API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockAddRequest {
    pub ip: String,
    pub duration_seconds: Option<u64>,
    pub reason: Option<String>,
}

/// Request Payload cho Blocker Remove API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockRemoveRequest {
    pub ip: String,
}

/// Handler `GET /v1/status`: Trả về thông tin trạng thái agent và capabilities
pub async fn get_status_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<StatusResponse>, aegis_core::AegisError> {
    let capability = state.capability_detector.detect().await?;
    Ok(Json(StatusResponse {
        status: "RUNNING",
        version: env!("CARGO_PKG_VERSION"),
        capability,
    }))
}

/// Handler `GET /v1/firewall/policy`: Trả về policy mới nhất
pub async fn get_policy_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Option<FirewallPolicy>>, aegis_core::AegisError> {
    let policy = state.repository.get_latest_policy().await?;
    Ok(Json(policy))
}

/// Handler `POST /v1/firewall/validate`: Kiểm tra tính hợp lệ của Policy
pub async fn validate_policy_handler(
    Json(policy): Json<FirewallPolicy>,
) -> Json<aegis_policy::ValidationReport> {
    let report = PolicyValidator::validate(&policy);
    Json(report)
}

/// Handler `POST /v1/firewall/apply`: Thực thi Safe Apply
pub async fn apply_policy_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ApplyRequest>,
) -> Result<Json<aegis_firewall::ApplyExecution>, aegis_core::AegisError> {
    let timeout = req
        .rollback_timeout_seconds
        .unwrap_or(state.config.firewall.rollback_timeout_seconds);

    let execution = state
        .safe_apply_manager
        .execute_safe_apply(&req.policy, timeout)
        .await?;

    let policy_hash = aegis_policy::PolicyHasher::compute_hash(&req.policy);
    let _ = state
        .repository
        .save_policy(&req.policy, &policy_hash)
        .await;
    let _ = state
        .repository
        .record_audit(
            "FIREWALL_APPLY",
            "api_user",
            &req.policy.metadata.name,
            &format!("Execution ID: {}", execution.execution_id),
        )
        .await;

    Ok(Json(execution))
}

/// Handler `POST /v1/firewall/confirm`: Xác nhận đợt Apply Execution
pub async fn confirm_policy_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExecutionActionRequest>,
) -> Result<Json<aegis_firewall::ApplyExecution>, aegis_core::AegisError> {
    let execution = state.safe_apply_manager.confirm(&req.execution_id).await?;

    let _ = state
        .repository
        .record_audit(
            "FIREWALL_CONFIRM",
            "api_user",
            &format!("Execution: {}", req.execution_id),
            "Confirmed policy changes",
        )
        .await;

    Ok(Json(execution))
}

/// Handler `POST /v1/firewall/rollback`: Rollback thủ công theo Execution ID
pub async fn rollback_policy_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExecutionActionRequest>,
) -> Result<Json<aegis_firewall::ApplyExecution>, aegis_core::AegisError> {
    let execution = state.safe_apply_manager.rollback(&req.execution_id).await?;

    let _ = state
        .repository
        .record_audit(
            "FIREWALL_ROLLBACK",
            "api_user",
            &format!("Execution: {}", req.execution_id),
            "Manual rollback triggered",
        )
        .await;

    Ok(Json(execution))
}

/// Handler `GET /v1/audit`: Truy vấn lịch sử Audit log
pub async fn get_audit_logs_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<aegis_storage::AuditRecord>>, aegis_core::AegisError> {
    let records = state.repository.list_audits(50).await?;
    Ok(Json(records))
}

/// Handler `GET /v1/docker/exposure`: Phân tích rủi ro phơi nhiễm cổng của Docker Containers
pub async fn get_docker_exposure_handler()
-> Result<Json<DockerExposureReport>, aegis_core::AegisError> {
    let inspector = DockerInspector::default_prod();
    let report = inspector.inspect().await?;
    Ok(Json(report))
}

/// Handler `POST /v1/router/forwarding`: Bật/tắt IP Forwarding cho Router mode
pub async fn set_router_forwarding_handler(
    Json(req): Json<RouterForwardingRequest>,
) -> Result<Json<SysctlSnapshot>, aegis_core::AegisError> {
    let snapshot = RouterManager::set_ip_forwarding(req.enabled).await?;
    Ok(Json(snapshot))
}

/// Handler `GET /v1/blocker/entries`: Trả về danh sách IP đang bị cấm
pub async fn get_blocker_entries_handler() -> Result<Json<Vec<BlockEntry>>, aegis_core::AegisError>
{
    let mut mgr = BlockManager::new(BlockerConfig::default());
    let entries = mgr.list_blocks();
    Ok(Json(entries))
}

/// Handler `POST /v1/blocker/add`: Thêm IP thủ công vào Blocklist
pub async fn add_block_entry_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BlockAddRequest>,
) -> Result<Json<BlockEntry>, aegis_core::AegisError> {
    let mut mgr = BlockManager::new(BlockerConfig::default());
    let reason = req.reason.as_deref().unwrap_or("Manual API Block");
    let entry = mgr.add_block(&req.ip, req.duration_seconds, reason, "api_user")?;

    let _ = state
        .repository
        .record_audit("BLOCK_ADD", "api_user", &req.ip, reason)
        .await;

    Ok(Json(entry))
}

/// Handler `POST /v1/blocker/remove`: Gỡ bỏ IP khỏi Blocklist
pub async fn remove_block_entry_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BlockRemoveRequest>,
) -> Result<Json<Option<BlockEntry>>, aegis_core::AegisError> {
    let mut mgr = BlockManager::new(BlockerConfig::default());
    let entry = mgr.remove_block(&req.ip)?;

    let _ = state
        .repository
        .record_audit("BLOCK_REMOVE", "api_user", &req.ip, "Manual API Unblock")
        .await;

    Ok(Json(entry))
}
