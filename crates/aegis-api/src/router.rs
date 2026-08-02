//! Axum Router xây dựng các RESTful Routes, API Endpoints và Web UI Static Asset Serving
//! Phục vụ Web UI SPA tại `/` và API endpoints tại `/v1/*`

use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

use crate::routes::*;
use crate::state::AppState;

/// Xây dựng Axum Router ứng dụng AegisNode Local Agent API & Static Web UI
pub fn create_router(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        .route("/metrics", get(prometheus_metrics_handler))
        .route("/v1/status", get(get_status_handler))
        .route("/v1/firewall/policy", get(get_policy_handler))
        .route("/v1/firewall/validate", post(validate_policy_handler))
        .route("/v1/firewall/apply", post(apply_policy_handler))
        .route("/v1/firewall/confirm", post(confirm_policy_handler))
        .route("/v1/firewall/rollback", post(rollback_policy_handler))
        .route("/v1/audit", get(get_audit_logs_handler))
        .route("/v1/docker/exposure", get(get_docker_exposure_handler))
        .route("/v1/router/forwarding", post(set_router_forwarding_handler))
        .route("/v1/blocker/entries", get(get_blocker_entries_handler))
        .route("/v1/blocker/add", post(add_block_entry_handler))
        .route("/v1/blocker/remove", post(remove_block_entry_handler))
        .with_state(state);

    // Phục vụ Web UI static assets từ `web/dist` với SPA fallback về index.html
    let serve_dir =
        ServeDir::new("web/dist").not_found_service(ServeFile::new("web/dist/index.html"));

    api_routes.fallback_service(serve_dir)
}
