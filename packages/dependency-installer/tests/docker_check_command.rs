//! Integration tests for the dependency-installer CLI using Docker containers.
//!
//! These tests verify that the CLI binary works correctly in a clean Ubuntu 24.04
//! environment. They use testcontainers to spin up isolated Docker containers.

use std::path::PathBuf;

mod containers;
use containers::ubuntu::UbuntuContainerBuilder;

/// Test that the check command correctly identifies missing dependencies
/// in a fresh Ubuntu 24.04 container
#[tokio::test]
async fn test_check_all_reports_missing_dependencies() {
    // Get the binary path (built by cargo before running tests)
    let binary_path = get_binary_path();

    // Start Ubuntu container with the binary
    let container = UbuntuContainerBuilder::new()
        .with_binary(&binary_path)
        .start()
        .await;

    // Run the check command
    let output = container.exec(&["dependency-installer", "check"]);

    // Verify it reports missing dependencies
    assert!(
        output.contains("cargo-machete: not installed"),
        "Expected cargo-machete to be reported as not installed, got: {}",
        output
    );
    assert!(
        output.contains("OpenTofu: not installed"),
        "Expected OpenTofu to be reported as not installed, got: {}",
        output
    );
    assert!(
        output.contains("Ansible: not installed"),
        "Expected Ansible to be reported as not installed, got: {}",
        output
    );
    assert!(
        output.contains("LXD: not installed"),
        "Expected LXD to be reported as not installed, got: {}",
        output
    );

    // Verify exit code is non-zero (failure)
    let exit_code = container.exec_with_exit_code(&["dependency-installer", "check"]);
    assert_eq!(
        exit_code, 1,
        "check command should exit with 1 when dependencies missing"
    );
}

/// Test that the check command works for specific tools
#[tokio::test]
async fn test_check_specific_tool() {
    let binary_path = get_binary_path();

    let container = UbuntuContainerBuilder::new()
        .with_binary(&binary_path)
        .start()
        .await;

    // Check a specific tool (OpenTofu)
    let output = container.exec(&["dependency-installer", "check", "--tool", "opentofu"]);

    // The output contains "OpenTofu: not installed" in the status line
    // We check for the plain text version since the ✗ symbol may not be present
    // in all terminal environments or when output is redirected
    assert!(
        output.contains("OpenTofu: not installed"),
        "Expected OpenTofu to be reported as not installed, got: {}",
        output
    );

    let exit_code =
        container.exec_with_exit_code(&["dependency-installer", "check", "--tool", "opentofu"]);
    assert_eq!(
        exit_code, 1,
        "check command should exit with 1 for missing specific tool"
    );
}

/// Test that the list command works correctly
#[tokio::test]
async fn test_list_command() {
    let binary_path = get_binary_path();

    let container = UbuntuContainerBuilder::new()
        .with_binary(&binary_path)
        .start()
        .await;

    let output = container.exec(&["dependency-installer", "list"]);

    // Verify all tools are listed
    assert!(
        output.contains("cargo-machete"),
        "Expected cargo-machete to be listed, got: {}",
        output
    );
    assert!(
        output.contains("OpenTofu"),
        "Expected OpenTofu to be listed, got: {}",
        output
    );
    assert!(
        output.contains("Ansible"),
        "Expected Ansible to be listed, got: {}",
        output
    );
    assert!(
        output.contains("LXD"),
        "Expected LXD to be listed, got: {}",
        output
    );

    // Verify status is shown
    assert!(
        output.contains("not installed"),
        "Expected 'not installed' status to be shown, got: {}",
        output
    );
}

/// Test verbose output flag
#[tokio::test]
async fn test_verbose_output() {
    let binary_path = get_binary_path();

    let container = UbuntuContainerBuilder::new()
        .with_binary(&binary_path)
        .start()
        .await;

    let output = container.exec(&["dependency-installer", "check", "--verbose"]);

    // Verify debug/info logs are present
    // The CLI uses tracing, so we should see timestamp-prefixed log messages
    assert!(
        output.contains("INFO") || output.contains("Checking"),
        "Expected verbose output to contain INFO logs or 'Checking' message, got: {}",
        output
    );
}

/// Get the path to the compiled binary
///
/// This function assumes the binary was built before running tests.
/// Run `cargo build --bin dependency-installer` before running these tests.
///
/// # Implementation Note
///
/// We use CARGO_MANIFEST_DIR and navigate up to the workspace root, then into
/// the target directory. This works because:
/// 1. CARGO_MANIFEST_DIR points to packages/dependency-installer
/// 2. The workspace root is two directories up
/// 3. The target directory is in the workspace root
///
/// Alternative approaches considered:
/// - CARGO_TARGET_DIR: Not always set
/// - OUT_DIR: Points to build script output, not target/debug
/// - Searching for target dir: Too expensive
fn get_binary_path() -> PathBuf {
    // Get the package manifest directory (packages/dependency-installer)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // Navigate to workspace root (two levels up from packages/dependency-installer)
    let workspace_root = manifest_dir
        .parent() // packages/
        .and_then(|p| p.parent()) // workspace root
        .expect("Failed to find workspace root");

    // Build path to the binary in target/debug
    let path = workspace_root
        .join("target")
        .join("debug")
        .join("dependency-installer");

    assert!(
        path.exists(),
        "Binary not found at {:?}. Run 'cargo build --bin dependency-installer' first",
        path
    );

    path
}
