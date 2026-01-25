//! Port binding domain types for Docker Compose topology
//!
//! This module defines port binding types that represent how container ports
//! are exposed to the host network.
//!
//! ## Design Principles
//!
//! - Type-safe port bindings (no string typos in port specifications)
//! - Self-documenting with descriptions for sysadmin context
//! - Protocol-aware (UDP vs TCP have separate port spaces)
//! - Supports host IP binding (e.g., localhost-only for Prometheus)
//!
//! ## Port Rules (from refactoring plan)
//!
//! The following rules are encoded in the port derivation logic:
//!
//! - PORT-01: Tracker needs ports if UDP OR HTTP without TLS OR API without TLS
//! - PORT-02: UDP ports always exposed (UDP doesn't use TLS)
//! - PORT-03: HTTP ports WITHOUT TLS exposed directly
//! - PORT-04: HTTP ports WITH TLS NOT exposed (Caddy handles)
//! - PORT-05: API exposed only when no TLS
//! - PORT-06: API NOT exposed when TLS
//! - PORT-07: Grafana 3000 exposed only without TLS
//! - PORT-08: Grafana 3000 NOT exposed with TLS
//! - PORT-09: Caddy always exposes 80, 443, 443/udp
//! - PORT-10: Prometheus 9090 on localhost only
//! - PORT-11: `MySQL` no exposed ports

use std::fmt;
use std::net::IpAddr;

use crate::domain::tracker::Protocol;

/// A port binding for Docker Compose
///
/// Represents how a container port is exposed to the host network.
/// The description provides context for sysadmins inspecting the
/// rendered docker-compose.yml file.
///
/// # Examples
///
/// ```rust
/// use std::net::IpAddr;
/// use torrust_tracker_deployer_lib::domain::topology::PortBinding;
/// use torrust_tracker_deployer_lib::domain::tracker::Protocol;
///
/// // UDP tracker port - exposed on all interfaces
/// let udp_port = PortBinding::new(
///     6969,
///     6969,
///     Protocol::Udp,
///     None,
///     "BitTorrent UDP announce",
/// );
/// assert_eq!(udp_port.docker_compose_binding(), "6969:6969/udp");
///
/// // Prometheus - localhost only
/// let prometheus_port = PortBinding::new(
///     9090,
///     9090,
///     Protocol::Tcp,
///     Some("127.0.0.1".parse().unwrap()),
///     "Prometheus metrics (localhost only)",
/// );
/// assert_eq!(prometheus_port.docker_compose_binding(), "127.0.0.1:9090:9090");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortBinding {
    /// Port on the host machine
    host_port: u16,
    /// Port inside the container
    container_port: u16,
    /// Network protocol (TCP or UDP)
    protocol: Protocol,
    /// Host IP to bind to (None = 0.0.0.0, all interfaces)
    host_ip: Option<IpAddr>,
    /// Description of what this port is for (rendered as YAML comment)
    description: &'static str,
}

impl PortBinding {
    /// Creates a new port binding
    ///
    /// # Arguments
    ///
    /// * `host_port` - Port number on the host machine
    /// * `container_port` - Port number inside the container
    /// * `protocol` - Network protocol (TCP or UDP)
    /// * `host_ip` - Optional IP to bind to (None = all interfaces)
    /// * `description` - Short description for sysadmin documentation
    #[must_use]
    pub fn new(
        host_port: u16,
        container_port: u16,
        protocol: Protocol,
        host_ip: Option<IpAddr>,
        description: &'static str,
    ) -> Self {
        Self {
            host_port,
            container_port,
            protocol,
            host_ip,
            description,
        }
    }

    /// Creates a TCP port binding on all interfaces
    ///
    /// Convenience constructor for common TCP port mappings where
    /// host and container ports are the same.
    #[must_use]
    pub fn tcp(port: u16, description: &'static str) -> Self {
        Self::new(port, port, Protocol::Tcp, None, description)
    }

    /// Creates a UDP port binding on all interfaces
    ///
    /// Convenience constructor for common UDP port mappings where
    /// host and container ports are the same.
    #[must_use]
    pub fn udp(port: u16, description: &'static str) -> Self {
        Self::new(port, port, Protocol::Udp, None, description)
    }

    /// Creates a TCP port binding on localhost only
    ///
    /// Used for services that should not be exposed externally,
    /// like Prometheus.
    #[must_use]
    pub fn localhost_tcp(port: u16, description: &'static str) -> Self {
        Self::new(
            port,
            port,
            Protocol::Tcp,
            Some(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
            description,
        )
    }

    /// Returns the host port
    #[must_use]
    pub fn host_port(&self) -> u16 {
        self.host_port
    }

    /// Returns the container port
    #[must_use]
    pub fn container_port(&self) -> u16 {
        self.container_port
    }

    /// Returns the protocol
    #[must_use]
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }

