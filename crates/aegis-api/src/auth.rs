// Authentication & Authorization Middleware cho Controller REST API Server
// Kiểm tra Bearer Token trong Authorization Header hoặc Token Query Parameter

use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

/// Middleware xác thực Bearer Token
pub async fn require_auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..];
            if !token.is_empty() {
                // Token hợp lệ -> cho phép tiếp tục luồng request
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => {
            // Cho phép các endpoint công khai (như /v1/status hoặc /v1/auth/login) đi qua
            let path = request.uri().path();
            if path == "/v1/status" || path == "/v1/auth/login" || path == "/v1/nodes/enroll" {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
    }
}
