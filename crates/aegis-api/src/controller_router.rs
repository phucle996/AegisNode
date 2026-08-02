// Controller REST API Router cho `aegisnode server`
// Cung cấp các API quản trị tập trung Multi-Node: Authentication, Node Management, Enrollment, mTLS Heartbeats, Node Inventories, Network Profiles, Systemd Services, Combined Rollouts & HA Health Probes (Phase 23)

use std::sync::Arc;

use aegis_config::ControllerConfig;
use aegis_core::pki::PkiManager;
use aegis_storage::PgRepository;
use axum::Router;
use axum::extract::{Extension, Json, State};
use axum::middleware;
use axum::routing::{get, patch, post};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::middleware::auth::{
    DEFAULT_JWT_SECRET, check_permission_middleware, parse_bearer_token_middleware,
};
use crate::middleware::jwt_provider::JwtProvider;
use crate::middleware::pam_auth::PamAuthenticator;
use crate::routes::*;

/// Controller App State chứa PgRepository, ControllerConfig và PkiManager
#[derive(Clone)]
pub struct ControllerState {
    pub repository: Option<PgRepository>,
    pub config: ControllerConfig,
    pub pki_manager: PkiManager,
    // Trạng thái cờ bầu chọn leader đồng bộ luồng Arc<AtomicBool>
    pub is_leader: Arc<std::sync::atomic::AtomicBool>,
}

impl ControllerState {
    // Phương thức trợ giúp đọc cờ is_leader hiện tại
    pub fn is_leader(&self) -> bool {
        self.is_leader.load(std::sync::atomic::Ordering::SeqCst)
    }
}

/// Request Payload cho Login API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    // Tên tài khoản Linux OS
    pub username: String,
    // Mật khẩu truy cập tài khoản Linux OS (Plaintext string được gửi an toàn qua TLS/HTTPS)
    pub password: String,
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

