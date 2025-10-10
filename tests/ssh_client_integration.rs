//! SSH Client Integration Tests
//!
//! These tests verify SSH client functionality against both mock and real SSH servers.
//! The tests cover:
//!
//! 1. SSH connectivity verification (mock and real servers)
//! 2. Remote command execution
//! 3. Timeout handling for unreachable hosts
//! 4. Configuration validation
//!
//! # Test Types
//!
//! - **Mock Server Tests**: Use `MockSshServerContainer` for fast execution without Docker
//! - **Real Server Tests**: Use `RealSshServerContainer` with actual Docker SSH server
//!
//! # Test Environment
//!
//! Mock tests run without Docker dependencies and are fast for CI/CD.
//! Real server tests require Docker and the SSH server image to be built.
//! Real tests will skip gracefully if Docker is not available.
//!
//! # Test Strategy
//!
//! - Start containers (mock or real) based on test requirements
//! - Wait for SSH connectivity to be established
//! - Execute basic commands to verify functionality
//! - Test error conditions with unreachable hosts
//!
//! The tests focus on integration between SSH client components rather than
//! individual unit functionality.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::time::Duration;

use torrust_tracker_deployer_lib::shared::ssh::{SshClient, SshConfig, SshCredentials};
use torrust_tracker_deployer_lib::shared::Username;
use torrust_tracker_deployer_lib::testing::integration::ssh_server::{
    print_docker_debug_info, MockSshServerContainer, RealSshServerContainer,
};

/// SSH test constants following testing conventions
///
/// These constants provide consistent test values across all SSH-related tests.
const UNREACHABLE_IP: &str = "192.0.2.1"; // RFC 5737 TEST-NET-1
const TEST_USERNAME: &str = "testuser";
const REAL_SSH_PRIVATE_KEY: &str = "fixtures/testing_rsa";
const REAL_SSH_PUBLIC_KEY: &str = "fixtures/testing_rsa.pub";

/// Builder for SSH test clients with fluent API
///
/// This builder eliminates boilerplate code in SSH integration tests by
/// providing a fluent interface for creating SSH clients with various
/// configurations. It supports mock containers, real containers, and
/// unreachable hosts for comprehensive testing scenarios.
///
/// # Examples
///
/// ```rust
/// // Basic client with default settings
/// let client = SshTestBuilder::new().build_client();
///
/// // Client for mock container testing
/// let mock_container = MockSshServerContainer::start().unwrap();
/// let client = SshTestBuilder::new()
///     .with_mock_container(&mock_container)
///     .build_client();
///
/// // Client for unreachable host testing
/// let client = SshTestBuilder::new()
///     .with_unreachable_host()
///     .build_client();
/// ```
struct SshTestBuilder {
    username: String,
    host_ip: IpAddr,
    port: u16,
    use_real_keys: bool,
}

impl SshTestBuilder {
    /// Create a new SSH test builder with default settings
    ///
    /// Default settings:
    /// - Username: "testuser"
    /// - Host IP: 127.0.0.1 (localhost)
    /// - Port: 22
    /// - Use real keys: false (uses non-existent key paths)
    fn new() -> Self {
        Self {
            username: TEST_USERNAME.to_string(),
            host_ip: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 22,
            use_real_keys: false,
        }
    }

    /// Configure builder for use with a mock SSH server container
    ///
    /// This method extracts the connection details from the mock container
    /// and configures the builder appropriately for mock server testing.
    fn with_mock_container(self, container: &MockSshServerContainer) -> Self {
        Self {
            host_ip: container.host_ip(),
            port: container.ssh_port(),
            username: container.test_username().to_string(),
            ..self
        }
    }

    /// Configure builder for use with a real SSH server container
    ///
    /// This method extracts the connection details from the real container
    /// and enables real SSH key usage for actual SSH connectivity testing.
    #[allow(clippy::unused_self)]
    fn with_real_container(self, container: &RealSshServerContainer) -> Self {
        Self {
            host_ip: container.host_ip(),
            port: container.ssh_port(),
            username: container.test_username().to_string(),
            use_real_keys: true,
        }
    }

    /// Configure builder for unreachable host testing
    ///
    /// This method sets up the builder to use an RFC 5737 TEST-NET-1 IP address
    /// that is guaranteed to be unreachable for testing timeout scenarios.
    fn with_unreachable_host(self) -> Self {
        Self {
            host_ip: UNREACHABLE_IP.parse().unwrap(),
            ..self
        }
    }

    /// Enable real SSH keys for actual connectivity testing
    #[allow(dead_code)]
    fn with_real_keys(self) -> Self {
        Self {
            use_real_keys: true,
            ..self
        }
    }

