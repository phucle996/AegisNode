//! AegisNode API Crate
//! Xây dựng HTTP & Unix Socket RESTful API cho Agent Daemon và Controller Server bằng Axum Web Framework.

pub mod auth;
pub mod controller_router;
pub mod enrollment;
pub mod ha_status;
pub mod handlers;
pub mod inventory_router;
pub mod network_router;
pub mod rollout_router;
pub mod router;
pub mod state;
pub mod systemd_router;

pub use controller_router::{ControllerState, create_controller_router};
pub use ha_status::{ha_status_handler, health_check_handler, readiness_check_handler};
pub use router::create_router;
pub use state::AppState;
