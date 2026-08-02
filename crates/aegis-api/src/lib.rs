//! AegisNode API Crate
//! Xây dựng HTTP & Unix Socket RESTful API cho Agent Daemon bằng Axum Web Framework.

pub mod handlers;
pub mod router;
pub mod state;

pub use router::create_router;
pub use state::AppState;
