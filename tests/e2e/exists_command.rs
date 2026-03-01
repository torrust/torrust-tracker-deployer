//! End-to-End Black Box Tests for Exists Command
//!
//! This test suite provides true black-box testing of the exists command
//! by running the production application as an external process. These tests
//! verify that the exists command correctly reports environment existence.
//!
//! ## Test Approach
//!
//! - **Black Box**: Runs production binary as external process
//! - **Isolation**: Uses temporary directories for complete test isolation
//! - **Coverage**: Tests all four scenarios from the exit code contract
//! - **Verification**: Validates stdout output and exit codes
//!
//! ## Test Scenarios
//!
//! 1. Non-existing environment: Command outputs `false` and exits 0
//! 2. Existing environment: Command outputs `true` and exits 0
//! 3. Invalid environment name: Command exits 1 with a helpful error
//! 4. JSON output: `--output-format json` produces valid JSON `true`/`false`

use super::super::support::{process_runner, EnvironmentStateAssertions, TempWorkspace};
use anyhow::Result;
use torrust_dependency_installer::{verify_dependencies, Dependency};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::create_test_environment_config;

/// Verify that all required dependencies are installed for exists command E2E tests.
///
/// **Current State**: No system dependencies required.
///
/// # Errors
///
/// Returns an error if any required dependencies are missing or cannot be detected.
fn verify_required_dependencies() -> Result<()> {
    let required_deps: &[Dependency] = &[];
    verify_dependencies(required_deps)?;
    Ok(())
}

#[test]
fn it_should_output_false_and_exit_0_when_environment_does_not_exist() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace (empty — no environments)
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Act: Run exists command for a non-existing environment
    let result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_exists_command("non-existing-env")
        .expect("Failed to run exists command");

    // Assert: Command should succeed (exit 0) even when environment doesn't exist
    assert!(
        result.success(),
        "Exists command should exit 0 when environment does not exist, stderr: {}",
        result.stderr()
    );

    // Assert: stdout should contain "false"
    let stdout = result.stdout();
    assert!(
        stdout.trim() == "false",
        "Expected stdout to be 'false', got: {stdout:?}"
    );
}

#[test]
fn it_should_output_true_and_exit_0_when_environment_exists() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace and create an environment
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    let config = create_test_environment_config("test-exists-env");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    let create_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command("./environment.json")
        .expect("Failed to run create command");

    assert!(
        create_result.success(),
        "Pre-condition: create must succeed, stderr: {}",
        create_result.stderr()
    );

    // Verify environment was created
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-exists-env");

    // Act: Run exists command
    let result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_exists_command("test-exists-env")
        .expect("Failed to run exists command");

    // Assert: Command should succeed with exit 0
    assert!(
        result.success(),
        "Exists command should exit 0 when environment exists, stderr: {}",
        result.stderr()
    );

    // Assert: stdout should contain "true"
    let stdout = result.stdout();
    assert!(
        stdout.trim() == "true",
        "Expected stdout to be 'true', got: {stdout:?}"
    );
}

#[test]
fn it_should_exit_1_and_report_error_for_invalid_environment_name() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Act: Run exists command with an invalid name (uppercase is not allowed)
    let result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_exists_command("INVALID_NAME")
        .expect("Failed to run exists command");

    // Assert: Command should fail (exit 1) for invalid name
    assert!(
        !result.success(),
        "Exists command should exit 1 for invalid environment name"
    );

    // Assert: stderr should contain an error message
    let stderr = result.stderr();
    assert!(
        !stderr.is_empty(),
        "Expected error message in stderr for invalid name, got: {stderr:?}"
    );
}

#[test]
fn it_should_produce_valid_json_false_for_non_existing_environment_with_json_format() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Empty workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Act: Run exists command with JSON output format
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_torrust-tracker-deployer"));
    cmd.args([
        "exists",
        "non-existing-env",
        "--output-format",
        "json",
        "--working-dir",
        temp_workspace.path().to_str().unwrap(),
        "--log-dir",
        temp_workspace.path().join("logs").to_str().unwrap(),
    ]);
    let output = cmd
        .output()
        .expect("Failed to run exists command with JSON format");

    // Assert: Command should succeed
    assert!(
        output.status.success(),
        "Exists command should exit 0 even for non-existing environment with JSON format"
    );

    // Assert: stdout is a valid JSON boolean `false`
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("Exists command JSON output must be valid JSON");
    assert_eq!(
        json,
        serde_json::Value::Bool(false),
        "Expected JSON false for non-existing environment, got: {json}"
    );
}
