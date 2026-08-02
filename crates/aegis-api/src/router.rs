// Axum Router xây dựng các RESTful Routes, API Endpoints và Embedded Web UI Static Asset Serving
// Phục vụ Web UI SPA nhúng trực tiếp trong binary tại `/` và API endpoints tại `/v1/*`

use std::sync::Arc;

use axum::Router;
use axum::http::{HeaderValue, StatusCode, Uri, header};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use rust_embed::RustEmbed;

use crate::routes::*;
use crate::state::AppState;

/// Struct nhúng thư mục assets Web UI `web/dist` trực tiếp vào binary lúc compile
#[derive(RustEmbed)]
#[folder = "../../web/dist"]
pub struct WebAssets;

/// Handler phục vụ file tĩnh và SPA Fallback index.html từ bộ nhớ nhúng
pub async fn static_asset_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/');
    if path.is_empty() {
        path = "index.html";
    }

    match WebAssets::get(path) {
        Some(content) => {
            // Xác định MIME type phù hợp dựa vào đuôi mở rộng của file
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            let header_val = HeaderValue::from_str(mime.as_ref())
                .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream"));

            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, header_val)],
                content.data,
            )
                .into_response()
        }
        None => {
            // SPA Fallback: Trả về index.html cho mọi đường dẫn Client-side routing
            match WebAssets::get("index.html") {
                Some(content) => (
                    StatusCode::OK,
                    [(header::CONTENT_TYPE, HeaderValue::from_static("text/html"))],
                    content.data,
                )
                    .into_response(),
                None => (StatusCode::NOT_FOUND, "Web UI Assets Not Embedded").into_response(),
            }
        }
    }
}

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

    // Gắn handler static_asset_handler làm fallback service cho mọi route frontend
    api_routes.fallback(static_asset_handler)
}
