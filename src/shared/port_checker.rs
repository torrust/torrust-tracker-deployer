//! TCP Port Connectivity Checker
//!
//! This module provides functionality to check if a TCP port is open and accepting connections
//! on a remote host. It performs pure TCP connectivity tests without any protocol-specific logic.
//!
//! ## Key Features
//!
//! - Pure TCP port connectivity testing
//! - No protocol-specific handshakes or authentication attempts
//! - Lightweight and fast connection testing
//! - Configurable connection timeouts
//! - Clean success/failure states without error noise
//!
//! ## Usage
//!
//! ```rust,no_run
//! use std::net::{SocketAddr, IpAddr, Ipv4Addr};
//! use std::time::Duration;
//! use torrust_tracker_deploy::shared::PortChecker;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let checker = PortChecker::new();
//! let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 22);
//! let is_open = checker.is_port_open(socket_addr)?;
//! if is_open {
//!     println!("Port is open and accepting connections");
//! } else {
//!     println!("Port is not available");
//! }
//! # Ok(())
//! # }
//! ```

use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
use tracing::debug;

/// TCP Port connectivity checker errors
#[derive(Debug, thiserror::Error)]
pub enum PortCheckerError {
    /// Network operation failed (e.g., host unreachable, DNS resolution failed)
    #[error("Failed to test port connectivity to {socket_addr}: {source}")]
    NetworkError {
        socket_addr: SocketAddr,
        #[source]
        source: std::io::Error,
    },
}

/// Result type for port checking operations
pub type Result<T> = std::result::Result<T, PortCheckerError>;

/// TCP Port Connectivity Checker
///
/// This checker performs lightweight TCP connection tests to determine if a port
/// is open and accepting connections on a given host. It does not perform any
/// protocol-specific handshakes or data exchange.
///
/// The checker uses TCP connect with configurable timeouts to quickly determine
/// port availability without creating persistent connections.
#[derive(Debug)]
pub struct PortChecker {
    /// Connection timeout for TCP connection attempts
    connect_timeout: Duration,
}

impl Default for PortChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl PortChecker {
    /// Create a new port checker with default settings
    ///
    /// Default connection timeout is 3 seconds.
    #[must_use]
    pub fn new() -> Self {
        Self {
            connect_timeout: Duration::from_secs(3),
        }
    }

    /// Create a new port checker with custom connection timeout
    ///
    /// # Arguments
    /// * `connect_timeout` - Timeout duration for connection attempts
    #[must_use]
    pub fn with_timeout(connect_timeout: Duration) -> Self {
        Self { connect_timeout }
    }

    /// Check if a TCP port is open and accepting connections
    ///
    /// This method attempts a TCP connection to test port availability.
    /// It distinguishes between:
    /// - Port open and accepting connections
    /// - Port closed, filtered, or host unreachable
    /// - Network errors (DNS resolution, routing issues)
    ///
    /// # Arguments
    /// * `socket_addr` - The socket address (IP and port) to test
    ///
    /// # Returns
    /// * `Ok(true)` - Port is open and accepting connections
    /// * `Ok(false)` - Port is not available (closed, filtered, or unreachable)
    /// * `Err(PortCheckerError)` - Network error occurred during testing
    ///
    /// # Errors
    /// Returns an error only for unexpected network conditions that prevent
    /// the connectivity test from completing (e.g., DNS resolution failures,
    /// invalid socket addresses).
    ///
    /// Connection refused, timeouts, and unreachable hosts are returned as
    /// `Ok(false)` since they represent valid "port not available" states.
    pub fn is_port_open(&self, socket_addr: SocketAddr) -> Result<bool> {
        debug!(
            socket_addr = %socket_addr,
            timeout = ?self.connect_timeout,
            "Testing TCP port connectivity"
        );

        match TcpStream::connect_timeout(&socket_addr, self.connect_timeout) {
            Ok(_stream) => {
                // Connection successful - port is open and accepting connections
                debug!(
                    socket_addr = %socket_addr,
                    "TCP port is open and accepting connections"
                );
                Ok(true)
            }
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::ConnectionRefused => {
                        // Port is closed or no service is listening
                        debug!(
                            socket_addr = %socket_addr,
                            "TCP port is closed (connection refused)"
                        );
                        Ok(false)
                    }
                    std::io::ErrorKind::TimedOut => {
                        // Connection timed out - port may be filtered or host unreachable
                        debug!(
                            socket_addr = %socket_addr,
                            timeout = ?self.connect_timeout,
                            "TCP port connectivity test timed out"
                        );
                        Ok(false)
                    }
                    std::io::ErrorKind::NetworkUnreachable
                    | std::io::ErrorKind::HostUnreachable => {
                        // Host or network is unreachable
                        debug!(
                            socket_addr = %socket_addr,
                            "Host or network unreachable"
                        );
                        Ok(false)
                    }
                    _ => {
                        // Other network errors that prevent testing
                        debug!(
                            socket_addr = %socket_addr,
                            error = %e,
                            "Network error during port connectivity test"
                        );
                        Err(PortCheckerError::NetworkError {
                            socket_addr,
                            source: e,
                        })
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn it_should_create_port_checker_with_defaults() {
        let checker = PortChecker::new();
        assert_eq!(checker.connect_timeout, Duration::from_secs(3));
    }

    #[test]
    fn it_should_create_port_checker_with_custom_timeout() {
        let timeout = Duration::from_secs(10);
        let checker = PortChecker::with_timeout(timeout);
        assert_eq!(checker.connect_timeout, timeout);
    }

    #[test]
    fn it_should_implement_default_trait() {
        let checker = PortChecker::default();
        assert_eq!(checker.connect_timeout, Duration::from_secs(3));
    }

    #[test]
    fn it_should_have_proper_error_display() {
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 22);
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "Host not found");
        let error = PortCheckerError::NetworkError {
            socket_addr,
            source: io_error,
        };

        let error_str = error.to_string();
        assert!(error_str.contains("Failed to test port connectivity"));
        assert!(error_str.contains("192.168.1.1:22"));
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn it_should_support_debug_formatting() {
        let checker = PortChecker::new();
        let debug_str = format!("{checker:?}");
        assert!(debug_str.contains("PortChecker"));
        assert!(debug_str.contains("connect_timeout"));
    }

    #[test]
    fn it_should_handle_invalid_socket_addresses() {
        // Test with a socket address that should cause connection refused
        let checker = PortChecker::with_timeout(Duration::from_millis(100));
        let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 1); // Port 1 should be closed

        // This should return Ok(false) for connection refused, not an error
        let result = checker.is_port_open(socket_addr);
        match result {
            Ok(false) => {
                // Expected - port is closed
            }
            Ok(true) => {
                // Unexpected but possible if something is actually listening on port 1
                println!("Warning: Port 1 on localhost is unexpectedly open");
            }
            Err(e) => {
                panic!("Should not return error for connection refused: {e}");
            }
        }
    }

    // Note: We don't include integration tests that actually connect to real ports
    // as they would be flaky and depend on external services. The actual connectivity
    // testing logic is documented through these unit tests and the implementation.
}
