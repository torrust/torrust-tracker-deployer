//! Integration tests for the dependency-installer install command using Docker containers.
//!
//! These tests verify that the CLI binary can successfully install dependencies
//! in a clean Ubuntu 24.04 environment. They use testcontainers to spin up
//! isolated Docker containers.

use std::path::PathBuf;

mod containers;
use containers::ubuntu_container_builder::UbuntuContainerBuilder;

/// Test that cargo-machete can be installed
#[tokio::test]
async fn it_should_install_cargo_machete_successfully() {
    // Get the binary path (built by cargo before running tests)
    let binary_path = get_binary_path();

    // Start Ubuntu container with the binary and sudo
    let container = UbuntuContainerBuilder::new(&binary_path)
        .start_with_sudo()
        .await;

    // Install cargo first (required for cargo-machete)
    container.exec(&["apt-get", "update"]);
    container.exec(&["apt-get", "install", "-y", "cargo"]);

    // Install cargo-machete
    let output = container.exec(&[
        "dependency-installer",
        "install",
        "--dependency",
        "cargo-machete",
    ]);

    // Verify installation succeeded (logs should show success)
    let combined = output.to_string();
    assert!(
        combined.contains("installed") || combined.contains("Installing"),
        "Expected installation logs, got: {combined}"
    );

    // Verify the dependency is now installed
    let exit_code = container.exec_with_exit_code(&[
        "dependency-installer",
        "check",
        "--dependency",
        "cargo-machete",
        "--log-level",
        "off",
    ]);
    assert_eq!(
        exit_code, 0,
        "cargo-machete should be detected as installed after installation"
    );
}

/// Test that installation is idempotent (can run multiple times)
#[tokio::test]
async fn it_should_handle_idempotent_installation_of_cargo_machete() {
    let binary_path = get_binary_path();

    let container = UbuntuContainerBuilder::new(&binary_path)
        .start_with_sudo()
        .await;

    // Install cargo first
    container.exec(&["apt-get", "update"]);
    container.exec(&["apt-get", "install", "-y", "cargo"]);

    // Install cargo-machete first time
    let _output1 = container.exec(&[
        "dependency-installer",
        "install",
        "--dependency",
        "cargo-machete",
    ]);

    // Install cargo-machete second time (should not fail)
    let output2 = container.exec(&[
        "dependency-installer",
        "install",
        "--dependency",
        "cargo-machete",
    ]);

    // Second installation should complete without errors
    let combined = output2.to_string();
    assert!(
        !combined.contains("error") && !combined.contains("failed"),
        "Second installation should not fail, got: {combined}"
    );
}

/// Test that OpenTofu can be installed
#[tokio::test]
#[ignore] // This test is expensive, run with --ignored flag
async fn it_should_install_opentofu_successfully() {
    let binary_path = get_binary_path();

    let container = UbuntuContainerBuilder::new(&binary_path)
        .start_with_sudo()
        .await;

    // Install curl (required for OpenTofu installation)
    container.exec(&["apt-get", "update"]);
    container.exec(&["apt-get", "install", "-y", "curl", "gnupg"]);

    // Install OpenTofu
    let output = container.exec(&[
        "dependency-installer",
        "install",
        "--dependency",
        "opentofu",
    ]);

    // Verify installation succeeded
    let combined = output.to_string();
    assert!(
        combined.contains("installed") || combined.contains("Installing"),
        "Expected installation logs, got: {combined}"
    );

    // Verify the dependency is now installed
    let exit_code = container.exec_with_exit_code(&[
        "dependency-installer",
        "check",
        "--dependency",
        "opentofu",
        "--log-level",
        "off",
    ]);
    assert_eq!(
        exit_code, 0,
        "OpenTofu should be detected as installed after installation"
    );
}

/// Test that Ansible can be installed
#[tokio::test]
#[ignore] // This test is expensive, run with --ignored flag
async fn it_should_install_ansible_successfully() {
    let binary_path = get_binary_path();

    let container = UbuntuContainerBuilder::new(&binary_path)
        .start_with_sudo()
        .await;

    // Install Ansible
    let output = container.exec(&[
        "dependency-installer",
        "install",
        "--dependency",
        "ansible",
    ]);

    // Verify installation succeeded
    let combined = output.to_string();
    assert!(
        combined.contains("installed") || combined.contains("Installing"),
        "Expected installation logs, got: {combined}"
    );

    // Verify the dependency is now installed
    let exit_code = container.exec_with_exit_code(&[
        "dependency-installer",
        "check",
        "--dependency",
        "ansible",
        "--log-level",
        "off",
    ]);
    assert_eq!(
        exit_code, 0,
        "Ansible should be detected as installed after installation"
    );
}

/// Test that LXD can be installed
#[tokio::test]
#[ignore] // This test is expensive and requires snap, run with --ignored flag
async fn it_should_install_lxd_successfully() {
    let binary_path = get_binary_path();

    let container = UbuntuContainerBuilder::new(&binary_path)
        .start_with_sudo()
        .await;

    // Install snapd (required for LXD)
    container.exec(&["apt-get", "update"]);
    container.exec(&["apt-get", "install", "-y", "snapd"]);

    // Install LXD
    let output = container.exec(&[
        "dependency-installer",
        "install",
        "--dependency",
        "lxd",
    ]);

    // Verify installation succeeded
    let combined = output.to_string();
    assert!(
        combined.contains("installed") || combined.contains("Installing"),
        "Expected installation logs, got: {combined}"
    );

    // Verify the dependency is now installed
    let exit_code = container.exec_with_exit_code(&[
        "dependency-installer",
        "check",
        "--dependency",
        "lxd",
        "--log-level",
        "off",
    ]);
    assert_eq!(
        exit_code, 0,
        "LXD should be detected as installed after installation"
    );
}

/// Test that install command returns proper exit code on failure
#[tokio::test]
async fn it_should_return_error_exit_code_when_installation_fails() {
    let binary_path = get_binary_path();

    let container = UbuntuContainerBuilder::new(&binary_path)
        .start_with_sudo()
        .await;

    // Try to install cargo-machete without cargo (should fail)
    let exit_code = container.exec_with_exit_code(&[
        "dependency-installer",
        "install",
        "--dependency",
        "cargo-machete",
        "--log-level",
        "off",
    ]);

    assert_ne!(
        exit_code, 0,
        "install command should exit with non-zero code when installation fails"
    );
}

/// Get the path to the compiled binary
///
/// This function assumes the binary was built before running tests.
/// Run `cargo build --bin dependency-installer` before running these tests.
///
/// # Implementation Note
///
/// We use `CARGO_MANIFEST_DIR` and navigate up to the workspace root, then into
/// the target directory. This works because:
/// 1. `CARGO_MANIFEST_DIR` points to packages/dependency-installer
/// 2. The workspace root is two directories up
/// 3. The target directory is in the workspace root
///
/// Alternative approaches considered:
/// - `CARGO_TARGET_DIR`: Not always set
/// - `OUT_DIR`: Points to build script output, not target/debug
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
        "Binary not found at {}. Run 'cargo build --bin dependency-installer' first",
        path.display()
    );

    path
}
