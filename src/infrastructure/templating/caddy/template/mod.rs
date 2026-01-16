//! Caddy template functionality
//!
//! This module provides template-related functionality for Caddy configuration,
//! including context for dynamic templates.

pub mod renderer;
pub mod wrapper;

pub use renderer::{CaddyProjectGenerator, CaddyProjectGeneratorError};
pub use wrapper::{CaddyContext, CaddyService};
