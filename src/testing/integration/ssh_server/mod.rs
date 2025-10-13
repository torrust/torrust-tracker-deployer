//! SSH Server Container for Integration Testing
//!
//! This module provides SSH server containers for testing SSH client functionality.
//! Two implementations are available:
//!
//! - `MockSshServerContainer`: Fast mock for tests that don't need real SSH connectivity
//! - `RealSshServerContainer`: Actual Docker SSH server for full integration tests
//!
//! Both implementations provide the same interface through the `SshServerContainer` trait,
//! allowing for polymorphic usage in tests.

use std::net::IpAddr;

mod constants;
mod debug;
mod error;
mod mock_container;
mod real_container;

pub use debug::print_docker_debug_info;
pub use error::SshServerError;
pub use mock_container::MockSshServerContainer;
pub use real_container::RealSshServerContainer;

/// Common interface for SSH server containers (mock and real)
///
/// This trait defines the standard interface that all SSH server container
/// implementations must provide. It enables polymorphic code that works with
/// both mock and real containers.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::testing::integration::ssh_server::{
///     SshServerContainer, MockSshServerContainer, RealSshServerContainer
/// };
///
/// async fn test_with_container<C: SshServerContainer>(container: &C) {
///     let port = container.ssh_port();
///     let ip = container.host_ip();
///     println!("SSH available at {}:{}", ip, port);
/// }
/// ```
pub trait SshServerContainer {
    /// Get the SSH port mapped by the container
    ///
    /// Returns the host port that maps to the container's SSH port (22).
    fn ssh_port(&self) -> u16;

    /// Get the container's host IP address
    ///
    /// Returns the IP address to connect to the container from the host.
    fn host_ip(&self) -> IpAddr;

    /// Get the test username configured in the container
    fn test_username(&self) -> &str;

    /// Get the test password configured in the container
    fn test_password(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn it_should_start_mock_ssh_server_container() {
        let container = MockSshServerContainer::start();

        match container {
            Ok(ssh_container) => {
                // Verify basic container properties
                let port = ssh_container.ssh_port();
                assert!(port > 0, "SSH port should be positive");

                let host_ip = ssh_container.host_ip();
                assert_eq!(host_ip, IpAddr::V4(Ipv4Addr::LOCALHOST));

                assert_eq!(ssh_container.test_username(), "testuser");
                assert_eq!(ssh_container.test_password(), "testpass");
            }
            Err(e) => {
                panic!("Mock container should always start successfully: {e}");
            }
        }
    }

    #[tokio::test]
    async fn it_should_start_real_ssh_server_container() {
        let container = RealSshServerContainer::start().await;

        match container {
            Ok(ssh_container) => {
                // Verify basic container properties
                let port = ssh_container.ssh_port();
                assert!(port > 0, "SSH port should be positive");

                let host_ip = ssh_container.host_ip();
                assert_eq!(host_ip, IpAddr::V4(Ipv4Addr::LOCALHOST));

                assert_eq!(ssh_container.test_username(), "testuser");
                assert_eq!(ssh_container.test_password(), "testpass");
            }
            Err(e) => {
                // Real container start might fail in CI environments without Docker
                // or if the SSH server image hasn't been built
                println!("Real container start failed (expected in some environments): {e}");
            }
        }
    }

    #[tokio::test]
    async fn it_should_work_with_trait_object() {
        // Test that we can use the trait for polymorphic behavior
        let mock = MockSshServerContainer::start()
            .expect("Mock container should always start successfully");

        // Use trait method through trait object
        let container: &dyn SshServerContainer = &mock;
        assert!(container.ssh_port() > 0);
        assert_eq!(container.host_ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_eq!(container.test_username(), "testuser");
        assert_eq!(container.test_password(), "testpass");
    }

    #[tokio::test]
    async fn it_should_enable_generic_code() {
        // Helper function that works with any SshServerContainer
        fn verify_container<C: SshServerContainer>(container: &C) {
            assert!(container.ssh_port() > 0);
            assert_eq!(container.test_username(), "testuser");
        }

        let mock = MockSshServerContainer::start()
            .expect("Mock container should always start successfully");

        // Can call generic function with concrete type
        verify_container(&mock);
    }
}
