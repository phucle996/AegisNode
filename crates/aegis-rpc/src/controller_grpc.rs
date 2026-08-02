// Controller gRPC module — include! generated code từ controller.proto
// Wrapper cho ControllerServiceServer để triển khai trong aegisnode server binary

/// Module chứa generated protobuf types từ controller.proto
pub mod controller {
    // tonic-build sẽ gen file này tại OUT_DIR trong build time
    tonic::include_proto!("aegis.controller.v1");
}

use chrono::Utc;

/// Tạo HeartbeatRequest với timestamp hiện tại để Agent gửi lên Controller
pub fn build_heartbeat(
    node_id: &str,
    hostname: &str,
    ip_address: &str,
    agent_version: &str,
) -> controller::HeartbeatRequest {
    controller::HeartbeatRequest {
        node_id: node_id.to_string(),
        hostname: hostname.to_string(),
        ip_address: ip_address.to_string(),
        agent_version: agent_version.to_string(),
        sent_at_unix: Utc::now().timestamp(),
    }
}

/// Tạo InventoryReport để Agent đẩy System Inventory lên Controller
pub fn build_inventory_report(
    node_id: &str,
    inventory_json: String,
) -> controller::InventoryReport {
    controller::InventoryReport {
        node_id: node_id.to_string(),
        inventory_json,
    }
}
