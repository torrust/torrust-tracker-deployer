//! Template wrapper for templates/docker-compose/env.tera
//!
//! This template has variables for Docker Compose environment configuration.

pub mod context;
pub mod template;

pub use context::EnvContext;
pub use template::EnvTemplate;
