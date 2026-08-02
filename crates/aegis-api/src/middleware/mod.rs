//! AegisNode API Middlewares Module
//! Chứa các HTTP middleware phục vụ Authentication (PAM/JWT), Authorization Guard (RBAC) và Observability.

pub mod auth;
pub mod jwt_provider;
pub mod pam_auth;

pub use auth::{AuthMiddleware, DEFAULT_JWT_SECRET, check_request_permission, parse_bearer_token_middleware};
pub use jwt_provider::JwtProvider;
pub use pam_auth::PamAuthenticator;
