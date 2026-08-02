//! Grafana Dashboards JSON Spec Provisioning (Phase 26 Production Observability)
//! Cung cấp cấu hình Grafana Dashboards JSON cho Fleet Health, Firewall Activity và Rollout Operations.

/// Sinh cấu hình JSON Grafana Dashboard giám sát Sức khỏe Fleet (Fleet Health Dashboard)
pub fn generate_fleet_health_dashboard_json() -> String {
    serde_json::json!({
        "dashboard": {
            "id": null,
            "title": "AegisNode - Fleet Health & Agent Connectivity",
            "tags": ["aegisnode", "fleet", "connectivity"],
            "timezone": "browser",
            "schemaVersion": 16,
            "version": 1,
            "panels": [
                {
                    "title": "Total Connected Agents",
                    "type": "stat",
                    "targets": [
                        { "expr": "sum(aegis_agent_connected)" }
                    ]
                },
                {
                    "title": "Agent Connectivity Status",
                    "type": "stat",
                    "targets": [
                        { "expr": "aegis_agent_connected" }
                    ]
                }
            ]
        }
    })
    .to_string()
}

/// Sinh cấu hình JSON Grafana Dashboard giám sát Hoạt động Firewall (Firewall Activity Dashboard)
pub fn generate_firewall_activity_dashboard_json() -> String {
    serde_json::json!({
        "dashboard": {
            "id": null,
            "title": "AegisNode - Firewall Activity & Dropped Packets",
            "tags": ["aegisnode", "firewall", "security"],
            "timezone": "browser",
            "schemaVersion": 16,
            "version": 1,
            "panels": [
                {
                    "title": "Firewall Drops Rate (packets/sec)",
                    "type": "graph",
                    "targets": [
                        { "expr": "rate(aegis_firewall_drops_total[1m])" }
                    ]
                },
                {
                    "title": "Active Blocklist IPs",
                    "type": "stat",
                    "targets": [
                        { "expr": "aegis_blocklist_size" }
                    ]
                }
            ]
        }
    })
    .to_string()
}
