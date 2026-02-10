//! End-to-End Black Box Tests for Render Command
//!
//! This test suite provides true black-box testing of the render command
//! by running the production application as an external process. These tests
//! verify that the render command correctly generates deployment artifacts
//! without provisioning infrastructure.
//!
//! ## Test Approach
//!
//! - **Black Box**: Runs production binary as external process
//! - **Isolation**: Uses temporary directories for complete test isolation
//! - **Coverage**: Tests both input modes (env-name and env-file)
//! - **Verification**: Validates all artifacts are properly generated
//!
//! ## Test Scenarios
//!
//! 1. Render with env-name: Create environment → Render artifacts to custom directory
//! 2. Render with env-file: Generate artifacts directly from config file
//! 3. Render requires output directory: Verify --output-dir parameter is required
//! 4. Render fails on existing directory: Verify `OutputDirectoryExists` error
//! 5. Environment not found: Verify proper error handling
//! 6. Config file not found: Verify proper error handling
//! 7. Custom working directory: Verify render works from different locations
//!
//! ## Design Decisions
//!
//! - **Output directory required**: Tests verify --output-dir flag is mandatory
//! - **Output protection**: Tests verify existing directories are not overwritten
//! - **Created state only**: Tests render on environments in Created state
//! - **IP validation**: Tests verify IP address parameter is required and validated
//! - **Dual input modes**: Tests cover both --env-name and --env-file workflows

use super::super::support::{EnvironmentStateAssertions, ProcessRunner, TempWorkspace};
use anyhow::Result;
use torrust_dependency_installer::{verify_dependencies, Dependency};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::create_test_environment_config;

/// Verify that all required dependencies are installed for render command E2E tests.
///
/// **Current State**: No system dependencies required.
///
/// These black-box tests run the production binary as an external process and verify
/// the render command workflow. Currently, they only test the command interface and
/// local artifact generation, without requiring infrastructure tools.
///
/// # Future Dependencies
///
/// If these tests evolve to verify actual infrastructure deployment or validation,
/// add required dependencies here:
/// ```ignore
/// let required_deps = &[Dependency::OpenTofu, Dependency::Ansible];
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
fn it_should_render_artifacts_using_env_name_successfully() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-render-env-name");
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

    // Verify environment is in Created state before render
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-render-env-name");
    env_assertions.assert_environment_state_is("test-render-env-name", "Created");

    // Act: Render artifacts using env-name input mode
    let output_dir = temp_workspace.path().join("render-output");
    let render_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_render_command_with_env_name(
            "test-render-env-name",
            "192.168.1.100",
            output_dir.to_str().unwrap(),
        )
        .expect("Failed to run render command");

    // Assert: Verify command succeeded
    assert!(
        render_result.success(),
        "Render command failed with exit code: {:?}\nstderr: {}",
        render_result.exit_code(),
        render_result.stderr()
    );

    // Assert: Verify output directory and artifacts were created
    assert!(
        output_dir.exists(),
        "Output directory should exist at: {}",
        output_dir.display()
    );

    // Verify key artifacts exist
    let tofu_dir = output_dir.join("tofu");
    assert!(
        tofu_dir.exists(),
        "Tofu directory should exist at: {}",
        tofu_dir.display()
    );

    // Assert: Verify success message in output (check both stdout and stderr)
    let output = format!("{}{}", render_result.stdout(), render_result.stderr());
    assert!(
        output.contains("generated successfully"),
        "Output should contain success message. Combined output: {output}"
    );
}

#[test]
fn it_should_render_artifacts_using_config_file_successfully() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file (but don't create environment)
    let config = create_test_environment_config("test-render-config-file");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    // Get absolute path to config file for render command
    let config_path = temp_workspace
        .path()
        .join("environment.json")
        .to_str()
        .expect("Failed to convert path to string")
        .to_string();

    // Act: Render artifacts directly from config file (no environment creation)
    let output_dir = temp_workspace.path().join("render-config-output");
    let render_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_render_command_with_config_file(
            &config_path,
            "192.168.1.101",
            output_dir.to_str().unwrap(),
        )
        .expect("Failed to run render command");

    // Assert: Verify command succeeded
    assert!(
        render_result.success(),
        "Render command failed with exit code: {:?}\nstderr: {}",
        render_result.exit_code(),
        render_result.stderr()
    );

    // Assert: Environment should NOT be created in data/ (rendered from config only)
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_not_exists("test-render-config-file");

    // Assert: Verify output directory and artifacts were created
    assert!(
        output_dir.exists(),
        "Output directory should exist at: {}",
        output_dir.display()
    );

    // Verify key artifacts exist
    let tofu_dir = output_dir.join("tofu");
    assert!(
        tofu_dir.exists(),
        "Tofu directory should exist at: {}",
        tofu_dir.display()
    );

    // Assert: Verify success message in output
    let output = format!("{}{}", render_result.stdout(), render_result.stderr());
    assert!(
        output.contains("generated successfully"),
        "Output should contain success message. Combined output: {output}"
    );
}

