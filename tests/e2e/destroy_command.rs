//! End-to-End Black Box Tests for Destroy Command
//!
//! This test suite provides true black-box testing of the destroy command
//! by running the production application as an external process. These tests
//! verify that the destroy command correctly handles the working directory
//! parameter, ensuring environments can be destroyed from custom locations.
//!
//! ## Test Approach
//!
//! - **Black Box**: Runs production binary as external process
//! - **Isolation**: Uses temporary directories for complete test isolation
//! - **Coverage**: Tests complete workflow from environment creation to destruction
//! - **Verification**: Validates environment is properly removed
//!
//! ## Test Scenarios
//!
//! 1. Default working directory: Destroy environment from current directory
//! 2. Custom working directory: Destroy environment from temporary directory
//! 3. Full lifecycle: Create → Destroy with custom working directory

use super::super::support::{EnvironmentStateAssertions, ProcessRunner, TempWorkspace};
use anyhow::Result;
use torrust_dependency_installer::{verify_dependencies, Dependency};

/// Verify that all required dependencies are installed for destroy command E2E tests.
///
/// **Current State**: No system dependencies required.
///
/// These black-box tests run the production binary as an external process and verify
/// the destroy command workflow. Currently, they only test the command interface and
/// environment state transitions, without requiring infrastructure tools.
///
/// # Future Dependencies
///
/// If these tests evolve to verify actual infrastructure destruction or cleanup,
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

/// Helper function to create a test environment configuration
fn create_test_environment_config(env_name: &str) -> String {
    // Use absolute paths to SSH keys to ensure they work regardless of current directory
    let project_root = env!("CARGO_MANIFEST_DIR");
    let private_key_path = format!("{project_root}/fixtures/testing_rsa");
    let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

    serde_json::json!({
        "environment": {
            "name": env_name
        },
        "ssh_credentials": {
            "private_key_path": private_key_path,
            "public_key_path": public_key_path,
            "username": "torrust",
            "port": 22
        },
        "provider": {
            "provider": "lxd",
            "profile_name": format!("lxd-{}", env_name)
        },
        "tracker": {
            "core": {
                "database": {
                    "driver": "sqlite3",
                    "database_name": "tracker.db"
                },
                "private": false
            },
            "udp_trackers": [
                {
                    "bind_address": "0.0.0.0:6969"
                }
            ],
            "http_trackers": [
                {
                    "bind_address": "0.0.0.0:7070"
                }
            ],
            "http_api": {
                "admin_token": "MyAccessToken"
            }
        }
    })
    .to_string()
}

#[test]
fn it_should_destroy_environment_with_default_working_directory() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-destroy-default");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    // Create environment in default location
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
    env_assertions.assert_environment_exists("test-destroy-default");

    // Act: Destroy environment using destroy command
    let destroy_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_destroy_command("test-destroy-default")
        .expect("Failed to run destroy command");

    // Assert: Verify command succeeded
    assert!(
        destroy_result.success(),
        "Destroy command failed with exit code: {:?}\nstderr: {}",
        destroy_result.exit_code(),
        destroy_result.stderr()
    );

    // Assert: Verify environment was transitioned to Destroyed state
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-destroy-default");
    env_assertions.assert_environment_state_is("test-destroy-default", "Destroyed");
}

#[test]
fn it_should_destroy_environment_with_custom_working_directory() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-destroy-custom");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    // Create environment in custom location
    let create_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_create_command("./environment.json")
        .expect("Failed to run create command");

    assert!(
        create_result.success(),
        "Create command failed: {}",
        create_result.stderr()
    );

    // Verify environment was created in custom location
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-destroy-custom");

    // Act: Destroy environment using same working directory
    let destroy_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_destroy_command("test-destroy-custom")
        .expect("Failed to run destroy command");

    // Assert: Verify command succeeded
    assert!(
        destroy_result.success(),
        "Destroy command failed with exit code: {:?}\nstderr: {}",
        destroy_result.exit_code(),
        destroy_result.stderr()
    );

    // Assert: Verify environment was transitioned to Destroyed state in custom location
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-destroy-custom");
    env_assertions.assert_environment_state_is("test-destroy-custom", "Destroyed");
}

#[test]
fn it_should_fail_when_environment_not_found_in_working_directory() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Act: Try to destroy non-existent environment
    let destroy_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_destroy_command("nonexistent-environment")
        .expect("Failed to run destroy command");

    // Assert: Command should fail
    assert!(
        !destroy_result.success(),
        "Command should have failed when environment doesn't exist"
    );

    // Verify error message mentions environment not found
    let stderr = destroy_result.stderr();
    assert!(
        stderr.contains("not found") || stderr.contains("does not exist"),
        "Error message should mention environment not found, got: {stderr}"
    );
}

#[test]
fn it_should_complete_full_lifecycle_with_custom_working_directory() {
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-lifecycle");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    // Act: Create environment in custom location
    let create_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_create_command("./environment.json")
        .expect("Failed to run create command");

    assert!(
        create_result.success(),
        "Create command failed: {}",
        create_result.stderr()
    );

    // Verify environment exists
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-lifecycle");
    env_assertions.assert_environment_state_is("test-lifecycle", "Created");

    // Act: Destroy environment
    let destroy_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_destroy_command("test-lifecycle")
        .expect("Failed to run destroy command");

    // Assert: Both commands succeed
    assert!(
        destroy_result.success(),
        "Destroy command failed: {}",
        destroy_result.stderr()
    );

    // Assert: Environment is transitioned to Destroyed state
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-lifecycle");
    env_assertions.assert_environment_state_is("test-lifecycle", "Destroyed");
}
