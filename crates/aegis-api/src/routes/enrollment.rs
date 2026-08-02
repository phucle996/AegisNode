//! Node Enrollment & mTLS Certificate Exchange REST API Handlers (Phase 13 & 23)
//! Tạo Token đăng ký ngắn hạn, ký CSR Agent CSR và tiếp nhận Heartbeat định kỳ.

use std::result::Result as StdResult;
use std::sync::Arc;

use aegis_core::pki::EnrollmentToken;
use axum::extract::{Json, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::controller_router::ControllerState;

/// Request Payload tạo Enrollment Token mới
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEnrollmentTokenRequest {
    pub ttl_minutes: Option<i64>,
    pub max_usages: Option<u32>,
}

/// Response Payload trả về Enrollment Token
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEnrollmentTokenResponse {
    pub token_id: Uuid,
    pub token: String,
    pub expires_at: String,
    pub max_usages: u32,
}

/// Request Payload gửi CSR từ Linux Agent Node
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEnrollCsrRequest {
    pub enrollment_token: String,
    pub hostname: String,
    pub machine_id: String,
    pub ip_address: String,
    pub csr_pem: String,
}

/// Response Payload trả về Client Certificate cho Agent
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEnrollCsrResponse {
    pub node_id: Uuid,
    pub certificate_pem: String,
    pub ca_certificate_pem: String,
    pub expires_at: String,
}

/// Request Payload gửi Heartbeat định kỳ qua đường truyền mTLS
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeHeartbeatRequest {
    pub node_id: Uuid,
    pub status: String,
}

/// Handler `POST /v1/enrollment/token/create`: Sinh Token gia nhập mới cho Admin
pub async fn create_enrollment_token_handler(
    State(state): State<Arc<ControllerState>>,
    Json(req): Json<CreateEnrollmentTokenRequest>,
) -> StdResult<Json<CreateEnrollmentTokenResponse>, StatusCode> {
    let ttl = req.ttl_minutes.unwrap_or(60);
    let usages = req.max_usages.unwrap_or(1);
    // Tạo enrollment token với TTL và giới hạn số lần sử dụng
    let token = EnrollmentToken::new(ttl, usages);

    // Lưu token hash vào DB nếu có kết nối
    if let Some(repo) = &state.repository {
        repo.insert_enrollment_token(&token.token_hash, usages as i32, ttl)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(CreateEnrollmentTokenResponse {
        token_id: token.id,
        token: token.token_string,
        expires_at: token.expires_at.to_rfc3339(),
        max_usages: usages,
    }))
}

/// Handler `POST /v1/enrollment/sign`: Xác thực Token, ký CSR và cấp Client Certificate cho Agent Node
pub async fn sign_agent_csr_handler(
    State(state): State<Arc<ControllerState>>,
    Json(req): Json<NodeEnrollCsrRequest>,
) -> StdResult<Json<NodeEnrollCsrResponse>, StatusCode> {
    // 1. Kiểm tra tham số request thô không được rỗng
    if req.enrollment_token.trim().is_empty() || req.machine_id.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 2. Nếu có kết nối PostgreSQL Repository, thực hiện tiêu thụ Enrollment Token một cách nguyên tử (Atomic)
    if let Some(repo) = &state.repository {
        // Băm chuỗi Token gửi lên bằng SHA-256 để so sánh bản băm trong DB
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(req.enrollment_token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        // Thử tiêu thụ token trong PostgreSQL (chỉ chấp nhận nếu unexpired, unrevoked, usages < max)
        let consumed = repo
            .consume_enrollment_token(&token_hash)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        // Nếu token không hợp lệ hoặc đã hết lượt -> Từ chối cấp chứng chỉ (401 Unauthorized)
        if !consumed {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // 3. Sinh UUIDv4 duy nhất làm Node ID cho Agent
    let node_id = Uuid::new_v4();

    // 4. Ký X.509 Client Certificate cấp cho Agent bằng PkiManager của ControllerState
    let cert_record = state
        .pki_manager
        .sign_agent_csr(node_id, &req.machine_id, &req.hostname, &req.csr_pem, 365)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 5. Lưu thông tin Node & Certificate bản ghi vào PostgreSQL Database
    if let Some(repo) = &state.repository {
        let labels = serde_json::json!({ "machineId": req.machine_id });
        repo.upsert_node(&req.hostname, &req.ip_address, &labels, "0.1.0")
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        repo.save_agent_certificate(&cert_record)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    // 6. Trả về Response chứa Client Certificate PEM và Root CA Certificate PEM
    Ok(Json(NodeEnrollCsrResponse {
        node_id,
        certificate_pem: cert_record.cert_pem,
        ca_certificate_pem: state.pki_manager.ca_cert_pem.clone(),
        expires_at: cert_record.expires_at.to_rfc3339(),
    }))
}

/// Handler `POST /v1/nodes/heartbeat`: Nhận bản tin Heartbeat định kỳ từ Agent qua mTLS
pub async fn node_heartbeat_handler(
    State(state): State<Arc<ControllerState>>,
    Json(req): Json<NodeHeartbeatRequest>,
) -> StdResult<Json<serde_json::Value>, StatusCode> {
    if let Some(repo) = &state.repository {
        repo.update_node_heartbeat(req.node_id, &req.status)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(Json(serde_json::json!({
        "status": "ACK",
        "nodeId": req.node_id,
        "receivedAt": chrono::Utc::now().to_rfc3339()
    })))
}
