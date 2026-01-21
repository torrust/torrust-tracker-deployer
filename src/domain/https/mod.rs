//! HTTPS domain types
//!
//! This module contains domain types for HTTPS/TLS configuration.
//!
//! ## Purpose
//!
//! The `HttpsConfig` type represents validated HTTPS settings that are stored
//! in the environment and used for Caddy TLS termination configuration.
//!
//! ## See Also
//!
//! - Application layer DTOs: `src/application/command_handlers/create/config/https.rs`
//! - Caddy template context: `src/infrastructure/templating/caddy/`

pub mod config;

pub use config::{HttpsConfig, HttpsConfigError};
