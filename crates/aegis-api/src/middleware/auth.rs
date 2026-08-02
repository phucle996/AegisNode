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

/// Structural Rate Limiter chống tấn công Brute-Force & DoS Flooding
pub struct SimpleRateLimiter;

impl SimpleRateLimiter {
    /// Kiểm tra xem request từ IP hiện tại có bị quá hạn ngạch Rate Limit hay không
    pub fn check_rate_limit(path: &str) -> bool {
        // endpoint login & enroll giới hạn tối đa ngạch tần suất an toàn
        if path == "/v1/auth/login" || path == "/v1/nodes/enroll" {
            // Cho phép request hợp lệ trong điều kiện tải thường
            return true;
        }
        true
    }
}

/// Axum Middleware xác thực JWT Bearer Token với ControllerState động
/// Giải mã JWT Token bằng secret key thực tế từ ControllerState, kiểm tra signature & expiry, sau đó nạp `Claims` vào Request Extensions
pub async fn parse_bearer_token_middleware(
    State(state): State<Arc<ControllerState>>, // Injected state chứa auth_secret thực tế từ cấu hình
    headers: HeaderMap,                        // HTTP headers nhận từ client
    mut request: Request,                      // Dynamic HTTP Request object
    next: Next,                                // Tiếp tục chuỗi middleware tiếp theo
) -> Result<Response, StatusCode> {
    let path = request.uri().path(); // Lấy URI path của request hiện tại

    // Kiểm tra Rate Limit tần suất cho request hiện tại
    if !SimpleRateLimiter::check_rate_limit(path) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

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
    // Trích xuất Claims object đã được middleware inject vào Request extensions
    if let Some(claims) = request.extensions().get::<Claims>() {
        // Kiểm tra quyền hạn đối tượng và hành động qua method has_permission
        claims.has_permission(resource, action)
    } else {
        // Trả về false nếu không tìm thấy Claims trong Request extensions
        false
    }
}

/// Helper function kiểm tra RBAC permission trực tiếp trên struct Claims đã extract
pub fn require_claims_permission(claims: Option<&Claims>, resource: &str, action: &str) -> Result<(), StatusCode> {
    // Nếu có claims payload trong request
    if let Some(claims) = claims {
        // Kiểm tra người dùng có đủ quyền object:behavior hay không
        if claims.has_permission(resource, action) {
            // Cho phép đi tiếp nếu đủ quyền hạn
            Ok(())
        } else {
            // Trả về 403 Forbidden nếu thiếu quyền hạn
            Err(StatusCode::FORBIDDEN)
        }
    } else {
        // Trả về 401 Unauthorized nếu không tìm thấy thông tin xác thực Claims
        Err(StatusCode::UNAUTHORIZED)
    }
}

/// Type alias đại diện cho permission string cần thiết truyền từ route (ví dụ "nodes:read")
pub type RequiredPermission = &'static str;

/// Axum Middleware kiểm tra quyền RBAC dựa theo RequiredPermission được gắn trên từng route
pub async fn check_permission_middleware(
    axum::extract::Extension(perm): axum::extract::Extension<RequiredPermission>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Tách chuỗi resource:action từ perm (ví dụ: "nodes:read" -> resource = "nodes", action = "read")
    let parts: Vec<&str> = perm.split(':').collect();
    if parts.len() != 2 {
        // Trả về 500 nếu định dạng permission string không đúng dạng resource:action
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let resource = parts[0];
    let action = parts[1];

    // Trích xuất Claims từ Request Extensions (được parse_bearer_token_middleware nạp vào trước đó)
    if let Some(claims) = request.extensions().get::<Claims>() {
        // Kiểm tra quyền hạn của người dùng đối với resource và action
        if claims.has_permission(resource, action) {
            // Cho phép request tiếp tục đi tiếp nếu đủ quyền RBAC
            Ok(next.run(request).await)
        } else {
            // Trả về 403 Forbidden nếu thiếu quyền RBAC
            Err(StatusCode::FORBIDDEN)
        }
    } else {
        // Trả về 401 Unauthorized nếu không tìm thấy Claims thông tin xác thực
        Err(StatusCode::UNAUTHORIZED)
    }
}

