//! Systemd Service Management REST API Handlers (Phase 16 Systemd Management)
//! Đọc danh sách unit thực từ `systemctl`, điều khiển vòng đời dịch vụ và truy vấn journald log.

use std::process::Command;
use std::result::Result as StdResult;
use std::sync::Arc;

use aegis_firewall::SystemdManager;
use aegis_models::systemd::{
    JournalLogEntry, ServiceOpRequest, ServiceOpResult, ServiceUnitStatus,
};
use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::controller_router::ControllerState;

#[derive(Debug, Deserialize)]
pub struct ServiceControlPayload {
    pub action: String,
}

#[derive(Debug, Deserialize)]
pub struct JournalQueryParam {
    pub unit: String,
    pub limit: Option<usize>,
}

/// Đọc danh sách tất cả running service từ `systemctl list-units --type=service --state=running`
/// Trả về dạng `Vec<ServiceUnitStatus>` với dữ liệu thực từ systemd D-Bus (qua CLI)
fn list_running_units() -> Vec<ServiceUnitStatus> {
    let output = Command::new("systemctl")
        .args([
            "list-units",
            "--type=service",
            "--state=running",
            "--no-pager",
            "--no-legend",
            "--plain",
        ])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            // Format: UNIT LOAD ACTIVE SUB DESCRIPTION
            stdout
                .lines()
                .filter_map(|line| {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 4 {
                        return None;
                    }
                    Some(ServiceUnitStatus {
                        name: parts[0].to_string(),
                        load_state: parts[1].to_string(),
                        active_state: parts[2].to_string(),
                        sub_state: parts[3].to_string(),
                        description: parts.get(4..).map(|d| d.join(" ")).unwrap_or_default(),
                    })
                })
                .collect()
        }
        _ => vec![],
    }
}

/// Handler `GET /v1/systemd/services`: Lấy danh sách dịch vụ đang chạy thực tế từ systemd
pub async fn list_systemd_services_handler() -> Result<Json<serde_json::Value>, StatusCode> {
    // Chạy trong thread pool để không block async runtime
    let units = tokio::task::spawn_blocking(list_running_units)
        .await
        .unwrap_or_default();

    Ok(Json(serde_json::json!({ "services": units })))
}

/// Handler `POST /v1/systemd/services/:name/control`: Thực thi điều khiển dịch vụ systemd
pub async fn control_systemd_service_handler(
    Path(service_name): Path<String>,
    Json(payload): Json<ServiceControlPayload>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let manager = SystemdManager::new();
    // Parse ServiceOperation từ chuỗi action qua serde_json (SCREAMING_SNAKE_CASE)
    let action_json = format!("\"{}\"", payload.action.to_uppercase());
    let op: aegis_models::systemd::ServiceOperation =
        serde_json::from_str(&action_json).map_err(|_| StatusCode::BAD_REQUEST)?;
    let req = ServiceOpRequest {
        unit_name: service_name.clone(),
        operation: op,
        reason: None,
    };
    let result = manager.execute_op(&req).map_err(|e| {
        tracing::warn!("systemd op failed: {e}");
        match e {
            aegis_core::AegisError::Permission(_) => StatusCode::FORBIDDEN,
            aegis_core::AegisError::Validation(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    })?;

    Ok(Json(serde_json::json!({
        "status": if result.success { "EXECUTED" } else { "FAILED" },
        "serviceName": service_name,
        "action": payload.action,
        "executionTimeMs": result.execution_time_ms,
        "message": result.message
    })))
}

/// Handler `POST /v1/nodes/:id/services/op`: Thực thi thao tác điều khiển Systemd Unit trên remote node
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
