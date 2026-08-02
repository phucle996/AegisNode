// Controller REST API Router cho `aegisnode server`
// Cung cấp các API quản trị tập trung Multi-Node: Authentication, Node Management, Enrollment & mTLS Heartbeats

use std::sync::Arc;

use aegis_config::ControllerConfig;
use aegis_storage::PgRepository;
use axum::Router;
use axum::extract::{Json, State};
use axum::middleware;
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::require_auth_middleware;
use crate::enrollment::{
    create_enrollment_token_handler, node_heartbeat_handler, sign_agent_csr_handler,
};

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
    Router::new()
        .route("/v1/auth/login", post(login_handler))
        .route("/v1/nodes", get(list_nodes_handler))
        .route("/v1/nodes/enroll", post(enroll_node_handler))
        .route(
            "/v1/enrollment/token/create",
            post(create_enrollment_token_handler),
        )
        .route("/v1/enrollment/sign", post(sign_agent_csr_handler))
        .route("/v1/nodes/heartbeat", post(node_heartbeat_handler))
        .layer(middleware::from_fn(require_auth_middleware))
        .with_state(state)
}
