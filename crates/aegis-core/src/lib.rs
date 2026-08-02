//! AegisNode Core Library
//! Định nghĩa các Identifier, Error types, PKI helpers và Peer Credentials validator.

pub mod error;
pub mod identifiers;
pub mod peer_cred;
pub mod pki;

pub use error::{AegisError, ErrorResponse, Result};
pub use identifiers::*;
pub use peer_cred::*;
pub use pki::*;
