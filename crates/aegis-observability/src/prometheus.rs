//! AegisNode Prometheus Metrics Exporter
//! Phục vụ xuất các chỉ số giám sát hệ thống (Observability) cho Prometheus scraper.

use std::sync::atomic::{AtomicU64, Ordering};

/// Cấu trúc lưu trữ các chỉ số Prometheus cho AegisNode Controller & Agent
pub struct MetricsCollector {
    /// Số lượng HTTP Request đã xử lý
    pub http_requests_total: AtomicU64,
    /// Số lượng Agent hiện đang duy trì kết nối mTLS
    pub connected_agents: AtomicU64,
    /// Số lượng Rollout thất bại
    pub rollout_failures_total: AtomicU64,
    /// Số lượng Gói tin bị Firewall Drop
    pub firewall_drops_total: AtomicU64,
    /// Số lượng IP bị khóa bởi Auto-Blocker
    pub active_blocks_total: AtomicU64,
}

impl MetricsCollector {
    /// Khởi tạo Collector chỉ số mới
    pub const fn new() -> Self {
        Self {
            http_requests_total: AtomicU64::new(0),
            connected_agents: AtomicU64::new(0),
            rollout_failures_total: AtomicU64::new(0),
            firewall_drops_total: AtomicU64::new(0),
            active_blocks_total: AtomicU64::new(0),
        }
    }

    /// Tăng số lượng HTTP Request
    pub fn inc_http_requests(&self) {
        self.http_requests_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Cập nhật số lượng Agent đang kết nối
    pub fn set_connected_agents(&self, count: u64) {
        self.connected_agents.store(count, Ordering::Relaxed);
    }

    /// Ghi nhận 1 Rollout bị thất bại
    pub fn inc_rollout_failure(&self) {
        self.rollout_failures_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Ghi nhận gói tin bị Firewall chặn
    pub fn inc_firewall_drop(&self) {
        self.firewall_drops_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Cập nhật tổng số IP bị khóa
    pub fn set_active_blocks(&self, count: u64) {
        self.active_blocks_total.store(count, Ordering::Relaxed);
    }

    /// Render định dạng Prometheus Text Exposition Format (v0.0.4)
    pub fn render_prometheus_exposition(&self) -> String {
        let mut buffer = String::new();

        // 1. HTTP Requests Total Metric
        buffer.push_str("# HELP aegis_http_requests_total Total number of HTTP requests processed.\n");
        buffer.push_str("# TYPE aegis_http_requests_total counter\n");
        buffer.push_str(&format!("aegis_http_requests_total {}\n\n", self.http_requests_total.load(Ordering::Relaxed)));

        // 2. Connected Agents Metric
        buffer.push_str("# HELP aegis_connected_agents Number of mTLS connected agents.\n");
        buffer.push_str("# TYPE aegis_connected_agents gauge\n");
        buffer.push_str(&format!("aegis_connected_agents {}\n\n", self.connected_agents.load(Ordering::Relaxed)));

        // 3. Rollout Failures Metric
        buffer.push_str("# HELP aegis_rollout_failures_total Total failed multi-node rollouts.\n");
        buffer.push_str("# TYPE aegis_rollout_failures_total counter\n");
        buffer.push_str(&format!("aegis_rollout_failures_total {}\n\n", self.rollout_failures_total.load(Ordering::Relaxed)));

        // 4. Firewall Drops Metric
        buffer.push_str("# HELP aegis_firewall_drops_total Total packets dropped by nftables filter.\n");
        buffer.push_str("# TYPE aegis_firewall_drops_total counter\n");
        buffer.push_str(&format!("aegis_firewall_drops_total {}\n\n", self.firewall_drops_total.load(Ordering::Relaxed)));

        // 5. Active Blocked IPs Metric
        buffer.push_str("# HELP aegis_active_blocks_total Total currently blocked IPs in auto-blocker set.\n");
        buffer.push_str("# TYPE aegis_active_blocks_total gauge\n");
        buffer.push_str(&format!("aegis_active_blocks_total {}\n", self.active_blocks_total.load(Ordering::Relaxed)));

        buffer
    }
}

/// Global Collector Instance cho Observability Module
pub static GLOBAL_METRICS: MetricsCollector = MetricsCollector::new();
