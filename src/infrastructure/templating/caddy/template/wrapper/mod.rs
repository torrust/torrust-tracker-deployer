//! Template wrappers for Caddyfile.tera
//!
//! This module provides context and template wrappers for Caddy configuration.

pub mod caddyfile;

pub use caddyfile::{CaddyContext, CaddyService};
