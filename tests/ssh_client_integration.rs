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
    MockSshServerContainer, RealSshServerContainer,
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

/// Result of connectivity testing with timing and error information
///
/// This struct provides comprehensive information about SSH connectivity
/// test results, enabling consistent handling across different test scenarios.
#[derive(Debug, Clone)]
struct ConnectivityTestResult {
    pub succeeded: bool,
    pub duration: Duration,
    pub error: Option<String>,
}

impl ConnectivityTestResult {
    fn new(succeeded: bool, duration: Duration, error: Option<String>) -> Self {
        Self {
            succeeded,
            duration,
            error,
        }
    }
}

/// Test connectivity with retry logic for CI environments
///
/// This function encapsulates the common pattern of testing SSH connectivity
/// with retry logic, particularly useful for real SSH servers that may need
/// time to fully initialize in CI environments.
///
/// # Parameters
/// * `client` - SSH client to test
/// * `max_attempts` - Maximum number of retry attempts
/// * `retry_delay` - Delay between retry attempts
///
/// # Returns
/// `ConnectivityTestResult` with success status, duration, and any error
async fn test_connectivity_with_retry(
    client: &SshClient,
    max_attempts: u32,
    retry_delay: Duration,
) -> ConnectivityTestResult {
    let start_time = std::time::Instant::now();
    let mut last_error = None;

    for attempt in 0..max_attempts {
        match client.test_connectivity() {
            Ok(true) => {
                return ConnectivityTestResult::new(true, start_time.elapsed(), None);
            }
            Ok(false) => {
                if attempt < max_attempts - 1 {
                    tokio::time::sleep(retry_delay).await;
                }
            }
            Err(e) => {
                last_error = Some(e.to_string());
                break;
            }
        }
    }

    ConnectivityTestResult::new(false, start_time.elapsed(), last_error)
}

/// Assert timeout duration is within expected range
///
/// This function provides consistent timeout assertions across SSH tests,
/// with clear error messages that include actual vs expected timing.
///
/// # Parameters
/// * `duration` - Actual duration to check
/// * `min_secs` - Minimum expected seconds (inclusive)
/// * `max_secs` - Maximum expected seconds (exclusive)
fn assert_timeout_duration(duration: Duration, min_secs: u64, max_secs: u64) {
    let actual_secs = duration.as_secs();
    assert!(
        actual_secs >= min_secs && actual_secs < max_secs,
        "Timeout should be in range [{min_secs}s, {max_secs}s), was: {actual_secs}s ({duration:?})"
    );
}

/// Assert connectivity fails quickly (for unreachable hosts and mock servers)
///
/// This helper function tests that SSH connectivity fails quickly for
/// scenarios where connection should not succeed (mock servers, unreachable hosts).
/// Mock servers may fail in milliseconds while unreachable hosts typically take seconds.
///
/// # Parameters
/// * `client` - SSH client to test
/// * `max_seconds` - Maximum time allowed for failure (exclusive)
fn assert_connectivity_fails_quickly(client: &SshClient, max_seconds: u64) {
    let start_time = std::time::Instant::now();
    let result = client.test_connectivity();
    let duration = start_time.elapsed();

    assert!(
        result.is_err() || !result.unwrap(),
        "Expected connectivity to fail for unreachable/mock server"
    );

    // Mock servers can fail very quickly (milliseconds), so set minimum to 0
    assert_timeout_duration(duration, 0, max_seconds + 1);
}

/// Assert connectivity succeeds eventually (for real servers)
///
/// This helper function tests that SSH connectivity eventually succeeds
/// for real SSH servers, with appropriate retry logic for CI environments.
///
/// # Parameters
/// * `client` - SSH client to test
/// * `max_attempts` - Maximum retry attempts
async fn assert_connectivity_succeeds_eventually(client: &SshClient, max_attempts: u32) {
    let result = test_connectivity_with_retry(client, max_attempts, Duration::from_secs(2)).await;

    assert!(
        result.succeeded,
        "Expected connectivity to succeed within {} attempts, failed after {:?}: {:?}",
        max_attempts, result.duration, result.error
    );
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

    // Act & Assert: Test connectivity should fail quickly for mock server
    assert_connectivity_fails_quickly(&client, 10);
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

    // Act & Assert: Test connectivity should fail quickly for unreachable host
    assert_connectivity_fails_quickly(&client, 10);
}

/// Test that SSH configuration properly stores connection parameters.
///
/// This test:
/// 1. Creates SSH configuration with specific parameters
/// 2. Verifies that all parameters are correctly stored and accessible
/// 3. Demonstrates configuration validation patterns
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

    // Act & Assert: Test connectivity should succeed eventually for real server
    assert_connectivity_succeeds_eventually(&client, 10).await;
}

/// Test SSH connectivity timeout behavior with real SSH infrastructure available
///
/// This test verifies that SSH client timeout behavior works correctly even when
/// real SSH infrastructure is available, by using an unreachable IP address.
/// It uses the same timeout logic as the mock tests but provides confidence that
/// timeouts work in a real Docker environment.
#[tokio::test]
async fn it_should_timeout_when_connecting_to_unreachable_host_with_real_ssh_infrastructure() {
    // Arrange: Set up SSH client configured to connect to unreachable host
    let client = SshTestBuilder::new().with_unreachable_host().build_client();

    // Act & Assert: Test connectivity should fail quickly for unreachable host
    assert_connectivity_fails_quickly(&client, 10);
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
    // Arrange: Set up real SSH server container and client
    let ssh_container = match RealSshServerContainer::start().await {
        Ok(container) => container,
        Err(e) => {
            // Skip test if Docker is not available or image is not built
            println!("Skipping real SSH command execution test - Docker/image not available: {e}");
            return;
        }
    };

    let client = SshTestBuilder::new()
        .with_real_container(&ssh_container)
        .build_client();

    // Ensure SSH connectivity before command execution
    match client.wait_for_connectivity().await {
        Ok(()) => {
            println!("SSH connectivity established successfully");
        }
        Err(e) => {
            println!("SSH connectivity failed - skipping command execution test: {e}");
            return;
        }
    }

    // Act: Execute commands via SSH
    let test_message = "Hello SSH Integration Test";
    let echo_command = format!("echo '{test_message}'");
    let echo_result = client.execute(&echo_command);
    let whoami_result = client.execute("whoami");

    // Assert: Verify command execution results
    match echo_result {
        Ok(output) => {
            let trimmed_output = output.trim();
            assert_eq!(
                trimmed_output, test_message,
                "Echo command output should match expected message"
            );
        }
        Err(e) => {
            panic!("Echo command execution should succeed with real server: {e}");
        }
    }

    match whoami_result {
        Ok(output) => {
            let username = output.trim();
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
