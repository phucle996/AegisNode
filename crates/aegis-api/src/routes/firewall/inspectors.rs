// Quản lý Docker Container Exposure và Router Forwarding Control Handlers

use aegis_core::AegisError;
use aegis_firewall::{DockerInspector, RouterManager};
use aegis_observability::prometheus::GLOBAL_METRICS;
use axum::extract::Json;
use axum::response::IntoResponse;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ForwardingPayload {
    pub enabled: bool,
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
