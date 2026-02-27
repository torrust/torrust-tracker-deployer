//! End-to-End Black Box Tests for Purge Command
//!
//! This test suite provides true black-box testing of the purge command
//! by running the production application as an external process. These tests
//! verify that the purge command correctly removes local environment data
//! including data directories, build directories, and environment registry entries.
//!
//! ## Test Approach
//!
//! - **Black Box**: Runs production binary as external process
//! - **Isolation**: Uses temporary directories for complete test isolation
//! - **Coverage**: Tests complete workflow from environment creation to purge
//! - **Verification**: Validates all data is properly removed
//!
//! ## Test Scenarios
//!
//! 1. Purge destroyed environment: Create → Destroy → Purge (normal workflow)
//! 2. Purge with --force flag: Verify purge skips confirmation prompt
//! 3. Purge non-existent environment: Verify proper error handling
//! 4. Purge removes all artifacts: Verify data/, build/, and registry removal
//! 5. Full lifecycle with custom working directory: Create → Destroy → Purge
//!
//! ## Design Decisions
//!
//! - **Always uses --force**: Tests use `--force` flag to skip interactive prompts
//! - **After destroy**: Tests purge after destroy (normal workflow pattern)
//! - **Complete verification**: Checks data/, build/, and registry all removed

use super::super::support::{process_runner, EnvironmentStateAssertions, TempWorkspace};
use anyhow::Result;
use torrust_dependency_installer::{verify_dependencies, Dependency};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::create_test_environment_config;

/// Verify that all required dependencies are installed for purge command E2E tests.
///
/// **Current State**: No system dependencies required.
///
/// These black-box tests run the production binary as an external process and verify
/// the purge command workflow. Currently, they only test the command interface and
/// local file system cleanup, without requiring infrastructure tools.
///
/// # Future Dependencies
///
/// If these tests evolve to verify actual infrastructure cleanup or validation,
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
fn it_should_purge_destroyed_environment_successfully() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-purge-destroyed");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    // Create environment in default location
    let create_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command("./environment.json")
        .expect("Failed to run create command");

    assert!(
        create_result.success(),
        "Create command failed: {}",
        create_result.stderr()
    );

    // Destroy environment first (normal workflow)
    let destroy_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_destroy_command("test-purge-destroyed")
        .expect("Failed to run destroy command");

    assert!(
        destroy_result.success(),
        "Destroy command failed: {}",
        destroy_result.stderr()
    );

    // Verify environment exists before purge
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-purge-destroyed");
    env_assertions.assert_environment_state_is("test-purge-destroyed", "Destroyed");

    // Act: Purge environment using purge command with --force
    let purge_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_purge_command("test-purge-destroyed")
        .expect("Failed to run purge command");

    // Assert: Verify command succeeded
    assert!(
        purge_result.success(),
        "Purge command failed with exit code: {:?}\nstderr: {}",
        purge_result.exit_code(),
        purge_result.stderr()
    );

    // Assert: Verify all environment artifacts were removed
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_not_exists("test-purge-destroyed");
    env_assertions.assert_data_directory_not_exists("test-purge-destroyed");
    env_assertions.assert_build_directory_not_exists("test-purge-destroyed");

    // Assert: Verify success output (check both stdout and stderr)
    let output = format!("{}{}", purge_result.stdout(), purge_result.stderr());
    assert!(
        output.contains("\"purged\": true"),
        "Output should contain JSON success field. Combined output: {output}"
    );
}

#[test]
fn it_should_fail_when_purging_nonexistent_environment() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace (but don't create environment)
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Act: Try to purge non-existent environment
    let purge_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_purge_command("nonexistent-env")
        .expect("Failed to run purge command");

    // Assert: Verify command failed with proper error
    assert!(
        !purge_result.success(),
        "Purge command should fail for non-existent environment"
    );

    // Assert: Verify error message mentions environment not found
    let stderr = purge_result.stderr();
    assert!(
        stderr.contains("not found") || stderr.contains("Environment not found"),
        "Error message should mention environment not found. Stderr: {stderr}"
    );
}

#[test]
fn it_should_purge_with_custom_working_directory() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-purge-custom-dir");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    // Create environment in custom location
    let create_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command("./environment.json")
        .expect("Failed to run create command");

    assert!(
        create_result.success(),
        "Create command failed: {}",
        create_result.stderr()
    );

    // Destroy environment
    let destroy_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_destroy_command("test-purge-custom-dir")
        .expect("Failed to run destroy command");

    assert!(
        destroy_result.success(),
        "Destroy command failed: {}",
        destroy_result.stderr()
    );

    // Act: Purge environment from custom working directory
    let purge_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_purge_command("test-purge-custom-dir")
        .expect("Failed to run purge command");

    // Assert: Verify command succeeded
    assert!(
        purge_result.success(),
        "Purge command failed with exit code: {:?}\nstderr: {}",
        purge_result.exit_code(),
        purge_result.stderr()
    );

    // Assert: Verify environment was completely removed from custom location
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_not_exists("test-purge-custom-dir");
    env_assertions.assert_data_directory_not_exists("test-purge-custom-dir");
    env_assertions.assert_build_directory_not_exists("test-purge-custom-dir");
}

