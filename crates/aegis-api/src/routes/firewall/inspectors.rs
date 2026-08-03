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

use crate::controller_router::ControllerState;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallRuleSyncItem {
    pub chain: String,
    pub rule_id: String,
    pub protocol: String,
    pub src_cidr: String,
    pub dst_cidr: String,
    pub port_spec: String,
    pub action: String,
    pub packets: i64,
    pub bytes: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallSyncPayload {
    pub node_id: Uuid,
    pub rules: Vec<FirewallRuleSyncItem>,
}

#[derive(Debug, Deserialize)]
pub struct QueryNodeParams {
    pub node_id: Option<Uuid>,
}

/// Handler `POST /v1/nodes/firewall/sync`: Nhận báo cáo trạng thái luật Kernel `nftables` thực tế từ Agent
pub async fn sync_node_firewall_rules_handler(
    State(state): State<Arc<ControllerState>>,
    Json(payload): Json<FirewallSyncPayload>,
) -> Result<impl IntoResponse, StatusCode> {
    GLOBAL_METRICS.inc_http_requests();
    // Ghi nhận toàn bộ các luật nftables thực tế gửi từ Linux Agent vào CSDL
    if let Some(repo) = &state.repository {
        for rule in &payload.rules {
            let _ = repo
                .upsert_node_firewall_rule(
                    payload.node_id,
                    &rule.chain,
                    &rule.rule_id,
                    &rule.protocol,
                    &rule.src_cidr,
                    &rule.dst_cidr,
                    &rule.port_spec,
                    &rule.action,
                    rule.packets,
                    rule.bytes,
                )
                .await;
        }
    }
    Ok(Json(serde_json::json!({
        "status": "synced",
        "nodeId": payload.node_id,
        "ruleCount": payload.rules.len()
    })))
}

/// Handler `GET /v1/firewall/rules`: Trả về danh sách luật tường lửa Kernel thực tế từ CSDL
pub async fn get_live_firewall_rules_handler(
    State(state): State<Arc<ControllerState>>,
    Query(params): Query<QueryNodeParams>,
) -> Result<impl IntoResponse, StatusCode> {
    GLOBAL_METRICS.inc_http_requests();
    // Truy vấn danh sách bản ghi luật firewall thực tế được lưu trong PostgreSQL CSDL
    if let Some(repo) = &state.repository {
        if let Ok(records) = repo.list_live_firewall_rules(params.node_id).await {
            return Ok(Json(serde_json::to_value(records).unwrap_or_default()));
        }
    }

    Ok(Json(serde_json::json!([])))
}