/// Handler `POST /v1/auth/login`: Xác thực tài khoản Linux OS (PAM/Cockpit style) và cấp JWT Token
/// JWT Payload được inject danh sách Roles & Permissions (`object:behavior`) dựa trên Linux Groups
pub async fn login_handler(
    State(state): State<Arc<ControllerState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, axum::http::StatusCode> {
    // Kiểm tra tên đăng nhập và mật khẩu không được phép trống
    if req.username.is_empty() || req.password.is_empty() {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    // 1. Xác thực tài khoản Linux OS qua PAM / Shadow password hash verification thực sự
    let groups = PamAuthenticator::authenticate(&req.username, &req.password)
        .map_err(|_| axum::http::StatusCode::UNAUTHORIZED)?;

    // 2. Ánh xạ từ Linux Groups sang Roles & Permissions list
    let (roles, permissions) = PamAuthenticator::map_groups_to_permissions(&groups);

    // 3. Đóng gói Claims và Ký số JWT Token
    let secret = &state.config.server.auth_secret;
    let effective_secret = if secret.is_empty() {
        DEFAULT_JWT_SECRET
    } else {
        secret
    };

    let ttl = state.config.server.session_ttl_seconds;
    let claims = JwtProvider::issue_token(&req.username, roles, permissions, effective_secret, ttl)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let jwt_token = JwtProvider::encode_claims(&claims, effective_secret)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse {
        token: jwt_token,
        expires_in_seconds: ttl,
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

/// Helper tạo sub-router cho 1 endpoint có gắn Extension permission string và middleware kiểm tra RBAC
fn perm_route(
    path: &str,
    method_router: axum::routing::MethodRouter<Arc<ControllerState>>,
    perm: &'static str,
) -> Router<Arc<ControllerState>> {
    // Tạo sub-router với route_layer kiểm tra permission
    Router::new()
        .route(path, method_router)
        .route_layer(middleware::from_fn(check_permission_middleware))
        .route_layer(Extension(perm))
}

/// Xây dựng Axum Router ứng dụng AegisNode Controller Server (`aegisnode server`)
pub fn create_controller_router(state: Arc<ControllerState>) -> Router {
    // 1. Routes công khai cho Load Balancer Probes & Login (Không yêu cầu mTLS token)
    let public_routes = Router::new()
        .route("/health", get(health_check_handler))
        .route("/readiness", get(readiness_check_handler))
        .route("/v1/ha/status", get(ha_status_handler))
        .route("/v1/auth/login", post(login_handler));

    // 2. Routes yêu cầu Authentication & RBAC middleware kiểm tra quyền hạn (object:behavior)
    let protected_routes = Router::new()
        // Route truy vấn danh sách Node: Yêu cầu quyền `nodes:read`
        .merge(perm_route("/v1/nodes", get(controller_list_nodes_handler), "nodes:read"))
        // Route đăng ký Node vào Cluster: Yêu cầu quyền `nodes:write`
        .merge(perm_route("/v1/nodes/enroll", post(enroll_node_handler), "nodes:write"))
        // Route tiếp nhận và đọc Node Inventory: Yêu cầu quyền `nodes:write` cho POST và `nodes:read` cho GET
        .merge(perm_route("/v1/nodes/:id/inventory", post(report_node_inventory_handler), "nodes:write"))
        .merge(perm_route("/v1/nodes/:id/inventory", get(get_node_inventory_handler), "nodes:read"))
        // Route xem và tạo Network Profiles: Yêu cầu quyền `network:read` cho GET và `network:write` cho POST
        .merge(perm_route("/v1/network/profiles", get(list_network_profiles_handler), "network:read"))
        .merge(perm_route("/v1/network/profiles", post(create_network_profile_handler), "network:write"))
        // Route điều khiển Systemd Unit: Yêu cầu quyền `service:restart`
        .merge(perm_route("/v1/nodes/:id/services/op", post(execute_service_op_handler), "service:restart"))
        // Route xem Journald logs: Yêu cầu quyền `service:read`
        .merge(perm_route("/v1/nodes/:id/services/logs", get(query_journal_logs_handler), "service:read"))
        // Route phát hành và kiểm tra tiến độ Rollout Plan: Yêu cầu quyền `rollout:manage`
        .merge(perm_route("/v1/rollouts", post(create_rollout_handler), "rollout:manage"))
        .merge(perm_route("/v1/rollouts/:id", get(get_rollout_status_handler), "rollout:manage"))
        // Phase 18: Rollout Control (Pause / Resume / Cancel / Rollback) — Yêu cầu quyền `rollout:manage`
        .merge(perm_route("/v1/rollouts/:id/pause", patch(pause_rollout_handler), "rollout:manage"))
        .merge(perm_route("/v1/rollouts/:id/resume", patch(resume_rollout_handler), "rollout:manage"))
        .merge(perm_route("/v1/rollouts/:id/cancel", patch(cancel_rollout_handler), "rollout:manage"))
        .merge(perm_route("/v1/rollouts/:id/rollback", patch(rollback_rollout_handler), "rollout:manage"))
        // Route sinh Enrollment Token: Yêu cầu quyền `admin:manage`
        .merge(perm_route("/v1/enrollment/token/create", post(create_enrollment_token_handler), "admin:manage"))
        // Route Heartbeat định kỳ: Yêu cầu quyền `nodes:write`
        .merge(perm_route("/v1/nodes/heartbeat", post(node_heartbeat_handler), "nodes:write"))
        // Route mTLS CSR Signing công khai (Agent đăng ký lần đầu với Enrollment Token)
        .route("/v1/enrollment/sign", post(sign_agent_csr_handler))
        // Layer xác thực Bearer JWT Token được áp dụng cho toàn bộ các protected routes
        .layer(middleware::from_fn_with_state(
            state.clone(),
            parse_bearer_token_middleware,
        ));

    public_routes
        .merge(protected_routes)
        .with_state(state)
        .fallback(crate::router::static_asset_handler)
}
