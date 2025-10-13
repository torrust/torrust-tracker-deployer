//! SSH Configuration Tests
//!
//! Tests for SSH client configuration validation and storage.
//! These tests verify that SSH configuration values are properly
//! stored and accessible through the SSH client interface.

use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

use super::*;

/// Test SSH configuration storage and retrieval
///
/// This test verifies that SSH configuration values (host, port, username, key paths)
/// are correctly stored and retrievable from the SSH client. It focuses on the
/// configuration management aspect without actual network connectivity.
#[tokio::test]
async fn it_should_store_ssh_configuration_correctly() {
    // Arrange: Set up test parameters and SSH client
    let test_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 100));
    let test_port = 2222;
    let test_username = "testuser";
    let private_key_path = PathBuf::from("/path/to/private_key");
    let public_key_path = PathBuf::from("/path/to/public_key.pub");

    let ssh_credentials = SshCredentials::new(
        private_key_path.clone(),
        public_key_path.clone(),
        Username::new(test_username).unwrap(),
    );
    let ssh_config = SshConfig::new(ssh_credentials, SocketAddr::new(test_ip, test_port));
    let ssh_client = SshClient::new(ssh_config);

    // Act: Configuration is stored during client creation (no explicit action needed)

    // Assert: Verify all configuration values are stored correctly
    assert_eq!(ssh_client.ssh_config().host_ip(), test_ip);
    assert_eq!(ssh_client.ssh_config().ssh_port(), test_port);
    assert_eq!(ssh_client.ssh_config().ssh_username(), test_username);
    assert_eq!(
        ssh_client.ssh_config().ssh_priv_key_path(),
        &private_key_path
    );
    assert_eq!(ssh_client.ssh_config().ssh_pub_key_path(), &public_key_path);
}