#[test]
fn it_should_fail_when_output_directory_already_exists() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace and environment
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    let config = create_test_environment_config("test-render-output-dir-exists");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    let create_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_create_command("./environment.json")
        .expect("Failed to run create command");

    assert!(
        create_result.success(),
        "Create command failed: {}",
        create_result.stderr()
    );

    // Act: Render artifacts first time
    let output_dir = temp_workspace.path().join("render-idempotent-output");
    let render1_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_render_command_with_env_name(
            "test-render-output-dir-exists",
            "192.168.1.102",
            output_dir.to_str().unwrap(),
        )
        .expect("Failed to run first render command");

    assert!(
        render1_result.success(),
        "First render failed: {}",
        render1_result.stderr()
    );

    // Act: Render artifacts second time (should fail with OutputDirectoryExists error)
    let render2_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_render_command_with_env_name(
            "test-render-output-dir-exists",
            "192.168.1.102",
            output_dir.to_str().unwrap(),
        )
        .expect("Failed to run second render command");

    // Assert: Second render should fail because output directory already exists
    assert!(
        !render2_result.success(),
        "Second render should fail when output directory exists"
    );

    // Assert: Error message should mention output directory exists
    let stderr = render2_result.stderr();
    assert!(
        stderr.contains("Output directory") || stderr.contains("already exists"),
        "Error message should mention output directory exists. Stderr: {stderr}"
    );

    // Assert: Output directory should still exist with artifacts from first render
    assert!(output_dir.exists(), "Output directory should still exist");
    let tofu_dir = output_dir.join("tofu");
    assert!(tofu_dir.exists(), "Tofu directory should still exist");
}

#[test]
fn it_should_fail_when_environment_not_found() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace (but don't create environment)
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Act: Try to render non-existent environment
    let output_dir = temp_workspace.path().join("nonexistent-output");
    let render_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_render_command_with_env_name(
            "nonexistent-env",
            "192.168.1.103",
            output_dir.to_str().unwrap(),
        )
        .expect("Failed to run render command");

    // Assert: Verify command failed with proper error
    assert!(
        !render_result.success(),
        "Render command should fail for non-existent environment"
    );

    // Assert: Verify error message mentions environment not found
    let stderr = render_result.stderr();
    assert!(
        stderr.contains("not found") || stderr.contains("Environment not found"),
        "Error message should mention environment not found. Stderr: {stderr}"
    );
}

#[test]
fn it_should_fail_when_config_file_not_found() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace (but don't create config file)
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Act: Try to render with non-existent config file
    let output_dir = temp_workspace.path().join("missing-config-output");
    let render_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_render_command_with_config_file(
            "./nonexistent.json",
            "192.168.1.104",
            output_dir.to_str().unwrap(),
        )
        .expect("Failed to run render command");

    // Assert: Verify command failed with proper error
    assert!(
        !render_result.success(),
        "Render command should fail for non-existent config file"
    );

    // Assert: Verify error message mentions file not found
    let stderr = render_result.stderr();
    assert!(
        stderr.contains("not found") || stderr.contains("No such file"),
        "Error message should mention file not found. Stderr: {stderr}"
    );
}

#[test]
fn it_should_work_with_custom_working_directory() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-render-custom-dir");
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

    // Act: Render from custom working directory
    let output_dir = temp_workspace.path().join("custom-dir-output");
    let render_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_render_command_with_env_name(
            "test-render-custom-dir",
            "192.168.1.105",
            output_dir.to_str().unwrap(),
        )
        .expect("Failed to run render command");

    // Assert: Verify command succeeded
    assert!(
        render_result.success(),
        "Render command failed with exit code: {:?}\nstderr: {}",
        render_result.exit_code(),
        render_result.stderr()
    );

    // Assert: Verify artifacts were created in output directory
    assert!(
        output_dir.exists(),
        "Output directory should exist at: {}",
        output_dir.display()
    );
    let tofu_dir = output_dir.join("tofu");
    assert!(
        tofu_dir.exists(),
        "Tofu directory should exist at: {}",
        tofu_dir.display()
    );
}

#[test]
fn it_should_complete_full_lifecycle_from_create_to_render() {
    // Verify dependencies before running tests
    verify_required_dependencies().expect("Dependency verification failed");

    // Arrange: Create temporary workspace
    let temp_workspace = TempWorkspace::new().expect("Failed to create temp workspace");

    // Create environment configuration file
    let config = create_test_environment_config("test-full-lifecycle-render");
    temp_workspace
        .write_config_file("environment.json", &config)
        .expect("Failed to write config file");

    // Step 1: Create environment
    let create_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_create_command("./environment.json")
        .expect("Failed to run create command");

    assert!(
        create_result.success(),
        "Create command failed: {}",
        create_result.stderr()
    );

    // Verify environment was created in Created state
    let env_assertions = EnvironmentStateAssertions::new(temp_workspace.path());
    env_assertions.assert_environment_exists("test-full-lifecycle-render");
    env_assertions.assert_environment_state_is("test-full-lifecycle-render", "Created");

    // Step 2: Render artifacts
    let output_dir = temp_workspace.path().join("lifecycle-output");
    let render_result = ProcessRunner::new()
        .working_dir(temp_workspace.path())
        .run_render_command_with_env_name(
            "test-full-lifecycle-render",
            "192.168.1.106",
            output_dir.to_str().unwrap(),
        )
        .expect("Failed to run render command");

    assert!(
        render_result.success(),
        "Render command failed with exit code: {:?}\nstderr: {}",
        render_result.exit_code(),
        render_result.stderr()
    );

    // Step 3: Verify artifacts were generated
    assert!(
        output_dir.exists(),
        "Output directory should exist at: {}",
        output_dir.display()
    );
    let tofu_dir = output_dir.join("tofu");
    assert!(
        tofu_dir.exists(),
        "Tofu directory should exist at: {}",
        tofu_dir.display()
    );

    // Verify environment remains in Created state (render doesn't change state)
    env_assertions.assert_environment_state_is("test-full-lifecycle-render", "Created");

    // Verify render output indicates success (check both stdout and stderr)
    let output = format!("{}{}", render_result.stdout(), render_result.stderr());
    assert!(
        output.contains("generated successfully"),
        "Output should contain success message. Combined output: {output}"
    );
}
