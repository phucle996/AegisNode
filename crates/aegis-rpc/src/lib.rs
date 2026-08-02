//! AegisNode RPC & Transport Crate
//! Chứa gRPC Server/Client qua UDS & mTLS, IPC client, Executor Protocol và Multi-Endpoint Failover Client (Phase 23).

pub mod agent_grpc;
pub mod controller_grpc;
pub mod executor_proto;
pub mod failover_client;
pub mod ipc_client;
pub mod rpc_client;

pub use agent_grpc::*;
pub use controller_grpc::*;
pub use executor_proto::*;
pub use failover_client::*;
pub use ipc_client::*;
pub use rpc_client::*;
