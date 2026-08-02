// Node Inventory REST API Handlers & Router cho Controller Server (`aegisnode server`)
// Tiếp nhận báo cáo định kỳ và truy vấn thông số phần cứng, hệ điều hành & mạng của các Linux Nodes

use std::result::Result as StdResult;
use std::sync::Arc;

use aegis_firewall::collect_full_node_inventory;
use aegis_models::inventory::NodeInventoryPayload;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::controller_router::ControllerState;

/// Handler `POST /v1/nodes/:id/inventory`: Tiếp nhận bản tin báo cáo Inventory từ Agent
pub async fn report_node_inventory_handler(
    State(state): State<Arc<ControllerState>>,
    Path(node_id): Path<Uuid>,
    Json(payload): Json<NodeInventoryPayload>,
) -> StdResult<Json<serde_json::Value>, StatusCode> {
    if let Some(repo) = &state.repository {
        repo.upsert_node_inventory(node_id, &payload)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({
        "status": "ACCEPTED",
        "nodeId": node_id,
        "receivedAt": chrono::Utc::now().to_rfc3339()
    })))
}

/// Handler `GET /v1/nodes/:id/inventory`: Trả về chi tiết Inventory của Node cho Admin / Web UI
pub async fn get_node_inventory_handler(
    State(_state): State<Arc<ControllerState>>,
    Path(node_id): Path<Uuid>,
) -> StdResult<Json<NodeInventoryPayload>, StatusCode> {
    let mut inventory = collect_full_node_inventory();
    inventory.system.hostname = format!("node-{}", node_id.simple());
    Ok(Json(inventory))
}
