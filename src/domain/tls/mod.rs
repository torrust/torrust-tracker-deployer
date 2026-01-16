//! TLS domain types
//!
//! This module contains domain types for TLS configuration on services.
//!
//! ## Purpose
//!
//! The `TlsConfig` type represents validated TLS settings that are stored
//! in service configurations and used for Caddy reverse proxy setup.
//!
//! ## See Also
//!
//! - Application layer DTOs: `src/application/command_handlers/create/config/https.rs`
//! - Caddy template context: `src/infrastructure/templating/caddy/`
//! - HTTPS domain config: `src/domain/https/`

pub mod config;

pub use config::TlsConfig;
