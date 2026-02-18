//! End-to-End Black Box Tests for Validate Command
//!
//! This test suite provides true black-box testing of the validate command
//! by running the production application as an external process. These tests
//! verify that the validate command correctly validates environment configuration
//! files without creating actual deployments.
//!
//! ## Test Approach
//!
//! - **Black Box**: Runs production binary as external process
//! - **Isolation**: Uses temporary directories for complete test isolation
//! - **Coverage**: Tests validate with valid/invalid configurations
//! - **Verification**: Validates error messages and success output
//!
//! ## Test Scenarios
//!
//! 1. **File Not Found**: Validate reports missing configuration file
//! 2. **Invalid JSON**: Validate reports malformed JSON with helpful error
//! 3. **Missing Required Fields**: Validate catches schema violations
//! 4. **Invalid Values**: Validate catches domain constraint violations
//! 5. **Valid Configuration**: Validate succeeds and shows environment details

use super::super::support::{ProcessRunner, TempWorkspace};
use anyhow::Result;
use std::fs;
use torrust_dependency_installer::{verify_dependencies, Dependency};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::create_test_environment_config;

/// Verify that all required dependencies are installed for validate command E2E tests.
///
/// **Current State**: No system dependencies required.
///
/// These black-box tests run the production binary as an external process and verify
/// the validate command workflow. The validate command is read-only and does not
/// require infrastructure tools (`OpenTofu`, `Ansible`, `LXD`).
///
/// # Errors
///
/// Returns an error if any required dependencies are missing or cannot be detected.
fn verify_required_dependencies() -> Result<()> {
    // Currently no system dependencies required - empty array
    let required_deps: &[Dependency] = &[];
    verify_dependencies(required_deps)?;
    Ok(())
}

#[test]
fn it_should_report_file_not_found_when_configuration_file_does_not_exist() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace (empty)
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    let nonexistent_file = temp_workspace.path().join("nonexistent.json");

    // Act: Run validate command for non-existent file
    let result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_validate_command(nonexistent_file.to_str().unwrap())
        .expect("Failed to run validate command");

    // Assert: Command should fail
    assert!(
        !result.success(),
        "Validate command should fail when file does not exist"
    );

    // Assert: Error message should indicate file not found
    let stderr = result.stderr();
    assert!(
        stderr.contains("not found") || stderr.contains("does not exist"),
        "Expected error about missing file, got: {stderr}"
    );

    // Assert: Help text should provide guidance
    assert!(
        stderr.contains("create template") || stderr.contains("file path"),
        "Expected helpful guidance in error message, got: {stderr}"
    );
}

#[test]
fn it_should_report_invalid_json_when_configuration_file_has_malformed_json() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create file with invalid JSON
    let invalid_json = "{ invalid json }";
    let config_path = temp_workspace.path().join("invalid.json");
    fs::write(&config_path, invalid_json).expect("Failed to write invalid JSON");

    // Act: Run validate command
    let result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_validate_command(config_path.to_str().unwrap())
        .expect("Failed to run validate command");

    // Assert: Command should fail
    assert!(
        !result.success(),
        "Validate command should fail for invalid JSON"
    );

    // Assert: Error message should mention JSON parsing
    let stderr = result.stderr();
    assert!(
        stderr.contains("JSON") || stderr.contains("parsing") || stderr.contains("syntax"),
        "Expected error about JSON parsing, got: {stderr}"
    );

    // Assert: Help text should provide guidance
    assert!(
        stderr.contains("Common issues")
            || stderr.contains("validator")
            || stderr.contains("syntax"),
        "Expected helpful troubleshooting tips, got: {stderr}"
    );
}

// Note: Test for missing SSH key files removed.
// File existence is no longer validated during config validation.
// SSH key files are external resources checked at runtime (provision/configure commands).
// This allows configs to be validated even when SSH keys don't exist yet or are on different machines.

#[test]
fn it_should_succeed_when_configuration_file_is_valid() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create valid configuration file
    let config = create_test_environment_config("test-validate-success");
    temp_workspace
        .write_config_file("valid.json", &config)
        .expect("Failed to write config file");

    let config_path = temp_workspace.path().join("valid.json");

    // Act: Run validate command
    let result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_validate_command(config_path.to_str().unwrap())
        .expect("Failed to run validate command");

    // Assert: Command should succeed
    assert!(
        result.success(),
        "Validate command should succeed for valid configuration. Stderr: {}",
        result.stderr()
    );

    // Assert: Output should indicate success
    let output = format!("{}{}", result.stdout(), result.stderr());
    assert!(
        output.contains("valid") || output.contains("✅"),
        "Expected success message, got stdout: '{}', stderr: '{}'",
        result.stdout(),
        result.stderr()
    );

    // Assert: Output should show environment details
    assert!(
        output.contains("test-validate-success"),
        "Expected environment name in output, got stdout: '{}', stderr: '{}'",
        result.stdout(),
        result.stderr()
    );

    // Assert: Output should show provider type
    assert!(
        output.contains("lxd") || output.contains("Provider"),
        "Expected provider information in output, got stdout: '{}', stderr: '{}'",
        result.stdout(),
        result.stderr()
    );
}

#[test]
fn it_should_validate_configuration_without_creating_deployment() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create valid configuration
    let config = create_test_environment_config("test-no-deployment");
    temp_workspace
        .write_config_file("config.json", &config)
        .expect("Failed to write config file");

    let config_path = temp_workspace.path().join("config.json");

    // Act: Run validate command
    let result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_validate_command(config_path.to_str().unwrap())
        .expect("Failed to run validate command");

    // Assert: Command succeeds
    assert!(result.success(), "Validate should succeed");

    // Assert: No environment directory created (validate is read-only)
    let data_dir = temp_workspace.path().join("data");
    assert!(
        !data_dir.exists() || fs::read_dir(&data_dir).unwrap().count() == 0,
        "Validate command should not create environment data directory"
    );
}
