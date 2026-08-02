//! AegisNode Observability Crate
//! Khởi tạo Tracing Subscriber, JSON structured logging cho Production, W3C Distributed Tracing, Prometheus Alerting Rules & Grafana Dashboards.

pub mod alert_rules;
pub mod grafana_dashboards;
pub mod prometheus;
pub mod tracing_context;

pub use alert_rules::{PrometheusAlertRule, generate_prometheus_rules_yaml};
pub use grafana_dashboards::{
    generate_firewall_activity_dashboard_json, generate_fleet_health_dashboard_json,
};
pub use tracing_context::W3cTraceContext;

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