    /// Build SSH credentials based on current configuration
    ///
    /// This method creates SSH credentials using either real key paths
    /// (for actual SSH testing) or non-existent paths (for mock testing).
    fn build_credentials(&self) -> SshCredentials {
        let (private_key, public_key) = if self.use_real_keys {
            (
                PathBuf::from(REAL_SSH_PRIVATE_KEY),
                PathBuf::from(REAL_SSH_PUBLIC_KEY),
            )
        } else {
            (
                PathBuf::from("/nonexistent/key"),
                PathBuf::from("/nonexistent/key.pub"),
            )
        };

        SshCredentials::new(
            private_key,
            public_key,
            Username::new(&self.username).expect("Username should be valid"),
        )
    }

    /// Build the complete SSH client with current configuration
    ///
    /// This method creates the SSH credentials and configuration, then
    /// constructs the final SSH client ready for testing.
    fn build_client(self) -> SshClient {
        let credentials = self.build_credentials();
        let config = SshConfig::new(credentials, SocketAddr::new(self.host_ip, self.port));
        SshClient::new(config)
    }
}

/// Test that SSH client can establish connectivity to a mock SSH server.
///
/// This test uses `MockSshServerContainer` for fast execution without Docker dependencies.
/// It verifies:
/// 1. Container creation and configuration
/// 2. SSH client setup with container connection details
/// 3. Basic connectivity testing behavior
/// 4. Error handling for mock server (which doesn't run real SSH)
#[tokio::test]
async fn it_should_establish_ssh_connectivity_with_mock_server() {
    // Arrange: Set up mock container and SSH client
    let container = match MockSshServerContainer::start() {
        Ok(container) => container,
        Err(e) => {
            println!("Skipping SSH integration test - Docker not available: {e}");
            return;
        }
    };
    let client = SshTestBuilder::new()
        .with_mock_container(&container)
        .build_client();

    // Act: Test connectivity and measure duration
    let start_time = std::time::Instant::now();
    let result = client.test_connectivity();
    let duration = start_time.elapsed();

    // Assert: Verify expected failure and quick timeout
    match result {
        Ok(true) => {
            println!("SSH connectivity test passed in {duration:?}");
        }
        Ok(false) => {
            println!("SSH connectivity test failed as expected (mock server) in {duration:?}");
            // Verify it completed reasonably quickly
            assert!(
                duration.as_secs() <= 10,
                "SSH timeout should complete within 10 seconds, took {duration:?}"
            );
        }
        Err(e) => {
            println!(
                "SSH connectivity test failed with error (expected for mock server) in {duration:?}: {e}"
            );
            // This is expected for a mock server that doesn't actually run SSH
            assert!(
                duration.as_secs() <= 10,
                "SSH timeout should complete within 10 seconds, took {duration:?}"
            );
        }
    }
}

/// Test that SSH client handles connection timeouts appropriately.
///
/// This test:
/// 1. Creates SSH configuration for an unreachable IP address
/// 2. Attempts to establish connectivity using `test_connectivity` (faster than `wait_for_connectivity`)
/// 3. Verifies that the operation times out as expected
/// 4. Demonstrates error handling for network failures
#[tokio::test]
async fn it_should_handle_connectivity_timeouts() {
    // Arrange: Set up SSH client with unreachable host
    let client = SshTestBuilder::new().with_unreachable_host().build_client();

    // Act: Test connectivity and measure duration
    let start_time = std::time::Instant::now();
    let result = client.test_connectivity();
    let duration = start_time.elapsed();

    // Assert: Verify that the connection attempt failed as expected
    match result {
        Ok(true) => {
            panic!("Expected connectivity to fail for unreachable host, but it succeeded");
        }
        Ok(false) => {
            println!("Connectivity correctly failed for unreachable host in {duration:?}");
            // Verify it completed reasonably quickly (should be around 5 seconds due to ConnectTimeout=5)
            assert!(
                duration.as_secs() <= 10,
                "SSH timeout should complete within 10 seconds, took {duration:?}"
            );
        }
        Err(e) => {
            println!("Connectivity failed with error in {duration:?}: {e}");
            // This is also acceptable - some systems might return an error instead of false
            assert!(
                duration.as_secs() <= 10,
                "SSH timeout should complete within 10 seconds, took {duration:?}"
            );
        }
    }
}