    /// Returns the host IP if specified
    #[must_use]
    pub fn host_ip(&self) -> Option<IpAddr> {
        self.host_ip
    }

    /// Returns the description
    #[must_use]
    pub fn description(&self) -> &'static str {
        self.description
    }

    /// Returns the Docker Compose port binding string
    ///
    /// Formats the port binding for docker-compose.yml:
    /// - `"80:80"` - TCP on all interfaces
    /// - `"6969:6969/udp"` - UDP on all interfaces
    /// - `"127.0.0.1:9090:9090"` - TCP on localhost only
    #[must_use]
    pub fn docker_compose_binding(&self) -> String {
        let protocol_suffix = match self.protocol {
            Protocol::Tcp => String::new(),
            Protocol::Udp => "/udp".to_string(),
        };

        match self.host_ip {
            Some(ip) => format!(
                "{}:{}:{}{}",
                ip, self.host_port, self.container_port, protocol_suffix
            ),
            None => format!(
                "{}:{}{}",
                self.host_port, self.container_port, protocol_suffix
            ),
        }
    }
}

impl fmt::Display for PortBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.docker_compose_binding())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod port_binding_creation {
        use super::*;

        #[test]
        fn it_should_create_tcp_port_binding_on_all_interfaces() {
            let port = PortBinding::tcp(7070, "HTTP tracker announce");

            assert_eq!(port.host_port(), 7070);
            assert_eq!(port.container_port(), 7070);
            assert_eq!(port.protocol(), Protocol::Tcp);
            assert!(port.host_ip().is_none());
            assert_eq!(port.description(), "HTTP tracker announce");
        }

        #[test]
        fn it_should_create_udp_port_binding_on_all_interfaces() {
            let port = PortBinding::udp(6969, "BitTorrent UDP announce");

            assert_eq!(port.host_port(), 6969);
            assert_eq!(port.container_port(), 6969);
            assert_eq!(port.protocol(), Protocol::Udp);
            assert!(port.host_ip().is_none());
            assert_eq!(port.description(), "BitTorrent UDP announce");
        }

        #[test]
        fn it_should_create_localhost_only_port_binding() {
            let port = PortBinding::localhost_tcp(9090, "Prometheus metrics");

            assert_eq!(port.host_port(), 9090);
            assert_eq!(port.protocol(), Protocol::Tcp);
            assert_eq!(
                port.host_ip(),
                Some(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST))
            );
        }

        #[test]
        fn it_should_create_port_binding_with_different_host_and_container_ports() {
            let port = PortBinding::new(8080, 80, Protocol::Tcp, None, "Remapped HTTP");

            assert_eq!(port.host_port(), 8080);
            assert_eq!(port.container_port(), 80);
        }
    }

    mod docker_compose_binding {
        use super::*;

        #[test]
        fn it_should_format_tcp_port_without_protocol_suffix() {
            let port = PortBinding::tcp(7070, "HTTP tracker");

            assert_eq!(port.docker_compose_binding(), "7070:7070");
        }

        #[test]
        fn it_should_format_udp_port_with_udp_suffix() {
            let port = PortBinding::udp(6969, "UDP tracker");

            assert_eq!(port.docker_compose_binding(), "6969:6969/udp");
        }

        #[test]
        fn it_should_format_localhost_binding_with_ip_prefix() {
            let port = PortBinding::localhost_tcp(9090, "Prometheus");

            assert_eq!(port.docker_compose_binding(), "127.0.0.1:9090:9090");
        }

        #[test]
        fn it_should_format_different_host_and_container_ports() {
            let port = PortBinding::new(8080, 80, Protocol::Tcp, None, "Remapped");

            assert_eq!(port.docker_compose_binding(), "8080:80");
        }

        #[test]
        fn it_should_format_udp_with_localhost_binding() {
            let port = PortBinding::new(
                5353,
                53,
                Protocol::Udp,
                Some(IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)),
                "Local DNS",
            );

            assert_eq!(port.docker_compose_binding(), "127.0.0.1:5353:53/udp");
        }
    }

    mod display_trait {
        use super::*;

        #[test]
        fn it_should_display_as_docker_compose_binding() {
            let port = PortBinding::tcp(80, "HTTP");

            assert_eq!(format!("{port}"), "80:80");
        }
    }
}
