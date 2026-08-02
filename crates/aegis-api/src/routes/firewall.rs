//! Firewall & Policy REST API Handlers
//! Quản lý xem, kiểm tra, thực thi (Safe Apply), xác nhận và hoàn tác (Rollback) tường lửa nftables.

use std::sync::Arc;
use aegis_core::{AegisError, ExecutionId};
use aegis_firewall::{DockerInspector, RouterManager};
use aegis_models::firewall::FirewallPolicy;
use aegis_observability::prometheus::GLOBAL_METRICS;
use aegis_policy::PolicyValidator;
use aegis_storage::{AuditRepository, PolicyRepository};
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::state::AppState;

/// Thời gian chờ tối đa cho Safe Apply transaction (giây) trước khi auto-rollback
const DEFAULT_TIMEOUT_SECS: u64 = 60;

#[derive(Debug, Deserialize)]
pub struct ConfirmPayload {
    /// ID của execution Safe Apply cần xác nhận
    pub execution_id: String,
}

#[derive(Debug, Deserialize)]
pub struct RollbackPayload {
    /// ID của execution Safe Apply cần hoàn tác
    pub execution_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ForwardingPayload {
    pub enabled: bool,
}

/// Handler `GET /metrics`: Xuất chỉ số Prometheus Metrics Text Exposition
pub async fn prometheus_metrics_handler() -> impl IntoResponse {
    let output = GLOBAL_METRICS.render_prometheus_exposition();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        output,
    )
}

/// Handler `GET /v1/status`: Kiểm tra trạng thái Daemon và version policy hiện tại
pub async fn get_status_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    let current_version = state.current_version.lock().await;

    Ok(Json(serde_json::json!({
        "status": "online",
        "service": "aegisnode-agent",
        "policy_version": *current_version
    })))
}

/// Handler `GET /v1/firewall/policy`: Lấy cấu hình Policy hiện tại từ SQLite
pub async fn get_policy_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    let current_version = state.current_version.lock().await;
    let policy = state
        .repository
        .get_latest_policy()
        .await?;

    Ok(Json(serde_json::json!({
        "version": *current_version,
        "policy": policy
    })))
}

/// Handler `POST /v1/firewall/validate`: Kiểm tra tính hợp lệ Policy trước khi áp dụng
pub async fn validate_policy_handler(
    Json(policy): Json<FirewallPolicy>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    // ValidationReport không phải Result — kiểm tra is_valid() để detect lỗi
    let report = PolicyValidator::validate(&policy);
    if !report.is_valid() {
        return Err(AegisError::Validation(format!(
            "{} validation error(s)",
            report.errors.len()
        )));
    }

    // Tính hash của policy qua std hasher (không cần crypto-grade ở đây)
    let hash = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        format!("{:?}", policy.metadata.id).hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    };

    Ok(Json(serde_json::json!({
        "valid": true,
        "hash": hash,
        "warnings": report.warnings.len()
    })))
}

/// Handler `POST /v1/firewall/apply`: Thực thi Safe Apply Policy với auto-rollback sau timeout
pub async fn apply_policy_handler(
    State(state): State<Arc<AppState>>,
    Json(policy): Json<FirewallPolicy>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    // Validate trước khi apply
    let report = PolicyValidator::validate(&policy);
    let warning_count = report.warnings.len();
    if !report.is_valid() {
        return Err(AegisError::Validation("Policy validation failed".to_string()));
    }

    // Gọi SafeApplyManager — tự động rollback sau DEFAULT_TIMEOUT_SECS nếu không confirm
    let execution = state
        .safe_apply
        .execute_safe_apply(&policy, DEFAULT_TIMEOUT_SECS)
        .await?;

    // Tăng version sau khi apply thành công
    let mut lock = state.current_version.lock().await;
    *lock += 1;

    Ok(Json(serde_json::json!({
        "status": "pending_confirmation",
        "execution_id": execution.execution_id.as_str(),
        "new_version": *lock,
        "warning_count": warning_count,
        "timeout_seconds": DEFAULT_TIMEOUT_SECS
    })))
}

/// Handler `POST /v1/firewall/confirm`: Xác nhận Transaction Safe Apply
pub async fn confirm_policy_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ConfirmPayload>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    let execution_id = ExecutionId(payload.execution_id.clone());
    state.safe_apply.confirm(&execution_id).await?;

    Ok(Json(serde_json::json!({
        "status": "confirmed",
        "execution_id": payload.execution_id
    })))
}

/// Handler `POST /v1/firewall/rollback`: Hoàn tác Transaction Safe Apply
pub async fn rollback_policy_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RollbackPayload>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    GLOBAL_METRICS.inc_rollout_failure();
    let execution_id = ExecutionId(payload.execution_id.clone());
    state.safe_apply.rollback(&execution_id).await?;

    Ok(Json(serde_json::json!({
        "status": "rolled_back",
        "execution_id": payload.execution_id
    })))
}

/// Handler `GET /v1/audit`: Truy vấn 50 bản ghi Audit Log gần nhất từ SQLite
pub async fn get_audit_logs_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    let logs = state.repository.list_audits(50).await?;
    Ok(Json(logs))
}

/// Handler `GET /v1/docker/exposure`: Kiểm tra cổng công khai của Docker containers qua Unix socket
pub async fn get_docker_exposure_handler() -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    let inspector = DockerInspector::default_prod();
    let report = inspector.inspect().await?;
    Ok(Json(report))
}

/// Handler `POST /v1/router/forwarding`: Bật/tắt IP Forwarding trong kernel via sysctl
pub async fn set_router_forwarding_handler(
    Json(payload): Json<ForwardingPayload>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    let snapshot = RouterManager::set_ip_forwarding(payload.enabled).await?;
    Ok(Json(serde_json::json!({
        "status": "updated",
        "ip_forwarding": payload.enabled,
        "previousIpv4Forward": snapshot.old_ipv4_forward
    })))
}
