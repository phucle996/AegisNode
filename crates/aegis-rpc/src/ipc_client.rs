// IPC Client — gRPC client qua Unix Domain Socket (CLI → Agent daemon)
// Dùng hyper-util TokioIo wrapper để bọc UnixStream thành hyper-compatible IO

use std::path::Path;

use aegis_core::{AegisError, Result};
use hyper_util::rt::TokioIo;
use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

use crate::agent_grpc::agent::{
    agent_service_client::AgentServiceClient, ApplyFirewallPolicyRequest, ServiceOpRequest,
};

/// Khởi tạo gRPC channel qua Unix Domain Socket tới Agent daemon
pub async fn connect_uds(socket_path: &Path) -> Result<Channel> {
    let socket_path = socket_path.to_path_buf();

    // URI placeholder — tonic cần URI hợp lệ nhưng kết nối thực tế qua UDS
    let channel = Endpoint::try_from("http://localhost")
        .map_err(|e| AegisError::Internal(format!("Invalid UDS endpoint config: {e}")))?
        .connect_with_connector(service_fn(move |_: Uri| {
            let path = socket_path.clone();
            async move {
                // Mở Unix Domain Socket connection
                let stream = UnixStream::connect(&path).await.map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::ConnectionRefused,
                        format!("Cannot connect to UDS at {path:?}: {e}"),
                    )
                })?;
                // Wrap UnixStream bằng TokioIo để satisfy hyper's rt::Read + rt::Write
                Ok::<_, std::io::Error>(TokioIo::new(stream))
            }
        }))
        .await
        .map_err(|e| AegisError::Internal(format!("Failed to establish UDS gRPC channel: {e}")))?;

    Ok(channel)
}

/// Wrapper client cung cấp typed methods cho CLI → Agent IPC calls
pub struct IpcAgentClient {
    inner: AgentServiceClient<Channel>,
}

impl IpcAgentClient {
    /// Kết nối tới Agent daemon qua Unix Domain Socket
    pub async fn connect(socket_path: &Path) -> Result<Self> {
        let channel = connect_uds(socket_path).await?;
        Ok(Self {
            inner: AgentServiceClient::new(channel),
        })
    }

    /// Gọi GetStatus RPC — lấy trạng thái tổng quan Agent
    pub async fn get_status(
        &mut self,
    ) -> Result<crate::agent_grpc::agent::AgentStatusResponse> {
        let req = tonic::Request::new(crate::agent_grpc::agent::Empty {});
        let resp = self
            .inner
            .get_status(req)
            .await
            .map_err(|e| AegisError::Internal(format!("GetStatus gRPC call failed: {e}")))?;
        Ok(resp.into_inner())
    }

    /// Gọi ApplyFirewallPolicy RPC — apply policy qua IPC tới daemon
    pub async fn apply_firewall_policy(
        &mut self,
        policy_json: String,
        rollback_timeout_secs: u32,
    ) -> Result<crate::agent_grpc::agent::ApplyFirewallPolicyResponse> {
        let req = tonic::Request::new(ApplyFirewallPolicyRequest {
            policy_json,
            rollback_timeout_secs,
        });
        let resp = self
            .inner
            .apply_firewall_policy(req)
            .await
            .map_err(|e| {
                AegisError::Internal(format!("ApplyFirewallPolicy gRPC call failed: {e}"))
            })?;
        Ok(resp.into_inner())
    }

    /// Gọi ExecuteServiceOp RPC — thực thi thao tác Systemd qua IPC
    pub async fn execute_service_op(
        &mut self,
        unit_name: String,
        operation: String,
        reason: String,
    ) -> Result<crate::agent_grpc::agent::ServiceOpResponse> {
        let req = tonic::Request::new(ServiceOpRequest {
            unit_name,
            operation,
            reason,
        });
        let resp = self
            .inner
            .execute_service_op(req)
            .await
            .map_err(|e| {
                AegisError::Internal(format!("ExecuteServiceOp gRPC call failed: {e}"))
            })?;
        Ok(resp.into_inner())
    }
}
