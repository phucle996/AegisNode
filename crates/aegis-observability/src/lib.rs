//! AegisNode Observability Crate
//! Khởi tạo Tracing Subscriber, JSON structured logging cho Production và human-readable cho Dev.
//! Xuất chỉ số Prometheus cho Observability.

pub mod prometheus;

use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Khởi tạo hệ thống Logging dựa trên môi trường RUST_LOG
pub fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,aegisnode=debug"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
