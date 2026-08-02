//! aegis-rpc — AegisNode gRPC Transport Layer
//! Cung cấp generated gRPC stubs, transport adapters và IPC client/server wrappers.
//!
//! # Cấu trúc:
//! - `agent_grpc`: Generated code từ agent.proto + AgentService impl helpers
//! - `controller_grpc`: Generated code từ controller.proto + ControllerService impl helpers
//! - `ipc_client`: gRPC client qua Unix Domain Socket (CLI → Agent)
//! - `rpc_client`: gRPC client qua TCP mTLS (Agent → Controller)

pub mod agent_grpc;
pub mod controller_grpc;
pub mod ipc_client;
pub mod rpc_client;

// Re-export generated proto types cho tiện sử dụng
pub use agent_grpc::agent::{
    AgentStatusResponse, ApplyFirewallPolicyRequest, ApplyFirewallPolicyResponse,
    InventoryResponse, RolloutCommand, ServiceOpRequest, ServiceOpResponse,
};
pub use controller_grpc::controller::{
    HeartbeatAck, HeartbeatRequest, InventoryReport, RolloutStatusUpdate,
};
