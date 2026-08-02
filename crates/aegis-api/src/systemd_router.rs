// Systemd Management REST API Handlers & Router cho Controller Server (`aegisnode server`)
// Xử lý thực thi điều khiển dịch vụ Linux Systemd có định kiểu, bảo vệ Protected Units và truy vấn nhật ký Journald Logs

use std::result::Result as StdResult;
use std::sync::Arc;

use aegis_firewall::SystemdManager;
use aegis_models::systemd::{JournalLogEntry, ServiceOpRequest, ServiceOpResult};
use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::controller_router::ControllerState;

/// Query Parameters cho Journald Logs API
#[derive(Debug, Deserialize)]
pub struct JournalQueryParam {
    pub unit: String,
    pub limit: Option<usize>,
}

/// Handler `POST /v1/nodes/:id/services/op`: Thực thi thao tác điều khiển Systemd Unit
pub async fn execute_service_op_handler(
    _state: State<Arc<ControllerState>>,
    Path(_node_id): Path<Uuid>,
    Json(req): Json<ServiceOpRequest>,
) -> StdResult<Json<ServiceOpResult>, StatusCode> {
    let manager = SystemdManager::new();
    let result = manager.execute_op(&req).map_err(|e| match e {
        aegis_core::AegisError::Permission(_) => StatusCode::FORBIDDEN,
        aegis_core::AegisError::Validation(_) => StatusCode::BAD_REQUEST,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    })?;

    Ok(Json(result))
}

/// Handler `GET /v1/nodes/:id/services/logs`: Truy vấn Journald Logs của Unit
pub async fn query_journal_logs_handler(
    _state: State<Arc<ControllerState>>,
    Path(_node_id): Path<Uuid>,
    Query(param): Query<JournalQueryParam>,
) -> StdResult<Json<Vec<JournalLogEntry>>, StatusCode> {
    let manager = SystemdManager::new();
    let limit = param.limit.unwrap_or(50);
    let logs = manager
        .query_journal_logs(&param.unit, limit)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    Ok(Json(logs))
}