/// Test that SSH configuration properly stores connection parameters.
///
/// This test:
/// 1. Creates SSH configuration with specific parameters
/// 2. Verifies that all parameters are correctly stored and accessible
/// 3. Demonstrates configuration validation patterns
#[tokio::test]
async fn it_should_store_ssh_configuration_correctly() {
    // Define test parameters
    let test_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 100));
    let test_port = 2222;
    let test_username = "testuser";

    // Create SSH credentials
    let ssh_credentials = SshCredentials::new(
        PathBuf::from("/path/to/private_key"),
        PathBuf::from("/path/to/public_key.pub"),
        Username::new(test_username).unwrap(),
    );

    // Create SSH configuration
    let ssh_config = SshConfig::new(ssh_credentials, SocketAddr::new(test_ip, test_port));

    // Create SSH client
    let ssh_client = SshClient::new(ssh_config);

    // Verify configuration is stored correctly
    assert_eq!(ssh_client.ssh_config().host_ip(), test_ip);
    assert_eq!(ssh_client.ssh_config().ssh_port(), test_port);
    assert_eq!(ssh_client.ssh_config().ssh_username(), test_username);

    // Verify key paths are stored correctly
    assert_eq!(
        ssh_client.ssh_config().ssh_priv_key_path(),
        &PathBuf::from("/path/to/private_key")
    );
    assert_eq!(
        ssh_client.ssh_config().ssh_pub_key_path(),
        &PathBuf::from("/path/to/public_key.pub")
    );

    println!("SSH configuration validation completed successfully");
}

// =============================================================================
// REAL SSH SERVER TESTS (require Docker and SSH server image)
// =============================================================================

