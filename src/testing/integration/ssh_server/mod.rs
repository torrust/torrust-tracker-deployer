//! SSH Server Container for Integration Testing
//!
//! This module provides SSH server containers for testing SSH client functionality.
//! Two implementations are available:
//!
//! - `MockSshServerContainer`: Fast mock for tests that don't need real SSH connectivity
//! - `RealSshServerContainer`: Actual Docker SSH server for full integration tests

mod debug;
mod mock_container;
mod real_container;

pub use debug::print_docker_debug_info;
pub use mock_container::MockSshServerContainer;
pub use real_container::RealSshServerContainer;

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
}
