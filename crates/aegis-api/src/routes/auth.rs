//! Controller Authentication & Login REST API Handlers
//! Xử lý đăng nhập, xác thực tài khoản Linux OS qua PAM và cấp phát JWT Token.

use std::sync::Arc;

use axum::extract::{Json, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::controller_router::ControllerState;
use crate::middleware::auth::DEFAULT_JWT_SECRET;
use crate::middleware::jwt_provider::JwtProvider;
use crate::middleware::pam_auth::PamAuthenticator;

/// Request Payload cho Login API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    // Tên tài khoản Linux OS
    pub username: String,
    // Mật khẩu truy cập tài khoản Linux OS (Plaintext string được gửi an toàn qua TLS/HTTPS)
    pub password: String,
}

/// Response Payload cho Login API
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub token: String,
    pub expires_in_seconds: u64,
}

/// Handler `POST /v1/auth/login`: Xác thực tài khoản Linux OS (PAM/Cockpit style) và cấp JWT Token
/// JWT Payload được inject danh sách Roles & Permissions (`object:behavior`) dựa trên Linux Groups
pub async fn login_handler(
    State(state): State<Arc<ControllerState>>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, StatusCode> {
    // Kiểm tra tên đăng nhập và mật khẩu không được phép trống
    if req.username.is_empty() || req.password.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // 1. Xác thực tài khoản Linux OS qua PAM / Shadow password hash verification thực sự
    let groups = PamAuthenticator::authenticate(&req.username, &req.password)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 2. Ánh xạ từ Linux Groups sang Roles & Permissions list
    let (roles, permissions) = PamAuthenticator::map_groups_to_permissions(&groups);

    // 3. Đóng gói Claims và Ký số JWT Token
    let secret = &state.config.server.auth_secret;
    let effective_secret = if secret.is_empty() {
        DEFAULT_JWT_SECRET
    } else {
        secret
    };

    let ttl = state.config.server.session_ttl_seconds;
    let claims = JwtProvider::issue_token(&req.username, roles, permissions, effective_secret, ttl)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let jwt_token = JwtProvider::encode_claims(&claims, effective_secret)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(LoginResponse {
        token: jwt_token,
        expires_in_seconds: ttl,
    }))
}