#[test]
fn it_should_complete_full_lifecycle_from_create_to_purge() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-full-lifecycle-purge");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    // Step 1: Create environment
    let create_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command("./environment.json")
        .expect("Failed to run create command");

    assert!(
        create_result.success(),
        "Create command failed: {}",
        create_result.stderr()
    );

    // Verify environment was created
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-full-lifecycle-purge");

    // Step 2: Destroy environment
    let destroy_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_destroy_command("test-full-lifecycle-purge")
        .expect("Failed to run destroy command");

    assert!(
        destroy_result.success(),
        "Destroy command failed: {}",
        destroy_result.stderr()
    );

    // Verify environment transitioned to Destroyed state
    env_assertions.assert_environment_state_is("test-full-lifecycle-purge", "Destroyed");

    // Step 3: Purge environment
    let purge_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_purge_command("test-full-lifecycle-purge")
        .expect("Failed to run purge command");

    assert!(
        purge_result.success(),
        "Purge command failed with exit code: {:?}\nstderr: {}",
        purge_result.exit_code(),
        purge_result.stderr()
    );

    // Step 4: Verify complete cleanup
    env_assertions.assert_environment_not_exists("test-full-lifecycle-purge");
    env_assertions.assert_data_directory_not_exists("test-full-lifecycle-purge");
    env_assertions.assert_build_directory_not_exists("test-full-lifecycle-purge");

    // Verify purge output indicates success (check both stdout and stderr)
    let output = format!("{}{}", purge_result.stdout(), purge_result.stderr());
    assert!(
        output.contains("\"purged\": true"),
        "Output should contain JSON success field. Combined output: {output}"
    );
}

#[test]
fn it_should_remove_only_specified_environment_data() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace with TWO environments
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create first environment
    let config1 = create_test_environment_config("test-purge-env1");
    temp_workspace
        .write_config_file("env1.json", &config1)
        .expect("Failed to write config file");

    let create1_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command("./env1.json")
        .expect("Failed to run create command");

    assert!(create1_result.success(), "Create env1 failed");

    // Create second environment
    let config2 = create_test_environment_config("test-purge-env2");
    temp_workspace
        .write_config_file("env2.json", &config2)
        .expect("Failed to write config file");

    let create2_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command("./env2.json")
        .expect("Failed to run create command");

    assert!(create2_result.success(), "Create env2 failed");

    // Destroy both environments
    let destroy1_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_destroy_command("test-purge-env1")
        .expect("Failed to run destroy command");

    assert!(destroy1_result.success(), "Destroy env1 failed");

    let destroy2_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_destroy_command("test-purge-env2")
        .expect("Failed to run destroy command");

    assert!(destroy2_result.success(), "Destroy env2 failed");

    // Verify both exist before purge
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-purge-env1");
    env_assertions.assert_environment_exists("test-purge-env2");

    // Act: Purge ONLY the first environment
    let purge_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_purge_command("test-purge-env1")
        .expect("Failed to run purge command");

    assert!(
        purge_result.success(),
        "Purge command failed: {}",
        purge_result.stderr()
    );

    // Assert: Verify only env1 was purged, env2 remains
    env_assertions.assert_environment_not_exists("test-purge-env1");
    env_assertions.assert_data_directory_not_exists("test-purge-env1");
    env_assertions.assert_build_directory_not_exists("test-purge-env1");

    // env2 should still exist
    env_assertions.assert_environment_exists("test-purge-env2");
    env_assertions.assert_environment_state_is("test-purge-env2", "Destroyed");
}

#[test]
fn it_should_produce_json_by_default() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create environment first so purge has something to remove
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");
    let config = create_test_environment_config("test-purge-json-default");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");
    let config_path = temp_workspace.path().join("environment.json");

    let create_result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_create_command(config_path.to_str().unwrap())
        .expect("Failed to run create command");

    assert!(
        create_result.success(),
        "Pre-condition: create must succeed, stderr: {}",
        create_result.stderr()
    );

    // Act: Run purge command without --output-format
    let result = process_runner()
        .working_dir(temp_workspace.path())
        .log_dir(temp_workspace.path().join("logs"))
        .run_purge_command("test-purge-json-default")
        .expect("Failed to run purge command");

    // Assert: Command succeeds
    assert!(
        result.success(),
        "Purge command should succeed for an existing environment, stderr: {}",
        result.stderr()
    );

    // Assert: stdout is valid JSON
    let stdout = result.stdout();
    let json: serde_json::Value =
        serde_json::from_str(&stdout).expect("Purge command default output must be valid JSON");

    // Assert: Expected field confirms environment was purged
    assert_eq!(
        json["purged"], true,
        "Expected `purged: true` in purge JSON output, got: {stdout}"
    );
}
