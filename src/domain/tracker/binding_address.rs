//! Binding address value object for tracker services
//!
//! This module provides a type-safe representation of socket addresses
//! with protocol information, used for validating tracker configurations.

use std::fmt;
use std::net::SocketAddr;

use super::Protocol;

/// A binding address combining socket address and protocol
///
/// Represents a complete socket binding specification including both the
/// network address (IP + port) and the protocol (UDP or TCP). This ensures
/// that socket address uniqueness validation accounts for protocol differences.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::tracker::{BindingAddress, Protocol};
///
/// let udp_addr = BindingAddress::new(
///     "0.0.0.0:6969".parse().unwrap(),
///     Protocol::Udp
/// );
///
/// let tcp_addr = BindingAddress::new(
///     "0.0.0.0:7070".parse().unwrap(),
///     Protocol::Tcp
/// );
///
/// assert_eq!(udp_addr.socket().port(), 6969);
/// assert_eq!(tcp_addr.protocol(), Protocol::Tcp);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindingAddress {
    socket: SocketAddr,
    protocol: Protocol,
}

impl BindingAddress {
    /// Creates a new binding address
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tracker::{BindingAddress, Protocol};
    ///
    /// let addr = BindingAddress::new(
    ///     "0.0.0.0:6969".parse().unwrap(),
    ///     Protocol::Udp
    /// );
    /// ```
    #[must_use]
    pub fn new(socket: SocketAddr, protocol: Protocol) -> Self {
        Self { socket, protocol }
    }

    /// Returns the socket address
    #[must_use]
    pub fn socket(&self) -> &SocketAddr {
        &self.socket
    }

    /// Returns the protocol
    #[must_use]
    pub fn protocol(&self) -> Protocol {
        self.protocol
    }
}

impl fmt::Display for BindingAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.socket, self.protocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_binding_address_with_udp_protocol() {
        let socket: SocketAddr = "0.0.0.0:6969".parse().unwrap();
        let addr = BindingAddress::new(socket, Protocol::Udp);

        assert_eq!(addr.socket(), &socket);
        assert_eq!(addr.protocol(), Protocol::Udp);
    }

    #[test]
    fn it_should_create_binding_address_with_tcp_protocol() {
        let socket: SocketAddr = "127.0.0.1:7070".parse().unwrap();
        let addr = BindingAddress::new(socket, Protocol::Tcp);

        assert_eq!(addr.socket(), &socket);
        assert_eq!(addr.protocol(), Protocol::Tcp);
    }

    #[test]
    fn it_should_display_binding_address_with_protocol() {
        let addr = BindingAddress::new("0.0.0.0:6969".parse().unwrap(), Protocol::Udp);
        assert_eq!(addr.to_string(), "0.0.0.0:6969 (UDP)");

        let addr = BindingAddress::new("127.0.0.1:7070".parse().unwrap(), Protocol::Tcp);
        assert_eq!(addr.to_string(), "127.0.0.1:7070 (TCP)");
    }

    #[test]
    fn it_should_consider_same_socket_different_protocol_as_different() {
        let udp_addr = BindingAddress::new("0.0.0.0:7070".parse().unwrap(), Protocol::Udp);
        let tcp_addr = BindingAddress::new("0.0.0.0:7070".parse().unwrap(), Protocol::Tcp);

        assert_ne!(udp_addr, tcp_addr);
    }

    #[test]
    fn it_should_consider_same_socket_same_protocol_as_equal() {
        let addr1 = BindingAddress::new("0.0.0.0:7070".parse().unwrap(), Protocol::Tcp);
        let addr2 = BindingAddress::new("0.0.0.0:7070".parse().unwrap(), Protocol::Tcp);

        assert_eq!(addr1, addr2);
    }

    #[test]
    fn it_should_consider_different_ips_same_port_as_different() {
        let addr1 = BindingAddress::new("192.168.1.10:7070".parse().unwrap(), Protocol::Tcp);
        let addr2 = BindingAddress::new("192.168.1.20:7070".parse().unwrap(), Protocol::Tcp);

        assert_ne!(addr1, addr2);
    }

    #[test]
    fn it_should_be_usable_as_hash_map_key() {
        use std::collections::HashMap;

        let mut map = HashMap::new();
        let addr = BindingAddress::new("0.0.0.0:6969".parse().unwrap(), Protocol::Udp);
        map.insert(addr, "UDP Tracker");

        assert_eq!(map.get(&addr), Some(&"UDP Tracker"));
    }
}
