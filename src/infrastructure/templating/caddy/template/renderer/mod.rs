//! Caddy template renderers
//!
//! This module provides renderers for Caddy configuration templates.

mod caddyfile;
mod project_generator;

pub use caddyfile::{CaddyfileRenderer, CaddyfileRendererError};
pub use project_generator::{CaddyProjectGenerator, CaddyProjectGeneratorError};
