//! Secret wrapper types for sensitive data
//!
//! This module provides newtype wrappers for sensitive data types.
//! The wrappers leverage the `secrecy` crate's `SecretString` type internally
//! for automatic memory zeroing and debug redaction, while adding serialization
//! support needed for environment configuration storage.

mod api_token;
mod password;

// Re-export ExposeSecret from secrecy for convenience
pub use secrecy::ExposeSecret;

// Re-export types from submodules
pub use api_token::{ApiToken, PlainApiToken};
pub use password::{Password, PlainPassword};
