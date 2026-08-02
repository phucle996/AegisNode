// RPC Client — gRPC client qua TCP mTLS (Agent → Controller network)
// Agent dùng client này để gửi Heartbeat, Inventory và Rollout Status lên Controller

use aegis_core::{AegisError, Result};
use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};

use crate::controller_grpc::controller::{
    HeartbeatRequest, InventoryReport, RolloutStatusUpdate,
    controller_service_client::ControllerServiceClient,
};

/// Config cho mTLS connection từ Agent → Controller
pub struct MtlsConfig {
    pub ca_cert_pem: String,
    pub client_cert_pem: String,
    pub client_key_pem: String,
    pub server_domain: String,
}

/// Khởi tạo mTLS gRPC Channel từ Agent → Controller
pub async fn connect_mtls(controller_url: &str, tls: &MtlsConfig) -> Result<Channel> {
    // Load CA cert để verify Controller identity
    let ca = Certificate::from_pem(&tls.ca_cert_pem);
    // Load Agent client cert + key để authenticate
    let identity = Identity::from_pem(&tls.client_cert_pem, &tls.client_key_pem);

    let tls_config = ClientTlsConfig::new()
        .ca_certificate(ca)
        .identity(identity)
        .domain_name(&tls.server_domain);

    let channel = Endpoint::try_from(controller_url.to_string())
        .map_err(|e| {
            AegisError::Internal(format!("Invalid Controller URL '{controller_url}': {e}"))
        })?
        .tls_config(tls_config)
        .map_err(|e| AegisError::Internal(format!("TLS config error: {e}")))?
        .connect()
        .await
        .map_err(|e| {
            AegisError::Internal(format!(
                "Failed to connect to Controller at '{controller_url}' via mTLS: {e}"
            ))
        })?;

    Ok(channel)
}

/// Wrapper RPC client cho Agent → Controller reporting
pub struct RpcControllerClient {
    inner: ControllerServiceClient<Channel>,
}

impl RpcControllerClient {
    /// Kết nối tới Controller qua TCP mTLS
    pub async fn connect(controller_url: &str, tls: &MtlsConfig) -> Result<Self> {
        let channel = connect_mtls(controller_url, tls).await?;
        Ok(Self {
            inner: ControllerServiceClient::new(channel),
        })
    }

    /// Gửi Heartbeat định kỳ lên Controller
    pub async fn report_heartbeat(
        &mut self,
        heartbeat: HeartbeatRequest,
    ) -> Result<crate::controller_grpc::controller::HeartbeatAck> {
        let req = tonic::Request::new(heartbeat);
        let resp =
            self.inner.report_heartbeat(req).await.map_err(|e| {
                AegisError::Internal(format!("ReportHeartbeat gRPC call failed: {e}"))
            })?;
        Ok(resp.into_inner())
    }

    /// Đẩy System Inventory lên Controller
    pub async fn report_inventory(&mut self, report: InventoryReport) -> Result<()> {
        let req = tonic::Request::new(report);
        self.inner
            .report_inventory(req)
            .await
            .map_err(|e| AegisError::Internal(format!("ReportInventory gRPC call failed: {e}")))?;
        Ok(())
    }

    /// Báo cáo tiến độ Rollout Step lên Controller
    pub async fn report_rollout_status(&mut self, update: RolloutStatusUpdate) -> Result<()> {
        let req = tonic::Request::new(update);
        self.inner.report_rollout_status(req).await.map_err(|e| {
            AegisError::Internal(format!("ReportRolloutStatus gRPC call failed: {e}"))
        })?;
        Ok(())
    }
}
