// Bearer JWT Authentication & RBAC Permission Guards (Phase 21 Authorization Guard)
// Xác thực Bearer JWT token từ Authorization Header, trích xuất Claims payload và thực thi so khớp RBAC `object:behavior`.

use std::sync::Arc; // Con trỏ tham chiếu luồng an toàn Arc

use aegis_models::security::rbac::Claims; // Import Claims định nghĩa RBAC payload
use axum::extract::{Request, State}; // Import Extractors từ Axum
use axum::http::{HeaderMap, StatusCode}; // Import HTTP headers và status code
use axum::middleware::Next; // Import Next middleware chain
use axum::response::Response; // Import HTTP Response

use crate::controller_router::ControllerState; // Import ControllerState chứa cấu hình runtime
use crate::middleware::jwt_provider::JwtProvider; // Import JwtProvider hỗ trợ giải mã token

/// Marker struct đại diện cho AuthMiddleware
pub struct AuthMiddleware;

/// Secret key dùng mặc định làm fallback an toàn (dùng khi không truyền config)
pub const DEFAULT_JWT_SECRET: &str = "aegisnode_jwt_secret_key_production_default";

/// Axum Middleware xác thực JWT Bearer Token với ControllerState động
/// Giải mã JWT Token bằng secret key thực tế từ ControllerState, kiểm tra signature & expiry, sau đó nạp `Claims` vào Request Extensions
pub async fn parse_bearer_token_middleware(
    State(state): State<Arc<ControllerState>>, // Injected state chứa auth_secret thực tế từ cấu hình
    headers: HeaderMap,                        // HTTP headers nhận từ client
    mut request: Request,                      // Dynamic HTTP Request object
    next: Next,                                // Tiếp tục chuỗi middleware tiếp theo
) -> Result<Response, StatusCode> {
    let path = request.uri().path(); // Lấy URI path của request hiện tại

    // Danh sách endpoint public không yêu cầu xác thực JWT (như health check probe và login)
    let is_public = matches!(
        path,
        "/health" | "/readiness" | "/v1/auth/login" | "/v1/enrollment/sign"
    );

    // Nếu là endpoint công khai thì cho phép đi tiếp không kiểm tra token
    if is_public {
        return Ok(next.run(request).await);
    }

    // Trích xuất Bearer token từ Authorization Header
    let auth_header = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(str::trim);

    // Lấy JWT secret key thực tế từ cấu hình server (fallback về DEFAULT_JWT_SECRET nếu rỗng)
    let secret = &state.config.server.auth_secret;
    let effective_secret = if secret.is_empty() {
        DEFAULT_JWT_SECRET
    } else {
        secret
    };

    match auth_header {
        Some(token) if !token.is_empty() => {
            // Xác thực chữ ký và thời hạn của JWT Token bằng effective_secret
            match JwtProvider::verify_token(token, effective_secret) {
                Ok(claims) => {
                    // Đưa claims đã giải mã thành công vào Request Extensions để handlers/guards truy cập
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
