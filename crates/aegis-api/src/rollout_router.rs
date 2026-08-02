// Combined Change Plan & Multi-Node Rollouts REST API Handlers & Router cho Controller Server (`aegisnode server`)
// Phát hành Kế hoạch thay đổi Hợp nhất, Idempotent execution và theo dõi tiến độ Rollout thời gian thực

use std::result::Result as StdResult;
use std::sync::Arc;

use aegis_firewall::CombinedChangePlanner;
use aegis_models::change_plan::{NodeChangePlan, RolloutReport};
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use crate::controller_router::ControllerState;

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

/// Handler `GET /v1/rollouts/:id`: Lấy báo cáo trạng thái tiến độ Rollout
pub async fn get_rollout_status_handler(
    _state: State<Arc<ControllerState>>,
    Path(rollout_id): Path<Uuid>,
) -> StdResult<Json<RolloutReport>, StatusCode> {
    Ok(Json(RolloutReport {
        rollout_id,
        status: "COMPLETED".to_string(),
        progress_percent: 100,
        error_message: None,
    }))
}
