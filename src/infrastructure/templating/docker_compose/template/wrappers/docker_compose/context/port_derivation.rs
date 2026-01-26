//! Port derivation functions for Docker Compose services
//!
//! This module implements PORT-01 through PORT-11 rules from the refactoring plan,
//! deriving port bindings for each service based on configuration.
//!
//! ## Design Principles
//!
//! - Single source of truth for port exposure logic
//! - TLS-aware: ports are hidden when Caddy handles TLS termination
//! - Type-safe: uses `PortBinding` domain type
//! - Self-documenting: each port has a description for YAML comments
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
//! | PORT-07 | Grafana 3000 exposed only without TLS                 |
//! | PORT-08 | Grafana 3000 NOT exposed with TLS                     |
//! | PORT-09 | Caddy always exposes 80, 443, 443/udp                 |
//! | PORT-10 | Prometheus 9090 on localhost only                     |
//! | PORT-11 | `MySQL` no exposed ports                              |

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

/// Derives port bindings for the Caddy service
///
/// Implements PORT-09: Caddy always exposes 80, 443, 443/udp
///
/// These ports are required for:
/// - Port 80: ACME HTTP-01 challenge for certificate renewal
/// - Port 443: HTTPS traffic
/// - Port 443/udp: HTTP/3 (QUIC) support
#[must_use]
pub fn derive_caddy_ports() -> Vec<PortBinding> {
    vec![
        PortBinding::tcp(80, "HTTP (ACME HTTP-01 challenge)"),
        PortBinding::tcp(443, "HTTPS"),
        PortBinding::udp(443, "HTTP/3 (QUIC)"),
    ]
}

/// Derives port bindings for the Prometheus service
///
/// Implements PORT-10: Prometheus 9090 on localhost only
///
/// Prometheus is bound to localhost to prevent external access.
/// Grafana accesses it via Docker network (`http://prometheus:9090`).
#[must_use]
pub fn derive_prometheus_ports() -> Vec<PortBinding> {
    vec![PortBinding::localhost_tcp(
        9090,
        "Prometheus metrics (localhost only)",
    )]
}

/// Derives port bindings for the Grafana service
///
/// Implements PORT-07 and PORT-08:
/// - Without TLS: expose port 3000 directly
/// - With TLS: don't expose (Caddy handles it)
///
/// # Arguments
///
/// * `has_tls` - Whether TLS is enabled (Caddy handles Grafana access)
#[must_use]
pub fn derive_grafana_ports(has_tls: bool) -> Vec<PortBinding> {
    // PORT-07: Grafana 3000 exposed only without TLS
    // PORT-08: Grafana 3000 NOT exposed with TLS
    if has_tls {
        vec![]
    } else {
        vec![PortBinding::tcp(3000, "Grafana dashboard")]
    }
}

/// Derives port bindings for the `MySQL` service
///
/// Implements PORT-11: `MySQL` has no exposed ports
///
/// `MySQL` is accessed only via Docker network by the tracker service.
/// It should never be exposed to the host network for security.
#[must_use]
pub fn derive_mysql_ports() -> Vec<PortBinding> {
    vec![]
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

    // =========================================================================
    // Caddy port derivation tests
    // =========================================================================

    mod caddy_ports {
        use super::*;

        #[test]
        fn it_should_expose_http_for_acme_challenge() {
            // PORT-09: Caddy always exposes 80
            let ports = derive_caddy_ports();

            let http_port = ports
                .iter()
                .find(|p| p.host_port() == 80 && p.protocol() == Protocol::Tcp);

            assert!(http_port.is_some());
            assert!(http_port.unwrap().description().contains("ACME"));
        }

        #[test]
        fn it_should_expose_https_on_443() {
            // PORT-09: Caddy always exposes 443
            let ports = derive_caddy_ports();

            let https_port = ports
                .iter()
                .find(|p| p.host_port() == 443 && p.protocol() == Protocol::Tcp);

            assert!(https_port.is_some());
            assert!(https_port.unwrap().description().contains("HTTPS"));
        }

        #[test]
        fn it_should_expose_quic_on_443_udp() {
            // PORT-09: Caddy always exposes 443/udp for HTTP/3
            let ports = derive_caddy_ports();

            let quic_port = ports
                .iter()
                .find(|p| p.host_port() == 443 && p.protocol() == Protocol::Udp);

            assert!(quic_port.is_some());
            assert!(quic_port.unwrap().description().contains("QUIC"));
        }

        #[test]
        fn it_should_return_exactly_three_ports() {
            let ports = derive_caddy_ports();

            assert_eq!(ports.len(), 3);
        }
    }

    // =========================================================================
    // Prometheus port derivation tests
    // =========================================================================

    mod prometheus_ports {
        use std::net::{IpAddr, Ipv4Addr};

        use super::*;

        #[test]
        fn it_should_expose_9090_on_localhost_only() {
            // PORT-10: Prometheus 9090 on localhost only
            let ports = derive_prometheus_ports();

            assert_eq!(ports.len(), 1);
            assert_eq!(ports[0].host_port(), 9090);
            assert_eq!(ports[0].host_ip(), Some(IpAddr::V4(Ipv4Addr::LOCALHOST)));
        }

        #[test]
        fn it_should_use_tcp_protocol() {
            let ports = derive_prometheus_ports();

            assert_eq!(ports[0].protocol(), Protocol::Tcp);
        }
    }

    // =========================================================================
    // Grafana port derivation tests
    // =========================================================================

    mod grafana_ports {
        use super::*;

        #[test]
        fn it_should_expose_3000_without_tls() {
            // PORT-07: Grafana 3000 exposed only without TLS
            let ports = derive_grafana_ports(false);

            assert_eq!(ports.len(), 1);
            assert_eq!(ports[0].host_port(), 3000);
        }

        #[test]
        fn it_should_not_expose_port_with_tls() {
            // PORT-08: Grafana 3000 NOT exposed with TLS
            let ports = derive_grafana_ports(true);

            assert!(ports.is_empty());
        }
    }

    // =========================================================================
    // MySQL port derivation tests
    // =========================================================================

    mod mysql_ports {
        use super::*;

        #[test]
        fn it_should_not_expose_any_ports() {
            // PORT-11: MySQL no exposed ports
            let ports = derive_mysql_ports();

            assert!(ports.is_empty());
        }
    }
}
