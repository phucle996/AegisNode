//! Auto-Blocker & Dynamic IP Set REST API Handlers
//! Quản lý xem danh sách IP bị chặn, thêm hoặc xóa IP khỏi danh sách khóa tự động qua nftables set.

use aegis_core::AegisError;
use aegis_observability::prometheus::GLOBAL_METRICS;
use axum::extract::{Json, State};
use axum::response::IntoResponse;
use serde::Deserialize;
use std::sync::Arc;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct BlockIpPayload {
    pub ip: String,
    pub reason: String,
    /// Thời gian chặn tính bằng giây — None = chặn vĩnh viễn
    pub duration_seconds: Option<u64>,
}

/// Handler `GET /v1/blocker/entries`: Lấy danh sách IP đang bị khóa (thực tế từ nftables set)
pub async fn get_blocker_entries_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    // Lấy lock để truy cập BlockManager (list_blocks cần &mut để dọn expired entries)
    let mut bm = state.block_manager.lock().await;
    let entries = bm.list_blocks();
    Ok(Json(entries))
}

/// Handler `POST /v1/blocker/add`: Thêm IP vào danh sách khóa động
pub async fn add_block_entry_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BlockIpPayload>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    let mut bm = state.block_manager.lock().await;
    // add_block signature: (ip, duration_seconds, reason, actor)
    // actor được đặt là "api" vì request đến từ REST API
    bm.add_block(
        &payload.ip,
        payload.duration_seconds,
        &payload.reason,
        "api",
    )?;

    Ok(Json(serde_json::json!({
        "status": "blocked",
        "ip": payload.ip,
        "reason": payload.reason,
        "durationSeconds": payload.duration_seconds
    })))
}

/// Handler `POST /v1/blocker/remove`: Xóa IP khỏi danh sách khóa và nftables set
pub async fn remove_block_entry_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<BlockIpPayload>,
) -> Result<impl IntoResponse, AegisError> {
    GLOBAL_METRICS.inc_http_requests();
    let mut bm = state.block_manager.lock().await;
    bm.remove_block(&payload.ip)?;

    Ok(Json(serde_json::json!({
        "status": "unblocked",
        "ip": payload.ip
    })))
}
