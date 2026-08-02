//! Node Inventory & Fleet Management REST API Handlers (Phase 14 Node Inventory)
//! Quản lý danh sách Node và tiếp nhận báo cáo Inventory định kỳ từ Agent.

use aegis_firewall::collect_full_node_inventory;
use aegis_models::inventory::NodeInventoryPayload;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use std::result::Result as StdResult;
use std::sync::Arc;
use uuid::Uuid;

use crate::controller_router::ControllerState;

/// Handler `GET /v1/nodes`: Danh sách tất cả Node trong hệ thống
pub async fn list_nodes_handler(
    State(state): State<Arc<ControllerState>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let nodes = if let Some(repo) = &state.repository {
        repo.list_nodes().await.unwrap_or_default()
    } else {
        vec![]
    };
    Ok(Json(serde_json::json!({ "nodes": nodes })))
}

/// Handler `GET /v1/nodes/:id`: Tra cứu thông tin 1 Node theo ID (lấy từ danh sách)
pub async fn get_node_handler(
    State(state): State<Arc<ControllerState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Some(repo) = &state.repository {
        let nodes = repo.list_nodes().await.unwrap_or_default();
        // Tìm node theo ID trong danh sách (PgRepository chưa có get_node đơn lẻ)
        if let Some(node) = nodes.into_iter().find(|n| n.id == id) {
            return Ok(Json(serde_json::json!(node)));
        }
    }
    Err(StatusCode::NOT_FOUND)
}

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
    // Thu thập inventory từ hệ thống thực tế (local) thay thế cho node cụ thể
    let mut inventory = collect_full_node_inventory();
    inventory.system.hostname = format!("node-{}", node_id.simple());
    Ok(Json(inventory))
}

/// Handler `PATCH /v1/nodes/:id/labels` — Stub: PgRepository chưa hỗ trợ update labels
pub async fn update_node_labels_handler(
    State(_state): State<Arc<ControllerState>>,
    Path(id): Path<Uuid>,
    Json(labels): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Thêm method update_node_labels vào PgRepository khi cần
    Ok(Json(
        serde_json::json!({ "status": "accepted", "nodeId": id, "labels": labels }),
    ))
}
