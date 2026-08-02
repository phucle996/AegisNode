// Combined Change Plan & Multi-Node Rollouts REST API Handlers & Router cho Controller
// Phát hành Change Plan, điều phối Multi-Node Rollout với Canary/Batch/AllAtOnce/Manual
// Phase 17: create_rollout, get_rollout_status
// Phase 18: pause, resume, cancel, rollback controls + get targets

use std::result::Result as StdResult;
use std::sync::Arc;

use aegis_firewall::{CombinedChangePlanner, RolloutCoordinator};
use aegis_models::change_plan::{NodeChangePlan, RolloutReport};
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::controller_router::ControllerState;

// ─── Create Rollout ───────────────────────────────────────────────────────────

/// Handler `POST /v1/rollouts`: Tạo và phát hành Combined Change Plan cho Node
pub async fn create_rollout_handler(
    State(state): State<Arc<ControllerState>>,
    Json(mut plan): Json<NodeChangePlan>,
) -> StdResult<Json<serde_json::Value>, StatusCode> {
    let planner = CombinedChangePlanner::new();
    planner
        .plan_execution(&mut plan)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(repo) = &state.repository {
        repo.create_rollout(&plan)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({
        "status": "SCHEDULED",
        "rolloutId": plan.id,
        "idempotencyKey": plan.idempotency_key,
        "riskLevel": plan.risk_level,
        "stepCount": plan.steps.len()
    })))
}

// ─── Get Rollout Status ───────────────────────────────────────────────────────

/// Handler `GET /v1/rollouts/:id`: Lấy báo cáo trạng thái tiến độ Rollout bao gồm node statuses
pub async fn get_rollout_status_handler(
    State(state): State<Arc<ControllerState>>,
    Path(rollout_id): Path<Uuid>,
) -> StdResult<Json<serde_json::Value>, StatusCode> {
    // Lấy danh sách node targets từ PostgreSQL để hiển thị trạng thái từng node
    let node_statuses = if let Some(repo) = &state.repository {
        repo.get_rollout_targets(rollout_id)
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };

    let total = node_statuses.len();
    let succeeded = node_statuses
        .iter()
        .filter(|(_, s)| s == "SUCCEEDED")
        .count();
    let failed = node_statuses.iter().filter(|(_, s)| s == "FAILED").count();
    let pending = node_statuses.iter().filter(|(_, s)| s == "PENDING").count();
    let progress = if total > 0 {
        (succeeded * 100) / total
    } else {
        0
    };

    Ok(Json(serde_json::json!({
        "rolloutId": rollout_id,
        "status": if failed > 0 { "DEGRADED" } else if pending == 0 { "COMPLETED" } else { "RUNNING" },
        "progressPercent": progress,
        "totalNodes": total,
        "succeededNodes": succeeded,
        "failedNodes": failed,
        "pendingNodes": pending,
        "nodeStatuses": node_statuses.iter().map(|(id, s)| serde_json::json!({
            "nodeId": id,
            "state": s
        })).collect::<Vec<_>>()
    })))
}

// ─── Rollout Control Handlers (Phase 18) ─────────────────────────────────────

/// Handler `PATCH /v1/rollouts/:id/pause`: Tạm dừng Rollout đang chạy
pub async fn pause_rollout_handler(
    State(state): State<Arc<ControllerState>>,
    Path(rollout_id): Path<Uuid>,
) -> StdResult<Json<serde_json::Value>, StatusCode> {
    if let Some(repo) = &state.repository {
        repo.update_rollout_state(rollout_id, "PAUSED")
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(Json(serde_json::json!({
        "rolloutId": rollout_id,
        "status": "PAUSED",
        "message": "Rollout paused. Use /resume to continue."
    })))
}

/// Handler `PATCH /v1/rollouts/:id/resume`: Tiếp tục Rollout từ batch đang dở
pub async fn resume_rollout_handler(
    State(state): State<Arc<ControllerState>>,
    Path(rollout_id): Path<Uuid>,
) -> StdResult<Json<serde_json::Value>, StatusCode> {
    if let Some(repo) = &state.repository {
        repo.update_rollout_state(rollout_id, "RUNNING")
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(Json(serde_json::json!({
        "rolloutId": rollout_id,
        "status": "RUNNING",
        "message": "Rollout resumed from last batch checkpoint."
    })))
}

/// Handler `PATCH /v1/rollouts/:id/cancel`: Huỷ bỏ hoàn toàn Rollout
pub async fn cancel_rollout_handler(
    State(state): State<Arc<ControllerState>>,
    Path(rollout_id): Path<Uuid>,
) -> StdResult<Json<serde_json::Value>, StatusCode> {
    if let Some(repo) = &state.repository {
        repo.update_rollout_state(rollout_id, "CANCELLED")
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(Json(serde_json::json!({
        "rolloutId": rollout_id,
        "status": "CANCELLED",
        "message": "Rollout cancelled. No further nodes will be processed."
    })))
}

/// Handler `PATCH /v1/rollouts/:id/rollback`: Phát lệnh Rollback toàn fleet về version trước
pub async fn rollback_rollout_handler(
    State(state): State<Arc<ControllerState>>,
    Path(rollout_id): Path<Uuid>,
) -> StdResult<Json<serde_json::Value>, StatusCode> {
    // Lấy danh sách nodes cần rollback (reverse order: cuối cùng rollout → đầu tiên rollback)
    let rollback_targets = if let Some(repo) = &state.repository {
        let targets = repo
            .get_rollout_targets(rollout_id)
            .await
            .unwrap_or_default();
        // Lọc chỉ SUCCEEDED nodes và đảo thứ tự
        let mut succeeded: Vec<Uuid> = targets
            .iter()
            .filter(|(_, s)| s == "SUCCEEDED")
            .map(|(id, _)| *id)
            .collect();
        succeeded.reverse();
        succeeded
    } else {
        vec![]
    };

    if let Some(repo) = &state.repository {
        repo.update_rollout_state(rollout_id, "ROLLING_BACK")
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({
        "rolloutId": rollout_id,
        "status": "ROLLING_BACK",
        "rollbackTargets": rollback_targets,
        "message": format!("Rollback initiated for {} nodes in reverse order.", rollback_targets.len())
    })))
}
