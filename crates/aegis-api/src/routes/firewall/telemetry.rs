// Phục vụ Prometheus Metrics, Status Check và Audit Logs Endpoints

use aegis_core::AegisError;
use aegis_observability::prometheus::GLOBAL_METRICS;
use aegis_storage::AuditRepository;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::sync::Arc;

use crate::state::AppState;

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

/// Handler `GET /v1/audit`: Truy vấn 50 bản ghi Audit Log gần nhất từ SQLite
pub async fn get_audit_logs_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    let logs = state.repository.list_audits(50).await?;
    Ok(Json(logs))
}
