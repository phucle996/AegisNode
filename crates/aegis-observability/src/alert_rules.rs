//! Prometheus Alerting Rules Generator (Phase 26 Production Observability)
//! Cung cấp các định nghĩa Quy tắc Cảnh báo Prometheus tiêu chuẩn cho AegisNode.

use serde::{Deserialize, Serialize};

/// Cấu trúc quy tắc cảnh báo Prometheus (PrometheusAlertRule)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrometheusAlertRule {
    /// Tên của cảnh báo (Alert Name)
    pub alert: String,
    /// Biểu thức Prometheus Query (PromQL)
    pub expr: String,
    /// Thời gian chờ kích hoạt (Duration)
    pub for_duration: String,
    /// Mức độ nghiêm trọng (critical / warning)
    pub severity: String,
    /// Tóm tắt nội dung cảnh báo
    pub summary: String,
    /// Mô tả chi tiết sự cố
    pub description: String,
}

/// Sinh nội dung YAML cấu hình Prometheus Alerting Rules tiêu chuẩn
pub fn generate_prometheus_rules_yaml() -> String {
    let rules = vec![
        PrometheusAlertRule {
            alert: "AegisAgentOffline".to_string(),
            expr: "aegis_agent_connected == 0".to_string(),
            for_duration: "3m".to_string(),
            severity: "critical".to_string(),
            summary: "AegisNode Agent has gone offline".to_string(),
            description: "Agent on node {{ $labels.node_id }} has lost connection to Controller for > 3 minutes.".to_string(),
        },
        PrometheusAlertRule {
            alert: "AegisRolloutHighFailureRate".to_string(),
            expr: "rate(aegis_rollout_failure_total[5m]) > 0.1".to_string(),
            for_duration: "2m".to_string(),
            severity: "critical".to_string(),
            summary: "High Rollout Failure Rate Detected".to_string(),
            description: "Rollout failure rate exceeds 10% over 5 minutes.".to_string(),
        },
        PrometheusAlertRule {
            alert: "AegisCertExpiringSoon".to_string(),
            expr: "aegis_certificate_expiry_days < 7".to_string(),
            for_duration: "1h".to_string(),
            severity: "warning".to_string(),
            summary: "mTLS Certificate Expiring Soon".to_string(),
            description: "Node {{ $labels.node_id }} mTLS certificate will expire in less than 7 days.".to_string(),
        },
        PrometheusAlertRule {
            alert: "AegisFirewallDropAnomaly".to_string(),
            expr: "rate(aegis_firewall_drops_total[1m]) > 1000".to_string(),
            for_duration: "1m".to_string(),
            severity: "warning".to_string(),
            summary: "Spike in Firewall Packet Drops".to_string(),
            description: "Sudden anomaly spike in packet drops (> 1000 drops/sec) detected on interface {{ $labels.device }}.".to_string(),
        },
    ];

    let mut yaml_output = String::from("groups:\n  - name: aegisnode_alerts\n    rules:\n");

    for r in rules {
        yaml_output.push_str(&format!("      - alert: {}\n", r.alert));
        yaml_output.push_str(&format!("        expr: {}\n", r.expr));
        yaml_output.push_str(&format!("        for: {}\n", r.for_duration));
        yaml_output.push_str("        labels:\n");
        yaml_output.push_str(&format!("          severity: {}\n", r.severity));
        yaml_output.push_str("        annotations:\n");
        yaml_output.push_str(&format!("          summary: \"{}\"\n", r.summary));
        yaml_output.push_str(&format!("          description: \"{}\"\n", r.description));
    }

    yaml_output
}
