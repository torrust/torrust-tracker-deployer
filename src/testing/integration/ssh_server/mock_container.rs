//! Mock SSH server container for fast testing

use std::net::{IpAddr, Ipv4Addr};

use super::config::SshServerConfig;
use super::error::SshServerError;

/// Mock SSH server container for fast testing
///
/// This implementation doesn't start a real container but provides the same
/// interface as a real SSH server. Use this for tests that only need to verify
/// configuration, timeouts, or client behavior without actual SSH connectivity.
pub struct MockSshServerContainer {
    config: SshServerConfig,
    host_ip: IpAddr,
    ssh_port: u16,
}

impl MockSshServerContainer {
    /// Create a mock SSH server container with custom configuration
    ///
    /// This doesn't start any actual container, making it very fast for tests
    /// that don't need real SSH connectivity.
    ///
    /// # Arguments
    ///
    /// * `config` - Configuration for the mock container
    ///
    /// # Returns
    ///
    /// A mock container configured with the provided settings.
    ///
    /// # Errors
    ///
    /// This function is infallible but returns a Result to match the interface
    /// of `RealSshServerContainer::start_with_config()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::testing::integration::ssh_server::{
    ///     MockSshServerContainer, SshServerConfig
    /// };
    ///
    /// let config = SshServerConfig::builder()
    ///     .username("customuser")
    ///     .password("custompass")
    ///     .build();
    ///
    /// let container = MockSshServerContainer::start_with_config(config).unwrap();
    /// ```
    pub fn start_with_config(config: SshServerConfig) -> Result<Self, SshServerError> {
        let ssh_port = config.mock_port;
        Ok(Self {
            config,
            host_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            ssh_port,
        })
    }

    /// Create a mock SSH server container with default configuration
    ///
    /// This is a convenience method that uses default configuration values.
    ///
    /// # Returns
    ///
    /// A mock container configured with default test credentials.
    ///
    /// # Errors
    ///
    /// This function is infallible but returns a Result to match the interface
    /// of `RealSshServerContainer::start()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::testing::integration::ssh_server::MockSshServerContainer;
    ///
    /// let container = MockSshServerContainer::start().unwrap();
    /// ```
    pub fn start() -> Result<Self, SshServerError> {
        Self::start_with_config(SshServerConfig::default())
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
    pub fn username(&self) -> &str {
        &self.config.username
    }

    /// Get the test password configured in the container
    #[must_use]
    pub fn password(&self) -> &str {
        &self.config.password
    }
}

impl super::SshServerContainer for MockSshServerContainer {
    fn ssh_port(&self) -> u16 {
        self.ssh_port
    }

    fn host_ip(&self) -> IpAddr {
        self.host_ip
    }

    fn username(&self) -> &str {
        &self.config.username
    }

    fn password(&self) -> &str {
        &self.config.password
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_use_default_mock_port_with_default_config() {
        let container = MockSshServerContainer::start().expect("Failed to create mock container");

        assert_eq!(container.ssh_port(), 2222);
    }

    #[test]
    fn it_should_use_custom_mock_port_from_config() {
        let config = SshServerConfig::builder().mock_port(5555).build();

        let container = MockSshServerContainer::start_with_config(config)
            .expect("Failed to create mock container");

        assert_eq!(container.ssh_port(), 5555);
    }

    #[test]
    fn it_should_allow_testing_with_different_port_configurations() {
        let ports = vec![3000, 4000, 5000];

        for port in ports {
            let config = SshServerConfig::builder().mock_port(port).build();

            let container = MockSshServerContainer::start_with_config(config)
                .expect("Failed to create mock container");

            assert_eq!(container.ssh_port(), port);
        }
    }
}
