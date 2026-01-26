//! Caddy TLS reverse proxy configuration domain type
//!
//! This module defines the Caddy configuration domain type which implements
//! the `PortDerivation` and `NetworkDerivation` traits following the same
//! pattern as other services.
//!
//! ## Port Rules Reference
//!
//! | Rule    | Description                               |
//! |---------|-------------------------------------------|
//! | PORT-09 | Caddy always exposes 80, 443, 443/udp     |
//!
//! ## Network Rules Reference
//!
//! | Rule   | Description                               |
//! |--------|-------------------------------------------|
//! | NET-09 | Caddy always connects to Proxy network    |

use serde::{Deserialize, Serialize};

use crate::domain::topology::{
    EnabledServices, Network, NetworkDerivation, PortBinding, PortDerivation,
};

/// Caddy TLS reverse proxy configuration
///
/// Caddy is a special service with fixed behavior:
/// - Always exposes ports 80 (ACME), 443 (HTTPS), 443/udp (QUIC)
/// - Always connects to the Proxy network
///
/// Unlike other services, Caddy doesn't have user-configurable port behavior,
/// but it still implements `PortDerivation` for consistency.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::caddy::CaddyConfig;
/// use torrust_tracker_deployer_lib::domain::topology::PortDerivation;
///
/// let config = CaddyConfig::new();
/// let ports = config.derive_ports();
/// assert_eq!(ports.len(), 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct CaddyConfig {
    // Caddy has no configurable fields for port/network derivation.
    // This is intentionally empty - the behavior is fixed.
    // Future: Could add ACME email, custom certificates, etc.
}

impl CaddyConfig {
    /// Creates a new `CaddyConfig`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::caddy::CaddyConfig;
    ///
    /// let config = CaddyConfig::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl PortDerivation for CaddyConfig {
    /// Derives port bindings for the Caddy TLS proxy service
    ///
    /// Implements PORT-09: Caddy always exposes 80, 443, 443/udp
    ///
    /// These ports are required for:
    /// - **Port 80/tcp**: ACME HTTP-01 challenge for Let's Encrypt certificate renewal
    /// - **Port 443/tcp**: HTTPS traffic for all proxied services
    /// - **Port 443/udp**: HTTP/3 (QUIC) support for modern browsers
    fn derive_ports(&self) -> Vec<PortBinding> {
        vec![
            PortBinding::tcp(80, "HTTP (ACME HTTP-01 challenge)"),
            PortBinding::tcp(443, "HTTPS"),
            PortBinding::udp(443, "HTTP/3 (QUIC)"),
        ]
    }
}

impl NetworkDerivation for CaddyConfig {
    /// Derives network assignments for the Caddy service
    ///
    /// Implements NET-09: Caddy always connects to Proxy network only
    fn derive_networks(&self, _enabled_services: &EnabledServices) -> Vec<Network> {
        vec![Network::Proxy]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::topology::Service;
    use crate::domain::tracker::Protocol;

    // =========================================================================
    // Constructor tests
    // =========================================================================

    mod constructor {
        use super::*;

        #[test]
        fn it_should_create_caddy_config() {
            let config = CaddyConfig::new();
            assert_eq!(config, CaddyConfig::default());
        }

        #[test]
        fn it_should_implement_default() {
            let config = CaddyConfig::default();
            assert_eq!(config, CaddyConfig::new());
        }
    }

    // =========================================================================
    // PortDerivation tests (PORT-09)
    // =========================================================================

    mod port_derivation {
        use super::*;

        #[test]
        fn it_should_expose_port_80_for_acme_challenge() {
            let config = CaddyConfig::new();
            let ports = config.derive_ports();

            let port_80 = ports
                .iter()
                .find(|p| p.host_port() == 80 && p.protocol() == Protocol::Tcp);

            assert!(port_80.is_some());
            assert!(port_80.unwrap().description().contains("ACME"));
        }

        #[test]
        fn it_should_expose_port_443_tcp_for_https() {
            let config = CaddyConfig::new();
            let ports = config.derive_ports();

            let port_443_tcp = ports
                .iter()
                .find(|p| p.host_port() == 443 && p.protocol() == Protocol::Tcp);

            assert!(port_443_tcp.is_some());
            assert!(port_443_tcp.unwrap().description().contains("HTTPS"));
        }

        #[test]
        fn it_should_expose_port_443_udp_for_quic() {
            let config = CaddyConfig::new();
            let ports = config.derive_ports();

            let port_443_udp = ports
                .iter()
                .find(|p| p.host_port() == 443 && p.protocol() == Protocol::Udp);

            assert!(port_443_udp.is_some());
            assert!(port_443_udp.unwrap().description().contains("QUIC"));
        }

        #[test]
        fn it_should_expose_exactly_three_ports() {
            let config = CaddyConfig::new();
            let ports = config.derive_ports();

            assert_eq!(ports.len(), 3);
        }
    }

    // =========================================================================
    // NetworkDerivation tests (NET-09)
    // =========================================================================

    mod network_derivation {
        use super::*;

        #[test]
        fn it_should_connect_to_proxy_network() {
            let config = CaddyConfig::new();
            let enabled = EnabledServices::from(&[]);
            let networks = config.derive_networks(&enabled);

            assert_eq!(networks, vec![Network::Proxy]);
        }

        #[test]
        fn it_should_connect_only_to_proxy_network_regardless_of_enabled_services() {
            let config = CaddyConfig::new();
            let enabled = EnabledServices::from(&[
                Service::Tracker,
                Service::Prometheus,
                Service::Grafana,
                Service::MySQL,
            ]);
            let networks = config.derive_networks(&enabled);

            // NET-09: Caddy only connects to Proxy network
            assert_eq!(networks, vec![Network::Proxy]);
        }
    }
}
