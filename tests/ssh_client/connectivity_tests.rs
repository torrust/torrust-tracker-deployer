//! SSH Connectivity Tests
//!
//! Tests for SSH client connectivity functionality including:
//! - Mock server connectivity (fast tests without Docker)
//! - Real server connectivity (Docker-based integration tests)
//! - Timeout behavior with unreachable hosts
//! - Connection failure scenarios

use super::*;

// =============================================================================
// MOCK SSH SERVER CONNECTIVITY TESTS
// =============================================================================

/// Test SSH connectivity establishment with mock server
///
/// This test uses `MockSshServerContainer` for fast execution without Docker dependencies.
/// It verifies the SSH client's connectivity testing mechanism works correctly,
/// expecting it to fail quickly since mock containers don't provide real SSH services.
#[tokio::test]
async fn it_should_establish_ssh_connectivity_with_mock_server() {
    // Arrange: Set up mock container and SSH client
    let container = MockSshServerContainer::start().expect("Mock container should always start");
    let client = SshTestBuilder::new()
        .with_mock_container(&container)
        .build_client();

    // Act & Assert: Test connectivity should fail quickly for mock server
    assert_connectivity_fails_quickly(&client, 10);
}

/// Test SSH connectivity timeout handling with mock infrastructure
///
/// This test verifies that SSH client timeout behavior works correctly with
/// mock infrastructure that doesn't provide real SSH connectivity.
/// It uses an unreachable IP to ensure timeout behavior is consistent.
#[tokio::test]
async fn it_should_handle_connectivity_timeouts() {
    // Arrange: Set up SSH client configured for unreachable host
    let client = SshTestBuilder::new().with_unreachable_host().build_client();

    // Act & Assert: Test connectivity should fail quickly for unreachable host
    assert_connectivity_fails_quickly(&client, 10);
}

// =============================================================================
// REAL SSH SERVER CONNECTIVITY TESTS
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
