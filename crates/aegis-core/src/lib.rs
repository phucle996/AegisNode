//! AegisNode Core Library
//! Định nghĩa các Identifier, Error types, PKI helpers, Peer Credentials validator và Security Hardening guards.

pub mod error;
pub mod hardening;
pub mod identifiers;
pub mod peer_cred;
pub mod pki;

pub use error::{AegisError, ErrorResponse, Result};
pub use hardening::{MAX_API_PAYLOAD_SIZE_BYTES, SecurityHardening};
pub use identifiers::*;
pub use peer_cred::*;
pub use pki::*;
