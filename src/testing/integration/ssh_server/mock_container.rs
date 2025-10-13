//! Mock SSH server container for fast testing

use std::net::{IpAddr, Ipv4Addr};

/// Mock SSH server container for fast testing
///
/// This implementation doesn't start a real container but provides the same
/// interface as a real SSH server. Use this for tests that only need to verify
/// configuration, timeouts, or client behavior without actual SSH connectivity.
pub struct MockSshServerContainer {
    host_ip: IpAddr,
    ssh_port: u16,
    test_username: String,
    test_password: String,
}

impl MockSshServerContainer {
    /// Create a new mock SSH server container
    ///
    /// This doesn't start any actual container, making it very fast for tests
    /// that don't need real SSH connectivity.
    ///
    /// # Returns
    ///
    /// A mock container configured with default test credentials.
    ///
    /// # Errors
    ///
    /// This function is infallible but returns a Result to match the interface
    /// of `RealSshServerContainer::start()`.
    pub fn start() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            host_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            ssh_port: 2222, // Mock port
            test_username: "testuser".to_string(),
            test_password: "testpass".to_string(),
        })
    }

    /// Get the SSH port mapped by the container
    ///
    /// Returns the host port that maps to the container's SSH port (22).
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.ssh_port
    }

    /// Get the container's host IP address
    ///
    /// Returns the IP address to connect to the container from the host.
    #[must_use]
    pub fn host_ip(&self) -> IpAddr {
        self.host_ip
    }

    /// Get the test username configured in the container
    #[must_use]
    pub fn test_username(&self) -> &str {
        &self.test_username
    }

    /// Get the test password configured in the container
    #[must_use]
    pub fn test_password(&self) -> &str {
        &self.test_password
    }
}
