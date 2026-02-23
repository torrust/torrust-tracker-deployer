//! End-to-End Black Box Tests for Create Command
//!
//! This test suite provides true black-box testing of the create command
//! by running the production application as an external process. Unlike
//! other E2E tests that mock infrastructure components, these tests exercise
//! the complete application workflow from configuration file to persisted
//! environment state.
//!
//! ## Test Approach
//!
//! - **Black Box**: Runs production binary as external process
//! - **Isolation**: Uses temporary directories for complete test isolation
//! - **Coverage**: Tests complete workflow from config file to persistence
//! - **Verification**: Validates environment state in data directory
//!
//! ## Test Scenarios
//!
//! 1. Happy path: Create environment from valid config file
//! 2. Invalid config: Graceful failure with validation errors
//! 3. Missing config file: Appropriate error when file not found
//! 4. Duplicate detection: Error when environment already exists

use super::super::support::{process_runner, EnvironmentStateAssertions, TempWorkspace};
use anyhow::Result;
use torrust_dependency_installer::{verify_dependencies, Dependency};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::create_test_environment_config;

/// Verify that all required dependencies are installed for create command E2E tests.
///
/// **Current State**: No system dependencies required.
///
/// These black-box tests run the production binary as an external process and verify
/// the create command workflow. Currently, they only test the command interface and
/// environment persistence, without requiring infrastructure tools.
///
/// # Future Dependencies
///
/// If these tests evolve to verify actual infrastructure provisioning or configuration,
/// add required dependencies here:
/// ```ignore
/// let required_deps = &[Dependency::OpenTofu, Dependency::Ansible, Dependency::Lxd];
/// ```
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
fn it_should_create_environment_from_config_file_black_box() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-environment");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    // Act: Run production application as external process
    let result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command("./environment.json")
        .expect("Failed to run create command");

    // Assert: Verify command succeeded
    assert!(
        result.success(),
        "Create command failed with exit code: {:?}\nstderr: {}",
        result.exit_code(),
        result.stderr()
    );

    // Assert: Verify environment state was persisted
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-environment");
    env_assertions.assert_environment_state_is("test-environment", "Created");
    env_assertions.assert_data_directory_structure("test-environment");
    // Note: traces directory is created on-demand, not during environment creation
}

#[test]
fn it_should_fail_gracefully_with_invalid_config() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create invalid configuration (missing required fields)
    let invalid_config = r#"{"invalid": "config"}"#;
    temp_workspace
        .write_file("invalid.json", invalid_config)
        .expect("Failed to write invalid config");

    // Run command and expect failure
    let result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command("./invalid.json")
        .expect("Failed to run create command");

    // Assert command failed with helpful error message
    assert!(
        !result.success(),
        "Command should have failed with invalid config"
    );

    // Verify error message mentions configuration validation
    let stderr = result.stderr();
    assert!(
        stderr.contains("missing field") || stderr.contains("Configuration"),
        "Error message should mention configuration issues, got: {stderr}"
    );
}

#[test]
fn it_should_fail_when_config_file_not_found() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Run command with non-existent config file
    let result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command("./nonexistent.json")
        .expect("Failed to run create command");

    // Assert command failed with file not found error
    assert!(
        !result.success(),
        "Command should have failed with missing file"
    );

    // Verify error message mentions file not found
    let stderr = result.stderr();
    assert!(
        stderr.contains("not found") || stderr.contains("No such file"),
        "Error message should mention file not found, got: {stderr}"
    );
}

#[test]
fn it_should_fail_when_environment_already_exists() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    let config = create_test_environment_config("duplicate-env");
    temp_workspace
        .write_config_file("config.json", &config)
        .expect("Failed to write config");

    // Create environment first time
    let result1 = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command("./config.json")
        .expect("Failed to run create command");

    assert!(
        result1.success(),
        "First create should succeed, stderr: {}",
        result1.stderr()
    );

    // Try to create same environment again
    let result2 = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command("./config.json")
        .expect("Failed to run create command");

    // Assert second create failed
    assert!(
        !result2.success(),
        "Second create should fail with duplicate environment"
    );

    // Verify error message mentions duplicate or already exists
    let stderr = result2.stderr();
    assert!(
        stderr.contains("Already Exists")
            || stderr.contains("already exists")
            || stderr.contains("AlreadyExists"),
        "Error message should mention environment already exists, got: {stderr}"
    );
}
