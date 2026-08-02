// Controller REST API Router cho `aegisnode server`
// Cung cấp các API quản trị tập trung Multi-Node: Authentication, Node Management, Enrollment, mTLS Heartbeats, Node Inventories, Network Profiles, Systemd Services, Combined Rollouts & HA Health Probes (Phase 23)

use std::sync::Arc;

use aegis_config::ControllerConfig;
use aegis_storage::PgRepository;
use axum::Router;
use axum::extract::{Json, State};
use axum::middleware;
use axum::routing::{get, patch, post};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::require_auth_middleware;
use crate::enrollment::{
    create_enrollment_token_handler, node_heartbeat_handler, sign_agent_csr_handler,
};
use crate::ha_status::{
    ha_status_handler, health_check_handler, readiness_check_handler,
};
use crate::inventory_router::{get_node_inventory_handler, report_node_inventory_handler};
use crate::network_router::{create_network_profile_handler, list_network_profiles_handler};
use crate::rollout_router::{
    cancel_rollout_handler, create_rollout_handler, get_rollout_status_handler,
    pause_rollout_handler, resume_rollout_handler, rollback_rollout_handler,
};
use crate::systemd_router::{execute_service_op_handler, query_journal_logs_handler};

/// Controller App State chứa PgRepository và ControllerConfig
#[derive(Clone)]
pub struct ControllerState {
    pub repository: Option<PgRepository>,
    pub config: ControllerConfig,
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

/// Handlers cho Controller API
pub async fn login_handler(
    State(state): State<Arc<ControllerState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, axum::http::StatusCode> {
    if req.username.is_empty() || req.password_hash.is_empty() {
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    Ok(Json(LoginResponse {
        token: format!("aegis_token_{}", Uuid::new_v4().simple()),
        expires_in_seconds: state.config.server.session_ttl_seconds,
    }))
}

pub async fn list_nodes_handler(
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

pub async fn enroll_node_handler(
    State(state): State<Arc<ControllerState>>,
    Json(req): Json<NodeEnrollRequest>,
) -> Result<Json<serde_json::Value>, axum::http::StatusCode> {
    if let Some(repo) = &state.repository {
        let record = repo
            .upsert_node(&req.hostname, &req.ip_address, &req.labels, &req.version)
            .await
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(serde_json::json!({
            "status": "ENROLLED",
            "nodeId": record.id,
            "registeredAt": record.created_at
        })))
    } else {
        Ok(Json(serde_json::json!({
            "status": "ENROLLED_NO_DB",
            "nodeId": Uuid::new_v4()
        })))
    }
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
        .route("/v1/nodes", get(list_nodes_handler))
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
        .layer(middleware::from_fn(require_auth_middleware));

    public_routes.merge(protected_routes).with_state(state)
}
