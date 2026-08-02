//! Health & High Availability Router Handlers (Phase 23 Controller HA)
//! Liveness/Readiness probes và HA status đọc từ ControllerConfig thực tế.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::sync::Arc;

use crate::controller_router::ControllerState;

/// Liveness Probe `/health` — Kubernetes / Load Balancer health check
pub async fn health_check_handler() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Readiness Probe `/readiness` — chỉ trả READY khi DB connection sẵn sàng
pub async fn readiness_check_handler(
    State(state): State<Arc<ControllerState>>,
) -> impl IntoResponse {
    // Kiểm tra DB connection pool thực tế nếu có
    if let Some(repo) = &state.repository {
        // Chạy ping query đơn giản để xác nhận DB reachable
        if repo.pool().acquire().await.is_err() {
            return (StatusCode::SERVICE_UNAVAILABLE, "DB_UNAVAILABLE");
        }
    }
    (StatusCode::OK, "READY")
}

/// High Availability Status `/v1/ha/status` — trả dữ liệu thực từ ControllerState
pub async fn ha_status_handler(
    State(state): State<Arc<ControllerState>>,
) -> Result<axum::Json<serde_json::Value>, StatusCode> {
    // Đọc giá trị cờ Leader hiện tại từ AtomicBool của ControllerState
    let is_leader = state.is_leader();

    // Lấy leader election backend từ config nếu có; mặc định là advisory lock
    let leader_election_backend = if state.config.database.url.is_empty() {
        "STANDALONE"
    } else {
        "POSTGRESQL_ADVISORY_LOCK"
    };

    Ok(axum::Json(serde_json::json!({
        "status": "UP",
        "role": if is_leader { "LEADER" } else { "FOLLOWER" },
        "leaderElection": leader_election_backend,
        "controllerHost": state.config.server.host,
        "controllerPort": state.config.server.port,
    })))
}
