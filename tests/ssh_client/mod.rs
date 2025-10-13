//! Common SSH test utilities and module exports
//!
//! This module provides shared utilities and re-exports for SSH integration tests.
//! It follows the project's module organization principles by centralizing common
//! imports, constants, and utilities.

pub mod command_execution_tests;
pub mod configuration_tests;
pub mod connectivity_tests;

// Re-export common SSH testing utilities
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use torrust_tracker_deployer_lib::shared::ssh::{SshClient, SshConfig, SshCredentials};
use torrust_tracker_deployer_lib::shared::Username;
use torrust_tracker_deployer_lib::testing::integration::ssh_server::{
    MockSshServerContainer, RealSshServerContainer,
};

/// SSH test constants following testing conventions
///
/// These constants provide consistent test values across all SSH-related tests.
pub const UNREACHABLE_IP: &str = "192.0.2.1"; // RFC 5737 TEST-NET-1
pub const TEST_USERNAME: &str = "testuser";
pub const REAL_SSH_PRIVATE_KEY: &str = "fixtures/testing_rsa";
pub const REAL_SSH_PUBLIC_KEY: &str = "fixtures/testing_rsa.pub";

/// Builder for SSH test clients with fluent API
///
/// Provides a convenient way to create SSH clients for testing with different configurations.
/// Follows the builder pattern with method chaining for readable test setup.
///
/// # Examples
///
/// ```rust,ignore
/// // Mock server test
/// let client = SshTestBuilder::new()
///     .with_mock_container(&container)
///     .build_client();
///
/// // Real server test
/// let client = SshTestBuilder::new()
///     .with_real_container(&container)
///     .build_client();
///
/// // Unreachable host test
/// let client = SshTestBuilder::new()
///     .with_unreachable_host()
///     .build_client();
/// ```
pub struct SshTestBuilder {
    host_ip: Option<IpAddr>,
    port: Option<u16>,
    username: Option<String>,
    private_key_path: Option<PathBuf>,
    public_key_path: Option<PathBuf>,
}

impl SshTestBuilder {
    /// Create a new SSH test builder
    pub fn new() -> Self {
        Self {
            host_ip: None,
            port: None,
            username: None,
            private_key_path: None,
            public_key_path: None,
        }
    }

    /// Configure for mock SSH server container
    pub fn with_mock_container(mut self, container: &MockSshServerContainer) -> Self {
        self.host_ip = Some(container.host_ip());
        self.port = Some(container.ssh_port());
        self.username = Some(container.test_username().to_string());
        self.private_key_path = Some(PathBuf::from(REAL_SSH_PRIVATE_KEY));
        self.public_key_path = Some(PathBuf::from(REAL_SSH_PUBLIC_KEY));
        self
    }

    /// Configure for real SSH server container
    pub fn with_real_container(mut self, container: &RealSshServerContainer) -> Self {
        self.host_ip = Some(container.host_ip());
        self.port = Some(container.ssh_port());
        self.username = Some(container.test_username().to_string());
        self.private_key_path = Some(PathBuf::from(REAL_SSH_PRIVATE_KEY));
        self.public_key_path = Some(PathBuf::from(REAL_SSH_PUBLIC_KEY));
        self
    }

    /// Configure for unreachable host (timeout testing)
    pub fn with_unreachable_host(mut self) -> Self {
        self.host_ip = Some(IpAddr::V4(UNREACHABLE_IP.parse().unwrap()));
        self.port = Some(22);
        self.username = Some(TEST_USERNAME.to_string());
        self.private_key_path = Some(PathBuf::from("/nonexistent/key"));
        self.public_key_path = Some(PathBuf::from("/nonexistent/key.pub"));
        self
    }

    /// Build the SSH client with configured parameters
    pub fn build_client(self) -> SshClient {
        let ssh_credentials = SshCredentials::new(
            self.private_key_path.unwrap(),
            self.public_key_path.unwrap(),
            Username::new(self.username.unwrap()).unwrap(),
        );

        let ssh_config = SshConfig::new(
            ssh_credentials,
            SocketAddr::new(self.host_ip.unwrap(), self.port.unwrap()),
        );

        SshClient::new(ssh_config)
    }
}

impl Default for SshTestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// SSH CONNECTIVITY HELPERS
// =============================================================================

/// Result of a connectivity test with retry logic
#[derive(Debug, Clone)]
pub struct ConnectivityTestResult {
    pub succeeded: bool,
    pub duration: Duration,
    pub error: Option<String>,
}

impl ConnectivityTestResult {
    /// Create a new connectivity test result
    pub fn new(succeeded: bool, duration: Duration, error: Option<String>) -> Self {
        Self {
            succeeded,
            duration,
            error,
        }
    }
}

/// Test connectivity with retry logic for CI environments
///
/// This helper function implements retry logic for connectivity testing,
/// which is useful in CI environments where network operations might
/// be unreliable or containers might need time to fully start.
pub async fn test_connectivity_with_retry(
    client: &SshClient,
    max_attempts: u32,
    retry_delay: Duration,
) -> ConnectivityTestResult {
    let start_time = Instant::now();
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

/// Assert connectivity fails quickly (for unreachable hosts and mock servers)
///
/// This helper verifies that SSH connectivity attempts fail within a reasonable
/// timeframe for unreachable hosts or mock servers that don't provide real SSH.
pub fn assert_connectivity_fails_quickly(client: &SshClient, max_seconds: u64) {
    let start_time = Instant::now();
    let result = client.test_connectivity();
    let duration = start_time.elapsed();

    assert!(
        result.is_err() || !result.unwrap(),
        "Expected connectivity to fail for unreachable/mock server"
    );

    // Mock servers can fail very quickly (milliseconds), unreachable hosts take longer (seconds)
    // Allow for a range from milliseconds to max_seconds to accommodate both scenarios
    assert!(
        duration <= Duration::from_secs(max_seconds),
        "Connectivity should fail within {max_seconds}s, but took: {duration:?}"
    );
}

/// Assert connectivity succeeds eventually (for real servers with potential startup delay)
///
/// This helper verifies that SSH connectivity eventually succeeds for real servers,
/// accounting for container startup time and SSH service initialization.
pub async fn assert_connectivity_succeeds_eventually(client: &SshClient, max_seconds: u64) {
    let retry_delay = Duration::from_millis(500);
    let max_attempts = u64::try_from(u128::from(max_seconds * 1000) / retry_delay.as_millis())
        .expect("max_attempts should fit in u64");
    let result = test_connectivity_with_retry(
        client,
        u32::try_from(max_attempts).expect("max_attempts should fit in u32"),
        retry_delay,
    )
    .await;

    assert!(
        result.succeeded,
        "Expected connectivity to succeed eventually. Duration: {:?}, Error: {:?}",
        result.duration, result.error
    );
}
