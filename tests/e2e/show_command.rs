//! End-to-End Black Box Tests for Show Command
//!
//! This test suite provides true black-box testing of the show command
//! by running the production application as an external process. These tests
//! verify that the show command correctly displays environment details.
//!
//! ## Test Approach
//!
//! - **Black Box**: Runs production binary as external process
//! - **Isolation**: Uses temporary directories for complete test isolation
//! - **Coverage**: Tests show command with existing and non-existing environments
//! - **Verification**: Validates environment details appear in output
//!
//! ## Test Scenarios
//!
//! 1. Non-existing environment: Show command reports environment not found
//! 2. Created environment: Show command displays environment details
//! 3. State information: Show command includes state-aware details

use super::super::support::{EnvironmentStateAssertions, ProcessRunner, TempWorkspace};
use anyhow::Result;
use torrust_dependency_installer::{verify_dependencies, Dependency};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::create_test_environment_config;

/// Verify that all required dependencies are installed for show command E2E tests.
///
/// **Current State**: No system dependencies required.
///
/// These black-box tests run the production binary as an external process and verify
/// the show command workflow. Currently, they only test the command interface and
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
fn it_should_report_environment_not_found_when_environment_does_not_exist() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace (empty)
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Act: Run show command for a non-existing environment
    let show_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_show_command("non-existing-env")
        .expect("Failed to run show command");

    // Assert: Command should fail because environment doesn't exist
    assert!(
        !show_result.success(),
        "Show command should fail when environment does not exist"
    );

    // Assert: Error message should indicate environment not found
    let stderr = show_result.stderr();
    assert!(
        stderr.contains("not found")
            || stderr.contains("Not found")
            || stderr.contains("does not exist")
            || stderr.contains("No environment")
            || stderr.contains("data"),
        "Expected error about missing environment, got: {stderr}"
    );
}

#[test]
fn it_should_show_created_environment_details() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-show-env");
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
    env_assertions.assert_environment_exists("test-show-env");

    // Act: Run show command
    let show_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_show_command("test-show-env")
        .expect("Failed to run show command");

    // Assert: Command should succeed
    assert!(
        show_result.success(),
        "Show command failed with exit code: {:?}\nstderr: {}",
        show_result.exit_code(),
        show_result.stderr()
    );

    // Assert: Output should contain the environment name
    let stdout = show_result.stdout();
    assert!(
        stdout.contains("test-show-env"),
        "Expected environment name 'test-show-env' in output, got: {stdout}"
    );
}

#[test]
fn it_should_show_environment_state() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-show-state");
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

    // Act: Run show command
    let show_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_show_command("test-show-state")
        .expect("Failed to run show command");

    // Assert: Command should succeed
    assert!(
        show_result.success(),
        "Show command failed with exit code: {:?}\nstderr: {}",
        show_result.exit_code(),
        show_result.stderr()
    );

    // Assert: Output should contain state information
    // A newly created environment should be in 'created' state
    let stdout = show_result.stdout();
    assert!(
        stdout.contains("created") || stdout.contains("Created") || stdout.contains("state"),
        "Expected state information in output, got: {stdout}"
    );
}

#[test]
fn it_should_show_provider_information() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-show-provider");
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

    // Act: Run show command
    let show_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_show_command("test-show-provider")
        .expect("Failed to run show command");

    // Assert: Command should succeed
    assert!(
        show_result.success(),
        "Show command failed with exit code: {:?}\nstderr: {}",
        show_result.exit_code(),
        show_result.stderr()
    );

    // Assert: Output should contain provider information
    // The test config uses 'lxd' provider
    let stdout = show_result.stdout();
    assert!(
        stdout.contains("lxd") || stdout.contains("LXD") || stdout.contains("provider"),
        "Expected provider information in output, got: {stdout}"
    );
}
