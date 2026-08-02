//! JWT Provider & Verifier Module (Phase 21 Stateless Authentication)
//! Ký số JWT mang theo claims (sub, roles, perms) và xác minh chữ ký HMAC-SHA256.

use aegis_core::{AegisError, Result};
use aegis_models::security::rbac::{Claims, Role};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

/// Provider hỗ trợ tạo và xác thực JWT token
pub struct JwtProvider;

impl JwtProvider {
    /// Sinh JWT Token chứa danh sách Roles & Permissions
    pub fn issue_token(
        username: &str,
        roles: Vec<Role>,
        permissions: Vec<String>,
        _secret: &str,
        ttl_seconds: u64,
    ) -> Result<Claims> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let claims = Claims {
            sub: username.to_string(),
            roles,
            permissions,
            exp: now + ttl_seconds,
            iat: now,
        };

        Ok(claims)
    }

    /// Ký claims thành chuỗi JWT Token dạng Base64 được bảo mật bằng HMAC-SHA256
    pub fn encode_claims(claims: &Claims, secret: &str) -> Result<String> {
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        encode(&Header::default(), claims, &encoding_key)
            .map_err(|e| AegisError::Permission(format!("Không thể ký số JWT token: {e}")))
    }

    /// Giải mã và xác thực chữ ký JWT Token (Cấu hình dung sai 60 giây chống từ chối nhầm do lệch clock skew)
    pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let mut validation = Validation::default();
        // Bật kiểm tra thời hạn hết hạn token
        validation.validate_exp = true;
        // Thiết lập dung sai lệch thời gian hệ thống NTP tối đa 60 giây
        validation.leeway = 60;

        let token_data = decode::<Claims>(token, &decoding_key, &validation)
            .map_err(|e| AegisError::Permission(format!("Xác thực JWT token thất bại: {e}")))?;

        Ok(token_data.claims)
    }
}
