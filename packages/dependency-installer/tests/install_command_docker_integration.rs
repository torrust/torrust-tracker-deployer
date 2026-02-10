//! Integration tests for the dependency-installer install command using Docker containers.
//!
//! These tests verify that the CLI binary can successfully install dependencies
//! in a clean Ubuntu 24.04 environment. They use testcontainers to spin up
//! isolated Docker containers.

use std::path::PathBuf;

mod containers;
use containers::ubuntu_container_builder::UbuntuContainerBuilder;

// ============================================================================
// NOTE: LXD Installer Test Not Included
// ============================================================================
//
// LXD installation via snap requires systemd to be running in the container,
// which is not available in standard Docker containers. To properly test LXD
// installation in a containerized environment, one would need:
//
// 1. A Docker image configured with systemd support
// 2. Running the container in privileged mode with systemd as init process
// 3. Additional container security capabilities
//
// This level of complexity goes beyond the scope of these integration tests.
// The LXD installer has been manually verified to work on real Ubuntu systems.
//
// For future maintainers: If you need to add LXD testing, consider:
// - Using a VM-based testing approach (e.g., with vagrant or multipass)
// - Creating a specialized Docker image with systemd support
// - Adding the test to a separate test suite that runs in VMs

/// Test that `cargo-machete` can be installed
#[tokio::test]
async fn it_should_install_cargo_machete_successfully() {
    // Get the binary path (built by cargo before running tests)
    let binary_path = get_binary_path();

    // Start Ubuntu container with the binary
    // Note: Container uses pre-built image with Rust nightly, sudo, and build tools
    let container = UbuntuContainerBuilder::new(&binary_path).start().await;

    // Install cargo-machete
    let exit_code = container.exec_with_exit_code(&[
        "dependency-installer",
        "install",
        "--dependency",
        "cargo-machete",
        "--log-level",
        "off",
    ]);

    // Verify installation succeeded via exit code
    assert_eq!(
        exit_code, 0,
        "cargo-machete installation should succeed (exit code 0)"
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

    let container = UbuntuContainerBuilder::new(&binary_path).start().await;

    // Note: Container uses pre-built image with Rust nightly, sudo, and build tools

    // Install cargo-machete first time
    let exit_code1 = container.exec_with_exit_code(&[
        "dependency-installer",
        "install",
        "--dependency",
        "cargo-machete",
        "--log-level",
        "off",
    ]);

    assert_eq!(
        exit_code1, 0,
        "First cargo-machete installation should succeed (exit code 0)"
    );

    // Install cargo-machete second time (should not fail - idempotent)
    let exit_code2 = container.exec_with_exit_code(&[
        "dependency-installer",
        "install",
        "--dependency",
        "cargo-machete",
        "--log-level",
        "off",
    ]);

    // Second installation should also succeed (idempotent behavior)
    assert_eq!(
        exit_code2, 0,
        "Second cargo-machete installation should succeed (exit code 0) - idempotent behavior"
    );
}

/// Test that `OpenTofu` can be installed
#[tokio::test]
async fn it_should_install_opentofu_successfully() {
    let binary_path = get_binary_path();

    let container = UbuntuContainerBuilder::new(&binary_path).start().await;

    // Install OpenTofu
    let exit_code = container.exec_with_exit_code(&[
        "dependency-installer",
        "install",
        "--dependency",
        "opentofu",
        "--log-level",
        "off",
    ]);

    // Verify installation succeeded via exit code
    assert_eq!(
        exit_code, 0,
        "OpenTofu installation should succeed (exit code 0)"
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

/// Test that `Ansible` can be installed
///
/// **Known Issue**: This test is flaky due to Ansible installation reliability in containers.
/// It's marked as `#[ignore]` to prevent CI failures. Run manually with:
/// `cargo test --package torrust-dependency-installer --test install_command_docker_integration it_should_install_ansible_successfully -- --ignored`
#[tokio::test]
#[ignore = "Flaky test: Ansible installation is unreliable in containers"]
async fn it_should_install_ansible_successfully() {
    let binary_path = get_binary_path();

    let container = UbuntuContainerBuilder::new(&binary_path).start().await;

    // Install Ansible
    let exit_code = container.exec_with_exit_code(&[
        "dependency-installer",
        "install",
        "--dependency",
        "ansible",
        "--log-level",
        "off",
    ]);

    // Verify installation succeeded via exit code
    assert_eq!(
        exit_code, 0,
        "Ansible installation should succeed (exit code 0)"
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

/// Test that install command returns proper exit code on failure
///
/// Note: This test uses `exec_with_exit_code_silent` to suppress the expected
/// error output, keeping test output clean while still validating the exit code.
#[tokio::test]
async fn it_should_return_error_exit_code_when_installation_fails() {
    let binary_path = get_binary_path();

    let container = UbuntuContainerBuilder::new(&binary_path).start().await;

    // Try to install an invalid/unknown dependency (should fail)
    // Use silent execution to avoid error noise in test output
    let exit_code = container.exec_with_exit_code_silent(&[
        "dependency-installer",
        "install",
        "--dependency",
        "invalid-dependency-name",
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
