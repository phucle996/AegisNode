//! AegisNode API Crate
//! Khai báo các API Route Handlers cho cả HTTP và Unix Socket Interface.

use axum::{Router, routing::get};

/// Tạo API router mặc định cho local agent
pub fn create_router() -> Router {
    Router::new().route("/v1/health", get(|| async { "OK" }))
}
