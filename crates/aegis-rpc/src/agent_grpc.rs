// Agent gRPC module — include! generated code từ agent.proto
// Wrapper cho AgentServiceServer để triển khai trong aegisnode binary

/// Module chứa generated protobuf types từ agent.proto
pub mod agent {
    // tonic-build sẽ gen file này tại OUT_DIR trong build time
    tonic::include_proto!("aegis.agent.v1");
}

use aegis_core::{AegisError, Result};

/// Adapter convert ApplyFirewallPolicyRequest gRPC message → domain action
pub fn parse_apply_request(req: &agent::ApplyFirewallPolicyRequest) -> Result<serde_json::Value> {
    serde_json::from_str(&req.policy_json)
        .map_err(|e| AegisError::Validation(format!("Invalid policy JSON in gRPC request: {e}")))
}

/// Tạo RolloutCommand message để Controller stream xuống Agent
pub fn build_rollout_command(
    rollout_id: &str,
    command_type: &str,
    payload: serde_json::Value,
) -> agent::RolloutCommand {
    agent::RolloutCommand {
        rollout_id: rollout_id.to_string(),
        command_type: command_type.to_string(),
        payload_json: payload.to_string(),
        issued_at_unix: chrono::Utc::now().timestamp(),
    }
}
