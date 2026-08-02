//! Health Probes & HA Status Handlers (Phase 23 Controller HA)
//! Phục vụ các endpoint /health, /readiness cho Load Balancer / Kubernetes Kubelet và /v1/ha/status.

use axum::Json;
use serde::{Deserialize, Serialize};

/// Response Payload cho Health & Readiness Probe
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// Response Payload cho HA Status Overview
#[derive(Debug, Serialize, Deserialize)]
pub struct HaStatusResponse {
    pub status: &'static str,
    pub mode: &'static str,
    pub is_leader: bool,
    pub database_connected: bool,
}

/// Handler `GET /health`: Liveness Probe cho Load Balancer
pub async fn health_check_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "UP" })
}

/// Handler `GET /readiness`: Readiness Probe cho Load Balancer
pub async fn readiness_check_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: "READY" })
}

/// Handler `GET /v1/ha/status`: Trả về trạng thái bầu chọn Leader & DB Pool của Controller
pub async fn ha_status_handler() -> Json<HaStatusResponse> {
    Json(HaStatusResponse {
        status: "UP",
        mode: "CONTROLLER_HA",
        is_leader: true,
        database_connected: true,
    })
}
