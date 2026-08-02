//! Bearer JWT Authentication & RBAC Permission Guards (Phase 21 Authorization Guard)
//! Xác thực Bearer JWT token từ Authorization Header, trích xuất Claims payload và thực thi so khớp RBAC `object:behavior`.

use aegis_models::security::rbac::Claims;
use axum::extract::Request;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::Response;

use crate::middleware::jwt_provider::JwtProvider;

/// Marker struct đại diện cho AuthMiddleware
pub struct AuthMiddleware;

/// Secret key dùng cho JWT signature (mặc định lấy từ ENV hoặc secret tĩnh)
pub const DEFAULT_JWT_SECRET: &str = "aegisnode_jwt_secret_key_production_default";

/// Axum Middleware xác thực JWT Bearer Token
/// Giải mã JWT Token, kiểm tra signature & expiry, sau đó nạp `Claims` vào Request Extensions
pub async fn parse_bearer_token_middleware(
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    // Danh sách endpoint public không yêu cầu xác thực JWT
    let is_public = matches!(
        path,
        "/health" | "/readiness" | "/v1/auth/login" | "/v1/enrollment/sign"
    );

    if is_public {
        return Ok(next.run(request).await);
    }

    // Trích xuất Bearer token từ Header
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(str::trim);

    match auth_header {
        Some(token) if !token.is_empty() => {
            // Xác thực chữ ký và thời hạn của JWT Token
            match JwtProvider::verify_token(token, DEFAULT_JWT_SECRET) {
                Ok(claims) => {
                    // Đưa claims vào Request Extensions để handlers/guards truy cập
                    request.extensions_mut().insert(claims);
                    Ok(next.run(request).await)
                }
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

/// Kiểm tra xem Claims trong Request Extensions có đủ quyền `object:behavior` hay không
pub fn check_request_permission(request: &Request, resource: &str, action: &str) -> bool {
    if let Some(claims) = request.extensions().get::<Claims>() {
        claims.has_permission(resource, action)
    } else {
        false
    }
}
