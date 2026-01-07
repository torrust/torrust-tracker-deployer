//! End-to-End Black Box Tests for List Command
//!
//! This test suite provides true black-box testing of the list command
//! by running the production application as an external process. These tests
//! verify that the list command correctly displays environments in the
//! working directory.
//!
//! ## Test Approach
//!
//! - **Black Box**: Runs production binary as external process
//! - **Isolation**: Uses temporary directories for complete test isolation
//! - **Coverage**: Tests list command with and without environments
//! - **Verification**: Validates environment names appear in output
//!
//! ## Test Scenarios
//!
//! 1. Empty workspace: List command shows no environments
//! 2. Single environment: List command shows created environment
//! 3. Multiple environments: List command shows all created environments

use super::super::support::{EnvironmentStateAssertions, ProcessRunner, TempWorkspace};
use anyhow::Result;
use torrust_dependency_installer::{verify_dependencies, Dependency};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::create_test_environment_config;

/// Verify that all required dependencies are installed for list command E2E tests.
///
/// **Current State**: No system dependencies required.
///
/// These black-box tests run the production binary as an external process and verify
/// the list command workflow. Currently, they only test the command interface and
/// output formatting, without requiring infrastructure tools.
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
fn it_should_report_no_data_directory_when_workspace_is_empty() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace (empty)
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Act: Run list command on empty workspace
    let list_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_list_command()
        .expect("Failed to run list command");

    // Assert: Command should fail because no data directory exists
    // This is expected behavior - the list command reports that
    // no environments have been created yet
    assert!(
        !list_result.success(),
        "List command should fail when no data directory exists"
    );

    // Assert: Error message should indicate data directory not found
    let stderr = list_result.stderr();
    assert!(
        stderr.contains("Data directory not found")
            || stderr.contains("No environments")
            || stderr.contains("data"),
        "Expected error about missing data directory, got: {stderr}"
    );
}

#[test]
fn it_should_list_created_environment() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-list-single");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    // Create environment
    let create_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_create_command("./environment.json")
        .expect("Failed to run create command");

    assert!(
        create_result.success(),
        "Create command failed: {}",
        create_result.stderr()
    );

    // Verify environment was created
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-list-single");

    // Act: Run list command
    let list_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_list_command()
        .expect("Failed to run list command");

    // Assert: Command should succeed
    assert!(
        list_result.success(),
        "List command failed with exit code: {:?}\nstderr: {}",
        list_result.exit_code(),
        list_result.stderr()
    );

    // Assert: Output should contain the environment name
    let stdout = list_result.stdout();
    assert!(
        stdout.contains("test-list-single"),
        "Expected environment name 'test-list-single' in output, got: {stdout}"
    );
}

#[test]
fn it_should_list_multiple_environments() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create first environment
    let config1 = create_test_environment_config("test-list-first");
    temp_workspace
        .write_config_file("env1.json", &config1)
        .expect("Failed to write first config file");

    let create_result1 = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_create_command("./env1.json")
        .expect("Failed to run first create command");

    assert!(
        create_result1.success(),
        "First create command failed: {}",
        create_result1.stderr()
    );

    // Create second environment
    let config2 = create_test_environment_config("test-list-second");
    temp_workspace
        .write_config_file("env2.json", &config2)
        .expect("Failed to write second config file");

    let create_result2 = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_create_command("./env2.json")
        .expect("Failed to run second create command");

    assert!(
        create_result2.success(),
        "Second create command failed: {}",
        create_result2.stderr()
    );

    // Verify both environments were created
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-list-first");
    env_assertions.assert_environment_exists("test-list-second");

    // Act: Run list command
    let list_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_list_command()
        .expect("Failed to run list command");

    // Assert: Command should succeed
    assert!(
        list_result.success(),
        "List command failed with exit code: {:?}\nstderr: {}",
        list_result.exit_code(),
        list_result.stderr()
    );

    // Assert: Output should contain both environment names
    let stdout = list_result.stdout();
    assert!(
        stdout.contains("test-list-first"),
        "Expected environment name 'test-list-first' in output, got: {stdout}"
    );
    assert!(
        stdout.contains("test-list-second"),
        "Expected environment name 'test-list-second' in output, got: {stdout}"
    );
}
