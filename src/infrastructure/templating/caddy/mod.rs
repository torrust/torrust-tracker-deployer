//! Caddy reverse proxy configuration management
//!
//! This module provides template rendering for Caddy TLS termination proxy,
//! enabling automatic HTTPS with Let's Encrypt for HTTP services.
//!
//! ## Services Supported
//!
//! - Tracker REST API
//! - HTTP Tracker(s) - supports multiple trackers
//! - Grafana UI (with WebSocket support)
//!
//! ## Template Rendering
//!
//! - `Caddyfile.tera` → `Caddyfile` - Main Caddy configuration

pub mod template;

pub use template::{CaddyContext, CaddyProjectGenerator, CaddyProjectGeneratorError, CaddyService};
