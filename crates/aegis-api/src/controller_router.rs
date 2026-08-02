// Controller REST API Router cho `aegisnode server`
// Cung cấp các API quản trị tập trung Multi-Node: Authentication, Node Management, Enrollment, mTLS Heartbeats, Node Inventories, Network Profiles, Systemd Services, Combined Rollouts & HA Health Probes (Phase 23)

use std::sync::Arc;

use aegis_config::ControllerConfig;
use aegis_core::pki::PkiManager;
use aegis_storage::PgRepository;
use axum::Router;
use axum::extract::{Json, State};
use axum::middleware;
use axum::routing::{get, patch, post};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::middleware::auth::parse_bearer_token_middleware;
use crate::routes::*;

/// Controller App State chứa PgRepository, ControllerConfig và PkiManager
#[derive(Clone)]
pub struct ControllerState {
    pub repository: Option<PgRepository>,
    pub config: ControllerConfig,
    pub pki_manager: PkiManager,
    /// Trạng thái leader election từ LeaderElector (None = luôn là leader)
    pub is_leader: bool,
}

/// Request Payload cho Login API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password_hash: String,
}

/// Response Payload cho Login API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token: String,
    pub expires_in_seconds: u64,
}

/// Request Payload cho Node Heartbeat/Enrollment API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEnrollRequest {
    pub hostname: String,
    pub ip_address: String,
    pub version: String,
    pub labels: serde_json::Value,
}

/// Request Payload cho Change Plan Creation API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChangePlanRequest {
    pub policy_id: Uuid,
    pub target_group_id: Option<Uuid>,
}

/// Handler `POST /v1/auth/login`: Xác thực credential và cấp API Token thực
/// Token được tạo ngẫu nhiên, hash bằng SHA-256 rồi lưu vào DB để xác minh sau
pub async fn login_handler(
    State(state): State<Arc<ControllerState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, axum::http::StatusCode> {
    if req.username.is_empty() || req.password_hash.is_empty() {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    // Xác minh password hash so sánh với HMAC của auth_secret từ config
    // auth_secret là thông tin bí mật được nạp từ environment/config file
    let expected_hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(state.config.server.auth_secret.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    if req.password_hash != expected_hash {
        return Err(axum::http::StatusCode::UNAUTHORIZED);
    }

    // Tạo token ngẫu nhiên và lưu hash vào DB để xác thực sau này
    let raw_token = Uuid::new_v4().to_string();
    let token_hash = {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        format!("{:x}", hasher.finalize())
    };

    if let Some(repo) = &state.repository {
        repo.create_api_token(&req.username, &token_hash)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    } else {
        // Không có DB thì không thể lưu token an toàn
        return Err(axum::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    Ok(Json(LoginResponse {
        token: raw_token,
        expires_in_seconds: state.config.server.session_ttl_seconds,
    }))
}

pub async fn controller_list_nodes_handler(
    State(state): State<Arc<ControllerState>>,
) -> Result<Json<Vec<aegis_storage::NodeRecord>>, axum::http::StatusCode> {
    if let Some(repo) = &state.repository {
        let nodes = repo
            .list_nodes()
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(nodes))
    } else {
        Ok(Json(vec![]))
    }
}

/// Handler `POST /v1/nodes/enroll`: Đăng ký Node vào Cluster (yêu cầu có DB)
pub async fn enroll_node_handler(
    State(state): State<Arc<ControllerState>>,
    Json(req): Json<NodeEnrollRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    let repo = state
        .repository
        .as_ref()
        .ok_or(axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

    let record = repo
        .upsert_node(&req.hostname, &req.ip_address, &req.labels, &req.version)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "status": "ENROLLED",
        "nodeId": record.id,
        "registeredAt": record.created_at
    })))
}

/// Xây dựng Axum Router ứng dụng AegisNode Controller Server (`aegisnode server`)
pub fn create_controller_router(state: Arc<ControllerState>) -> Router {
    // 1. Routes công khai cho Load Balancer Probes & Login (Không yêu cầu mTLS token)
    let public_routes = Router::new()
        .route("/health", get(health_check_handler))
        .route("/readiness", get(readiness_check_handler))
        .route("/v1/ha/status", get(ha_status_handler))
        .route("/v1/auth/login", post(login_handler));

    // 2. Routes yêu cầu Authentication middleware
    let protected_routes = Router::new()
        .route("/v1/nodes", get(controller_list_nodes_handler))
        .route("/v1/nodes/enroll", post(enroll_node_handler))
        .route(
            "/v1/nodes/:id/inventory",
            post(report_node_inventory_handler).get(get_node_inventory_handler),
        )
        .route(
            "/v1/network/profiles",
            get(list_network_profiles_handler).post(create_network_profile_handler),
        )
        .route(
            "/v1/nodes/:id/services/op",
            post(execute_service_op_handler),
        )
        .route(
            "/v1/nodes/:id/services/logs",
            get(query_journal_logs_handler),
        )
        .route("/v1/rollouts", post(create_rollout_handler))
        .route("/v1/rollouts/:id", get(get_rollout_status_handler))
        // Phase 18: Rollout Control (Pause / Resume / Cancel / Rollback)
        .route("/v1/rollouts/:id/pause", patch(pause_rollout_handler))
        .route("/v1/rollouts/:id/resume", patch(resume_rollout_handler))
        .route("/v1/rollouts/:id/cancel", patch(cancel_rollout_handler))
        .route("/v1/rollouts/:id/rollback", patch(rollback_rollout_handler))
        .route(
            "/v1/enrollment/token/create",
            post(create_enrollment_token_handler),
        )
        .route("/v1/enrollment/sign", post(sign_agent_csr_handler))
        .route("/v1/nodes/heartbeat", post(node_heartbeat_handler))
        .layer(middleware::from_fn(parse_bearer_token_middleware));

    public_routes.merge(protected_routes).with_state(state)
}
