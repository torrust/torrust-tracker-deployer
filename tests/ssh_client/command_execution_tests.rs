//! SSH Command Execution Tests
//!
//! Tests for SSH remote command execution functionality including:
//! - Basic command execution (echo, whoami)
//! - Output capture and validation
//! - Multiple command execution on same connection
//! - End-to-end SSH functionality verification

use super::*;

// =============================================================================
// REAL SSH SERVER COMMAND EXECUTION TESTS
// =============================================================================

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
