//! Caddy service configuration for Docker Compose
//!
//! This module defines the Caddy reverse proxy service configuration
//! for the docker-compose.yml template.
//!
//! ## Note on Context Separation
//!
//! This type (`CaddyServiceConfig`) is separate from the `CaddyContext` used
//! for rendering the Caddyfile.tera template. Each template has its own context:
//!
//! - `CaddyServiceConfig` (this module): For docker-compose.yml service definition
//! - `CaddyContext` (in caddy/template/wrapper): For Caddyfile content with domains/ports
//!
//! The docker-compose template only needs to know that Caddy is enabled
//! (for network/volume definitions), not the detailed service configurations.

use serde::Serialize;

use crate::domain::topology::Network;

/// Caddy reverse proxy service configuration for Docker Compose
///
/// Contains configuration for the Caddy service definition in docker-compose.yml.
/// This is intentionally minimal - the actual Caddy configuration (domains, ports)
/// is in the Caddyfile, rendered separately.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::context::CaddyServiceConfig;
/// use torrust_tracker_deployer_lib::domain::topology::Network;
///
/// let caddy = CaddyServiceConfig::new();
/// assert_eq!(caddy.networks, vec![Network::Proxy]);
/// ```
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CaddyServiceConfig {
    /// Networks this service connects to
    ///
    /// Caddy always connects to `proxy_network` for reverse proxying
    /// to backend services (tracker API, HTTP trackers, Grafana).
    pub networks: Vec<Network>,
}

impl CaddyServiceConfig {
    /// Creates a new `CaddyServiceConfig` with default networks
    ///
    /// Caddy connects to:
    /// - `proxy_network`: For reverse proxying to backend services
    #[must_use]
    pub fn new() -> Self {
        Self {
            networks: vec![Network::Proxy],
        }
    }
}

impl Default for CaddyServiceConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_connect_caddy_to_proxy_network() {
        let caddy = CaddyServiceConfig::new();

        assert_eq!(caddy.networks, vec![Network::Proxy]);
    }

    #[test]
    fn it_should_implement_default() {
        let caddy = CaddyServiceConfig::default();

        assert_eq!(caddy.networks, vec![Network::Proxy]);
    }

    #[test]
    fn it_should_serialize_network_to_name_string() {
        let caddy = CaddyServiceConfig::new();

        let json = serde_json::to_value(&caddy).expect("serialization should succeed");

        // Network serializes to its name string for template compatibility
        assert_eq!(json["networks"][0], "proxy_network");
    }
}
