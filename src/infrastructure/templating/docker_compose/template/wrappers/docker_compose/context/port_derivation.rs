//! Port derivation functions for Docker Compose services
//!
//! This module implements PORT-01 through PORT-06 rules from the refactoring plan,
//! deriving port bindings for the Tracker service based on configuration.
//!
//! ## Design Principles
//!
//! - Single source of truth for port exposure logic
//! - TLS-aware: ports are hidden when Caddy handles TLS termination
//! - Type-safe: uses `PortBinding` domain type
//! - Self-documenting: each port has a description for YAML comments
//!
//! ## Note on Port Derivation Migration
//!
//! Port derivation for other services has been moved to the domain layer:
//! - Tracker: `TrackerConfig::derive_ports()` (domain/tracker/config)
//! - Prometheus: `PrometheusConfig::derive_ports()` (domain/prometheus)
//! - Grafana: `GrafanaConfig::derive_ports()` (domain/grafana)
//! - Caddy: `domain::topology::caddy_ports()` (domain/topology)
//! - MySQL: `domain::topology::mysql_ports()` (domain/topology)
//!
//! This file contains legacy tracker port derivation for backward compatibility
//! during the migration. See `TrackerServiceConfig::from_domain_config()` for
//! the new domain-based approach.
//!
//! ## Port Rules Reference
//!
//! | Rule    | Description                                           |
//! |---------|-------------------------------------------------------|
//! | PORT-01 | Tracker needs ports if UDP OR HTTP/API without TLS    |
//! | PORT-02 | UDP ports always exposed (no TLS for UDP)             |
//! | PORT-03 | HTTP ports WITHOUT TLS exposed directly               |
//! | PORT-04 | HTTP ports WITH TLS NOT exposed (Caddy handles)       |
//! | PORT-05 | API exposed only when no TLS                          |
//! | PORT-06 | API NOT exposed when TLS                              |

use crate::domain::topology::PortBinding;

/// Derives port bindings for the Tracker service
///
/// Implements PORT-01 through PORT-06:
/// - UDP ports are always exposed (PORT-02)
/// - HTTP ports without TLS are exposed (PORT-03)
/// - HTTP ports with TLS are NOT exposed (PORT-04)
/// - API port exposed only when no TLS (PORT-05, PORT-06)
///
/// # Arguments
///
/// * `udp_ports` - UDP tracker port numbers
/// * `http_ports_without_tls` - HTTP tracker ports that don't use TLS
/// * `http_api_port` - HTTP API port number
/// * `http_api_has_tls` - Whether the HTTP API uses TLS (Caddy handles it)
#[must_use]
pub fn derive_tracker_ports(
    udp_ports: &[u16],
    http_ports_without_tls: &[u16],
    http_api_port: u16,
    http_api_has_tls: bool,
) -> Vec<PortBinding> {
    let mut ports = Vec::new();

    // PORT-02: UDP ports always exposed (UDP doesn't use TLS)
    for &port in udp_ports {
        ports.push(PortBinding::udp(port, "BitTorrent UDP announce"));
    }

    // PORT-03: HTTP ports WITHOUT TLS exposed directly
    // PORT-04: HTTP ports WITH TLS NOT exposed (handled by Caddy)
    for &port in http_ports_without_tls {
        ports.push(PortBinding::tcp(port, "HTTP tracker announce"));
    }

    // PORT-05: API exposed only when no TLS
    // PORT-06: API NOT exposed when TLS (Caddy handles it)
    if !http_api_has_tls {
        ports.push(PortBinding::tcp(
            http_api_port,
            "HTTP API (stats/whitelist)",
        ));
    }

    ports
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tracker::Protocol;

    // =========================================================================
    // Tracker port derivation tests
    // =========================================================================

    mod tracker_ports {
        use super::*;

        #[test]
        fn it_should_expose_udp_ports_always() {
            // PORT-02: UDP ports always exposed
            let ports = derive_tracker_ports(&[6969, 6868], &[], 1212, false);

            let udp_ports: Vec<_> = ports
                .iter()
                .filter(|p| p.protocol() == Protocol::Udp)
                .collect();

            assert_eq!(udp_ports.len(), 2);
            assert_eq!(udp_ports[0].host_port(), 6969);
            assert_eq!(udp_ports[1].host_port(), 6868);
        }

        #[test]
        fn it_should_expose_http_ports_without_tls() {
            // PORT-03: HTTP ports WITHOUT TLS exposed directly
            let ports = derive_tracker_ports(&[], &[7070, 8080], 1212, false);

            let http_ports: Vec<_> = ports
                .iter()
                .filter(|p| p.protocol() == Protocol::Tcp && p.host_port() != 1212)
                .collect();

            assert_eq!(http_ports.len(), 2);
            assert_eq!(http_ports[0].host_port(), 7070);
            assert_eq!(http_ports[1].host_port(), 8080);
        }

        #[test]
        fn it_should_expose_api_port_when_no_tls() {
            // PORT-05: API exposed only when no TLS
            let ports = derive_tracker_ports(&[], &[], 1212, false);

            let api_port = ports.iter().find(|p| p.host_port() == 1212);

            assert!(api_port.is_some());
            assert_eq!(api_port.unwrap().protocol(), Protocol::Tcp);
        }

        #[test]
        fn it_should_not_expose_api_port_when_tls_enabled() {
            // PORT-06: API NOT exposed when TLS
            let ports = derive_tracker_ports(&[], &[], 1212, true);

            let api_port = ports.iter().find(|p| p.host_port() == 1212);

            assert!(api_port.is_none());
        }

        #[test]
        fn it_should_return_empty_when_all_ports_behind_tls() {
            // All HTTP ports have TLS, no UDP ports
            let ports = derive_tracker_ports(&[], &[], 1212, true);

            assert!(ports.is_empty());
        }

        #[test]
        fn it_should_include_descriptions_for_all_ports() {
            let ports = derive_tracker_ports(&[6969], &[7070], 1212, false);

            for port in &ports {
                assert!(!port.description().is_empty());
            }
        }
    }
}
