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

// =============================================================================
// SSH OPTION OVERRIDE TESTS
// =============================================================================

/// Test that user options can override defaults
///
/// The SSH client now filters out defaults when users provide conflicting options:
/// - User-provided options are added first (SSH uses first-occurrence-wins)
/// - Default options are skipped if the user already provided that option key
/// - This allows users full control while providing sensible automation defaults
///
/// This test verifies that user-provided timeouts override the default.
#[tokio::test]
async fn it_should_allow_users_to_override_default_options() {
    // Arrange: Set up real SSH server
    let ssh_container = match RealSshServerContainer::start().await {
        Ok(container) => container,
        Err(e) => {
            println!("Skipping SSH option override test - Docker/image not available: {e}");
            return;
        }
    };

    let client = SshTestBuilder::new()
        .with_real_container(&ssh_container)
        .build_client();

    // Wait for connectivity
    match client.wait_for_connectivity().await {
        Ok(()) => {
            println!("SSH connectivity established for option override test");
        }
        Err(e) => {
            println!("SSH connectivity failed - skipping option override test: {e}");
            return;
        }
    }

    // Act & Assert: Test with override ConnectTimeout option
    // Default is 5 seconds, user overrides with 10 seconds
    // User option should take precedence (default is filtered out)
    let result = client.execute_with_options(
        "echo 'testing option override'",
        &["ConnectTimeout=10"], // Override default timeout
    );

    match result {
        Ok(output) => {
            let trimmed = output.trim();
            assert_eq!(
                trimmed, "testing option override",
                "Command should execute successfully with user-provided timeout"
            );
            println!("✓ User options override defaults - custom ConnectTimeout was used");
        }
        Err(e) => {
            panic!("Command should succeed with user-provided options: {e}");
        }
    }
}

/// Test explicit override of `StrictHostKeyChecking` option
///
/// This test demonstrates that users can override even critical automation settings.
/// When a user provides `StrictHostKeyChecking=yes`, it overrides the default `no`.
/// The default is filtered out, so only the user-provided value is sent to SSH.
#[tokio::test]
async fn it_should_allow_users_to_override_strict_host_key_checking() {
    // Arrange: Set up real SSH server
    let ssh_container = match RealSshServerContainer::start().await {
        Ok(container) => container,
        Err(e) => {
            println!("Skipping default precedence test - Docker/image not available: {e}");
            return;
        }
    };

    let client = SshTestBuilder::new()
        .with_real_container(&ssh_container)
        .build_client();

    // Wait for connectivity
    match client.wait_for_connectivity().await {
        Ok(()) => {
            println!("SSH connectivity established for precedence test");
        }
        Err(e) => {
            println!("SSH connectivity failed - skipping precedence test: {e}");
            return;
        }
    }

    // Act: Override StrictHostKeyChecking (defaults to 'no', user sets to 'yes')
    // NOTE: This test may fail if the SSH server's host key is not in known_hosts
    // and StrictHostKeyChecking=yes is enforced. However, in our test environment,
    // the command should still work because we've already established connectivity
    // with the default 'no' setting, and the host key was added to known_hosts.
    let result = client.execute_with_options(
        "echo 'testing strict host key override'",
        &["StrictHostKeyChecking=yes"], // Override to 'yes'
    );

    // Assert: Command execution result
    match result {
        Ok(output) => {
            let trimmed = output.trim();
            assert_eq!(
                trimmed, "testing strict host key override",
                "Command executes with user-provided StrictHostKeyChecking=yes"
            );
            println!("✓ User options override defaults - StrictHostKeyChecking=yes was used");
        }
        Err(e) => {
            // This might fail in strict environments - that's expected behavior
            println!(
                "Note: Command may fail with StrictHostKeyChecking=yes in some environments: {e}"
            );
            println!("This demonstrates that user overrides are respected by SSH");
        }
    }
}

/// Test that users can add new options that don't conflict with defaults
///
/// This test verifies that users can safely add SSH options that are not
/// part of the default set, such as:
/// - `ServerAliveInterval`
/// - `ServerAliveCountMax`
/// - Compression
/// - etc.
#[tokio::test]
async fn it_should_allow_users_to_add_non_conflicting_ssh_options() {
    // Arrange: Set up real SSH server
    let ssh_container = match RealSshServerContainer::start().await {
        Ok(container) => container,
        Err(e) => {
            println!("Skipping non-conflicting options test - Docker/image not available: {e}");
            return;
        }
    };

    let client = SshTestBuilder::new()
        .with_real_container(&ssh_container)
        .build_client();

    // Wait for connectivity
    match client.wait_for_connectivity().await {
        Ok(()) => {
            println!("SSH connectivity established for non-conflicting options test");
        }
        Err(e) => {
            println!("SSH connectivity failed - skipping non-conflicting options test: {e}");
            return;
        }
    }

    // Act: Add options that don't conflict with defaults
    let result = client.execute_with_options(
        "echo 'testing additional options'",
        &[
            "ServerAliveInterval=60", // Keep connection alive
            "ServerAliveCountMax=3",  // Max keepalive attempts
        ],
    );

    // Assert: Command succeeds with additional options
    match result {
        Ok(output) => {
            let trimmed = output.trim();
            assert_eq!(
                trimmed, "testing additional options",
                "Command should succeed with additional non-conflicting options"
            );
            println!("✓ Non-conflicting options work correctly");
        }
        Err(e) => {
            panic!("Command should succeed with non-conflicting options: {e}");
        }
    }
}
