//! AegisNode API Crate
//! Xây dựng HTTP & Unix Socket RESTful API cho Agent Daemon và Controller Server bằng Axum Web Framework.

pub mod controller_router;
pub mod middleware;
pub mod router;
pub mod routes;
pub mod state;

pub use controller_router::{ControllerState, create_controller_router};
pub use router::create_router;
pub use routes::*;
pub use state::AppState;