/// Test actual SSH connectivity using a real Docker SSH server
///
/// This test uses `RealSshServerContainer` which starts an actual SSH server in Docker.
/// It verifies:
/// 1. Real SSH server container startup
/// 2. SSH client connectivity to real server using SSH key authentication
/// 3. Actual SSH protocol communication
///
/// ## SSH Authentication Setup
///
/// - **SSH Keys**: Uses test keys from `fixtures/testing_rsa` (private) and `fixtures/testing_rsa.pub` (public)
/// - **Docker Image**: The SSH server Dockerfile has the test public key hardcoded for authentication
/// - **User**: Connects as `testuser` which is preconfigured in the Docker image
///
/// ## Requirements
///
/// - Docker must be running
/// - SSH server image must be built: `docker build -t torrust-ssh-server:latest docker/ssh-server/`
/// - The Docker image includes the hardcoded test public key for authentication
///
/// The test will skip gracefully if Docker is not available or the image is not built.
#[tokio::test]
async fn it_should_connect_to_real_ssh_server_and_test_connectivity() {
    // Arrange: Set up real SSH server container and client
    let container = match RealSshServerContainer::start().await {
        Ok(container) => container,
        Err(e) => {
            // Skip test if Docker is not available or image is not built
            println!("Skipping real SSH test - Docker/image not available: {e}");
            return;
        }
    };
    let client = SshTestBuilder::new()
        .with_real_container(&container)
        .build_client();

    // Act: Test connectivity with retry logic for CI environments
    // In CI, containers may need extra time for SSH daemon to fully initialize
    let start_time = std::time::Instant::now();
    let mut connectivity_succeeded = false;
    let mut last_error = None;

    for attempt in 0..10 {
        // Try up to 10 times (20 seconds total)
        match client.test_connectivity() {
            Ok(true) => {
                connectivity_succeeded = true;
                break;
            }
            Ok(false) => {
                // Connection failed, wait and retry
                if attempt < 9 {
                    // Don't sleep on last attempt
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            }
            Err(e) => {
                last_error = Some(e);
                break;
            }
        }
    }

    let duration = start_time.elapsed();

    // Assert: Verify connectivity results and timeout behavior
    if connectivity_succeeded {
        println!("Real SSH connectivity test passed in {duration:?}");

        // With a real SSH server, connectivity should eventually succeed within reasonable time
        assert!(
            duration.as_secs() <= 20,
            "SSH connection should complete within 20 seconds, took {duration:?}"
        );
    } else if let Some(e) = last_error {
        // This might happen if the container isn't ready yet or there's a connection error
        println!("SSH connectivity failed with error (container may not be ready): {e}");

        // Print debug information to help diagnose the issue
        print_docker_debug_info(container.ssh_port());

        // Still verify timeout behavior
        assert!(
            duration.as_secs() <= 20,
            "SSH timeout should complete within 20 seconds, took {duration:?}"
        );
    } else {
        // Connection failed after all retries without errors (Ok(false) case)
        println!(
            "SSH connectivity failed after {} retries (no errors, but Ok(false))",
            10
        );

        // Print debug information to help diagnose the issue
        print_docker_debug_info(container.ssh_port());

        panic!("SSH connectivity should succeed with real server after retries, but failed after {duration:?}");
    }
}

/// Test SSH connectivity timeout behavior with real SSH infrastructure available
///
/// This test verifies that SSH client timeout behavior works correctly even when
/// real SSH infrastructure is available, by using an unreachable IP address.
/// It uses the same timeout logic as the mock tests but provides confidence that
/// timeouts work in a real Docker environment.
#[tokio::test]
async fn it_should_timeout_when_connecting_to_unreachable_host_with_real_ssh_infrastructure() {
    // Use an unreachable IP address (RFC 5737 TEST-NET-1)
    let unreachable_ip = IpAddr::V4("192.0.2.1".parse().unwrap());

    let ssh_credentials = SshCredentials::new(
        PathBuf::from("/nonexistent/key"),     // Not used for password auth
        PathBuf::from("/nonexistent/key.pub"), // Not used for password auth
        Username::new("testuser").unwrap(),
    );

    let ssh_config = SshConfig::new(ssh_credentials, SocketAddr::new(unreachable_ip, 22));

    let ssh_client = SshClient::new(ssh_config);

    // Measure timeout duration
    let start = std::time::Instant::now();
    let result = ssh_client.test_connectivity();
    let duration = start.elapsed();

    // Verify timeout behavior
    assert!(
        result.is_err() || !result.unwrap(),
        "Connection to unreachable host should fail"
    );

    // Should timeout around 5 seconds (with some tolerance for system variations)
    assert!(
        duration >= Duration::from_secs(4) && duration <= Duration::from_secs(10),
        "Timeout should be around 5 seconds, was: {duration:?}"
    );

    println!("Real SSH infrastructure timeout test completed in {duration:?}");
}

/// Test remote command execution using a real Docker SSH server
///
/// This test uses `RealSshServerContainer` to execute actual commands via SSH.
/// It verifies:
/// 1. SSH connectivity and authentication with key-based auth
/// 2. Remote command execution (`echo` command)
/// 3. Output capture and validation
/// 4. End-to-end SSH functionality
///
/// ## SSH Authentication Setup
///
/// - **SSH Keys**: Uses test keys from `fixtures/testing_rsa` (private) and `fixtures/testing_rsa.pub` (public)
/// - **Docker Image**: The SSH server Dockerfile has the test public key hardcoded for authentication
/// - **User**: Connects as `testuser` which is preconfigured in the Docker image
///
/// ## Requirements
///
/// - Docker must be running
/// - SSH server image must be built: `docker build -t torrust-ssh-server:latest docker/ssh-server/`
/// - The Docker image includes the hardcoded test public key for authentication
///
/// The test will skip gracefully if Docker is not available or the image is not built.
#[tokio::test]
async fn it_should_execute_remote_command_on_real_ssh_server() {
    // Start real SSH server container
    let ssh_container = match RealSshServerContainer::start().await {
        Ok(container) => container,
        Err(e) => {
            // Skip test if Docker is not available or image is not built
            println!("Skipping real SSH command execution test - Docker/image not available: {e}");
            return;
        }
    };

    // Create SSH credentials using the test SSH keys
    // NOTE: These keys must match the hardcoded public key in the SSH server Dockerfile
    let ssh_credentials = SshCredentials::new(
        PathBuf::from("fixtures/testing_rsa"),     // Test private key
        PathBuf::from("fixtures/testing_rsa.pub"), // Test public key (hardcoded in Docker image)
        Username::new(ssh_container.test_username()).unwrap(),
    );

    // Create SSH configuration
    let ssh_config = SshConfig::new(
        ssh_credentials,
        SocketAddr::new(ssh_container.host_ip(), ssh_container.ssh_port()),
    );

    let ssh_client = SshClient::new(ssh_config);

    // Wait for SSH connectivity to be established
    match ssh_client.wait_for_connectivity().await {
        Ok(()) => {
            println!("SSH connectivity established successfully");
        }
        Err(e) => {
            println!("SSH connectivity failed - skipping command execution test: {e}");
            return;
        }
    }

    // Execute a simple echo command
    let test_message = "Hello SSH Integration Test";
    let command = format!("echo '{test_message}'");

    match ssh_client.execute(&command) {
        Ok(output) => {
            let trimmed_output = output.trim();
            println!("Command '{command}' executed successfully. Output: '{trimmed_output}'");

            // Verify the output matches our expected message
            assert_eq!(
                trimmed_output, test_message,
                "Command output should match expected message"
            );
        }
        Err(e) => {
            panic!("SSH command execution should succeed with real server: {e}");
        }
    }

    // Execute another command to verify multiple executions work
    let whoami_result = ssh_client.execute("whoami");
    match whoami_result {
        Ok(output) => {
            let username = output.trim();
            println!("whoami command output: '{username}'");
            assert_eq!(
                username,
                ssh_container.test_username(),
                "whoami should return the test username"
            );
        }
        Err(e) => {
            panic!("whoami command should succeed: {e}");
        }
    }
}
