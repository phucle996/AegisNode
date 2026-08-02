// Integration Test cho Prometheus Alerting Rules & Grafana Provisioning (Phase 26 Production Observability)

use aegis_observability::{
    generate_firewall_activity_dashboard_json, generate_fleet_health_dashboard_json,
    generate_prometheus_rules_yaml,
};

#[test]
fn test_prometheus_alert_rules_generation() {
    let yaml = generate_prometheus_rules_yaml();

    assert!(yaml.contains("AegisAgentOffline"));
    assert!(yaml.contains("AegisRolloutHighFailureRate"));
    assert!(yaml.contains("AegisCertExpiringSoon"));
    assert!(yaml.contains("AegisFirewallDropAnomaly"));
}

#[test]
fn test_grafana_dashboards_generation() {
    let fleet_json = generate_fleet_health_dashboard_json();
    assert!(fleet_json.contains("Fleet Health & Agent Connectivity"));

    let firewall_json = generate_firewall_activity_dashboard_json();
    assert!(firewall_json.contains("Firewall Activity & Dropped Packets"));
}
