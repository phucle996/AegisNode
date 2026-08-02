// Axum Router xây dựng các RESTful Routes và Middleware chung
// Tích hợp Request ID, JSON Body Limit và Middleware định dạng lỗi chuẩn ErrorResponse

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use crate::handlers::{
    apply_policy_handler, confirm_policy_handler, get_audit_logs_handler, get_policy_handler,
    get_status_handler, rollback_policy_handler, validate_policy_handler,
};
use crate::state::AppState;

/// Xây dựng Axum Router ứng dụng AegisNode API
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/v1/status", get(get_status_handler))
        .route("/v1/firewall/policy", get(get_policy_handler))
        .route("/v1/firewall/validate", post(validate_policy_handler))
        .route("/v1/firewall/apply", post(apply_policy_handler))
        .route("/v1/firewall/confirm", post(confirm_policy_handler))
        .route("/v1/firewall/rollback", post(rollback_policy_handler))
        .route("/v1/audit", get(get_audit_logs_handler))
        .with_state(state)
}
