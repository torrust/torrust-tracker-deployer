//! Fixed port definitions for services without configuration
//!
//! This module provides port bindings for services that have static port
//! configurations (no user-configurable ports). These are services where
//! the port exposure is fixed by the service's nature, not by configuration.
//!
//! ## Services
//!
//! - **Caddy**: Always exposes 80, 443, and 443/udp for TLS termination
//! - **MySQL**: Never exposes ports (internal-only database access)
//!
//! ## Port Rules Reference
//!
//! | Rule    | Service | Description                               |
//! |---------|---------|-------------------------------------------|
//! | PORT-09 | Caddy   | Always exposes 80, 443, 443/udp           |
//! | PORT-11 | MySQL   | No exposed ports (internal only)          |

use super::PortBinding;

/// Derives port bindings for the Caddy TLS proxy service
///
/// Implements PORT-09: Caddy always exposes 80, 443, 443/udp
///
/// These ports are required for:
/// - **Port 80/tcp**: ACME HTTP-01 challenge for Let's Encrypt certificate renewal
/// - **Port 443/tcp**: HTTPS traffic for all proxied services
/// - **Port 443/udp**: HTTP/3 (QUIC) support for modern browsers
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::topology::caddy_ports;
///
/// let ports = caddy_ports();
/// assert_eq!(ports.len(), 3);
/// assert!(ports.iter().any(|p| p.host_port() == 80));
/// assert!(ports.iter().any(|p| p.host_port() == 443));
/// ```
#[must_use]
pub fn caddy_ports() -> Vec<PortBinding> {
    vec![
        PortBinding::tcp(80, "HTTP (ACME HTTP-01 challenge)"),
        PortBinding::tcp(443, "HTTPS"),
        PortBinding::udp(443, "HTTP/3 (QUIC)"),
    ]
}

/// Derives port bindings for the MySQL database service
///
/// Implements PORT-11: MySQL has no exposed ports
///
/// MySQL is accessed only via Docker network by the tracker service.
/// It should never be exposed to the host network for security reasons.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::topology::mysql_ports;
///
/// let ports = mysql_ports();
/// assert!(ports.is_empty());
/// ```
#[must_use]
pub fn mysql_ports() -> Vec<PortBinding> {
    vec![]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::tracker::Protocol;

    // =========================================================================
    // Caddy port tests (PORT-09)
    // =========================================================================

    mod caddy {
        use super::*;

        #[test]
        fn it_should_expose_port_80_for_acme_challenge() {
            let ports = caddy_ports();

            let port_80 = ports
                .iter()
                .find(|p| p.host_port() == 80 && p.protocol() == Protocol::Tcp);

            assert!(port_80.is_some());
            assert!(port_80.unwrap().description().contains("ACME"));
        }

        #[test]
        fn it_should_expose_port_443_tcp_for_https() {
            let ports = caddy_ports();

            let port_443_tcp = ports
                .iter()
                .find(|p| p.host_port() == 443 && p.protocol() == Protocol::Tcp);

            assert!(port_443_tcp.is_some());
            assert!(port_443_tcp.unwrap().description().contains("HTTPS"));
        }

        #[test]
        fn it_should_expose_port_443_udp_for_http3() {
            let ports = caddy_ports();

            let port_443_udp = ports
                .iter()
                .find(|p| p.host_port() == 443 && p.protocol() == Protocol::Udp);

            assert!(port_443_udp.is_some());
            assert!(port_443_udp.unwrap().description().contains("HTTP/3"));
        }

        #[test]
        fn it_should_have_exactly_three_ports() {
            let ports = caddy_ports();

            assert_eq!(ports.len(), 3);
        }
    }

    // =========================================================================
    // MySQL port tests (PORT-11)
    // =========================================================================

    mod mysql {
        use super::*;

        #[test]
        fn it_should_have_no_exposed_ports() {
            // PORT-11: MySQL no exposed ports
            let ports = mysql_ports();

            assert!(ports.is_empty());
        }
    }
}
