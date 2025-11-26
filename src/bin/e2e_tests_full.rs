//! Full End-to-End Testing Binary for Torrust Tracker Deployer (LOCAL DEVELOPMENT ONLY)
//!
//! This binary provides complete end-to-end testing by combining infrastructure provisioning,
//! configuration management, and validation in a single LXD VM. It's designed for local
//! development and comprehensive testing workflows.
//!
//! ⚠️ **IMPORTANT**: This binary cannot run on GitHub Actions due to network connectivity
//! issues within LXD VMs on GitHub runners. For CI environments, use the split test suites:
//! - `cargo run --bin e2e-provision-and-destroy-tests` - Infrastructure provisioning only
//! - `cargo run --bin e2e-config-tests` - Configuration and software installation
//!
//! ## Usage
//!
//! Run the full E2E test suite:
//!
//! ```bash
//! cargo run --bin e2e-tests-full
//! ```
//!
//! Run with custom options:
//!
//! ```bash
//! # Keep test environment after completion (for debugging)
//! cargo run --bin e2e-tests-full -- --keep
//!
//! # Change logging format
//! cargo run --bin e2e-tests-full -- --log-format json
//!
//! # Show help
//! cargo run --bin e2e-tests-full -- --help
//! ```
//!
//! ## Test Workflow
//!
//! 1. **Preflight cleanup** - Remove any artifacts from previous test runs
//! 2. **Create environment** - Execute `create environment` CLI command
//! 3. **Provision infrastructure** - Execute `provision` CLI command (creates LXD VM)
//! 4. **Configure services** - Execute `configure` CLI command (runs Ansible playbooks)
//! 5. **Validate deployment** - Execute `test` CLI command (verifies services)
//! 6. **Destroy infrastructure** - Execute `destroy` CLI command (cleanup)
//!
//! ## Black-Box Testing Approach
//!
//! This test executes the CLI commands as external processes, without importing
//! application or domain layer logic. This ensures we test the public interface
//! exactly as end-users would use it.

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use std::time::Instant;
use torrust_dependency_installer::Dependency;
use tracing::{error, info, warn};

use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput, LoggingBuilder};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::verify_required_dependencies;
use torrust_tracker_deployer_lib::testing::e2e::tasks::virtual_machine::preflight_cleanup::{
    preflight_cleanup_previous_resources, PreflightCleanupContext,
};
use torrust_tracker_deployer_lib::testing::e2e::ProcessRunner;

// Constants for the e2e-full environment
const ENVIRONMENT_NAME: &str = "e2e-full";

#[derive(Parser)]
#[command(name = "e2e-tests-full")]
#[command(about = "Full E2E tests for Torrust Tracker Deployer (LOCAL ONLY)")]
struct Cli {
    /// Keep the test environment after completion (skip destroy step)
    #[arg(long)]
    keep: bool,

    /// Logging format to use
    #[arg(
        long,
        default_value = "pretty",
        help = "Logging format: pretty, json, or compact"
    )]
    log_format: LogFormat,
}

/// Main entry point for the full E2E test suite
///
/// This function orchestrates the complete E2E test workflow:
/// 1. Initializes logging
/// 2. Verifies required dependencies
/// 3. Performs pre-flight cleanup
/// 4. Executes CLI commands: create → provision → configure → test → destroy
/// 5. Reports results
///
/// Returns `Ok(())` if all tests pass, `Err` otherwise.
///
/// # Errors
///
/// This function may return errors in the following cases:
/// - Required dependencies are missing
/// - Pre-flight cleanup encounters issues
/// - Any CLI command fails (non-zero exit code)
fn main() -> Result<()> {
    let cli = Cli::parse();

    LoggingBuilder::new(std::path::Path::new("./data/e2e-full/logs"))
        .with_format(cli.log_format.clone())
        .with_output(LogOutput::FileAndStderr)
        .init();

    info!(
        application = "torrust_tracker_deployer",
        test_suite = "e2e_tests_full",
        log_format = ?cli.log_format,
        "Starting full E2E tests (black-box, LOCAL ONLY)"
    );

    verify_required_dependencies(&[Dependency::OpenTofu, Dependency::Ansible, Dependency::Lxd])?;

    run_preflight_cleanup(ENVIRONMENT_NAME)?;

    let test_start = Instant::now();

    let test_result = run_e2e_test_workflow(ENVIRONMENT_NAME, !cli.keep);

    let test_duration = test_start.elapsed();

    info!(
        performance = "test_execution",
        duration_secs = test_duration.as_secs_f64(),
        duration = ?test_duration,
        "Full E2E test execution completed"
    );

    // Report final results
    match &test_result {
        Ok(()) => {
            info!(
                test_suite = "e2e_tests_full",
                status = "success",
                "All full E2E tests passed successfully"
            );
        }
        Err(e) => {
            error!(
                test_suite = "e2e_tests_full",
                status = "failed",
                error = %e,
                "Full E2E test failed"
            );
        }
    }

    test_result
}

/// Performs preflight cleanup to remove artifacts from previous test runs.
///
/// This ensures a clean slate before starting new tests by removing:
/// - Build directory
/// - Templates directory
/// - Data directory for this environment
/// - LXD resources (instance and profile)
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to clean up
///
/// # Errors
///
/// Returns an error if cleanup fails.
fn run_preflight_cleanup(environment_name: &str) -> Result<()> {
    use torrust_tracker_deployer_lib::domain::EnvironmentName;

    info!(
        operation = "preflight_cleanup",
        environment = environment_name,
        "Running preflight cleanup"
    );

    // Create preflight cleanup context with paths for the environment
    let cleanup_context = PreflightCleanupContext::new(
        format!("./build/{environment_name}").into(),
        format!("./templates/{environment_name}").into(),
        EnvironmentName::new(environment_name).expect("Valid environment name"),
        format!("torrust-tracker-vm-{environment_name}")
            .try_into()
            .expect("Valid instance name"),
        format!("torrust-profile-{environment_name}")
            .try_into()
            .expect("Valid profile name"),
    );

    preflight_cleanup_previous_resources(&cleanup_context)?;

    info!(
        operation = "preflight_cleanup",
        status = "success",
        "Preflight cleanup completed"
    );

    Ok(())
}

/// Runs the full E2E test workflow using CLI commands.
///
/// Executes the following commands in sequence:
/// 1. `create environment` - Create the environment from config
/// 2. `provision` - Provision the infrastructure (LXD VM)
/// 3. `configure` - Configure services (Ansible playbooks)
/// 4. `test` - Validate deployment
/// 5. `destroy` - Destroy the infrastructure (if `destroy` is true)
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to test
/// * `destroy` - If true, destroy the infrastructure after testing; if false, keep it for debugging
///
/// # Errors
///
/// Returns an error if any command fails.
fn run_e2e_test_workflow(environment_name: &str, destroy: bool) -> Result<()> {
    let runner = ProcessRunner::new();

    // Generate config file with absolute paths for SSH keys
    let config_path = generate_environment_config(environment_name)?;

    // Step 1: Create environment
    create_environment(&runner, &config_path)?;

    // Step 2: Provision infrastructure
    provision_infrastructure(&runner, environment_name, destroy)?;

    // Step 3: Configure services
    configure_services(&runner, environment_name, destroy)?;

    // Step 4: Validate deployment
    validate_deployment(&runner, environment_name)?;

    // Step 5: Destroy infrastructure
    if destroy {
        destroy_infrastructure(&runner, environment_name)?;
    } else {
        info!(
            step = "destroy",
            status = "skipped",
            reason = "keep flag is set",
            "Skipping infrastructure destruction"
        );
    }

    Ok(())
}

/// Generates the environment configuration file with absolute SSH key paths.
///
/// This function creates a configuration file with absolute paths
/// to the SSH keys, ensuring they work correctly regardless of the directory
/// from which Ansible runs.
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to create
///
/// # Returns
///
/// Returns the path to the generated configuration file.
///
/// # Errors
///
/// Returns an error if the configuration file cannot be created.
fn generate_environment_config(environment_name: &str) -> Result<PathBuf> {
    use std::fs;

    // Get project root from current directory (cargo run runs from project root)
    let project_root = std::env::current_dir()
        .map_err(|e| anyhow::anyhow!("Failed to get current directory: {e}"))?;

    // Build absolute paths to SSH keys
    let private_key_path = project_root.join("fixtures/testing_rsa");
    let public_key_path = project_root.join("fixtures/testing_rsa.pub");

    // Verify SSH keys exist
    if !private_key_path.exists() {
        return Err(anyhow::anyhow!(
            "SSH private key not found at: {}",
            private_key_path.display()
        ));
    }
    if !public_key_path.exists() {
        return Err(anyhow::anyhow!(
            "SSH public key not found at: {}",
            public_key_path.display()
        ));
    }

    // Create configuration JSON with absolute paths
    let config = serde_json::json!({
        "environment": {
            "name": environment_name
        },
        "ssh_credentials": {
            "private_key_path": private_key_path.to_string_lossy(),
            "public_key_path": public_key_path.to_string_lossy()
        }
    });

    // Write to envs directory
    let config_path = project_root.join(format!("envs/{environment_name}.json"));

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Failed to create config directory: {e}"))?;
    }

    fs::write(&config_path, serde_json::to_string_pretty(&config)?)
        .map_err(|e| anyhow::anyhow!("Failed to write config file: {e}"))?;

    info!(
        config_path = %config_path.display(),
        private_key = %private_key_path.display(),
        public_key = %public_key_path.display(),
        "Generated environment configuration with absolute SSH key paths"
    );

    Ok(config_path)
}

/// Creates the environment from the configuration file.
///
/// # Arguments
///
/// * `runner` - The process runner to execute CLI commands
/// * `config_path` - Path to the environment configuration file
///
/// # Errors
///
/// Returns an error if the create command fails.
fn create_environment(runner: &ProcessRunner, config_path: &std::path::Path) -> Result<()> {
    info!(
        step = "create_environment",
        config_path = %config_path.display(),
        "Creating environment from config file"
    );

    let create_result = runner
        .run_create_command(config_path.to_str().expect("Valid UTF-8 path"))
        .map_err(|e| anyhow::anyhow!("Failed to execute create command: {e}"))?;

    if !create_result.success() {
        error!(
            step = "create_environment",
            exit_code = ?create_result.exit_code(),
            stderr = %create_result.stderr(),
            "Create environment command failed"
        );
        return Err(anyhow::anyhow!(
            "Create environment failed with exit code {:?}",
            create_result.exit_code()
        ));
    }

    info!(
        step = "create_environment",
        status = "success",
        "Environment created successfully"
    );

    Ok(())
}

/// Provisions the infrastructure for the environment.
///
/// # Arguments
///
/// * `runner` - The process runner to execute CLI commands
/// * `environment_name` - The name of the environment to provision
/// * `destroy_on_failure` - If true, attempt to destroy infrastructure on failure
///
/// # Errors
///
/// Returns an error if the provision command fails.
fn provision_infrastructure(
    runner: &ProcessRunner,
    environment_name: &str,
    destroy_on_failure: bool,
) -> Result<()> {
    info!(
        step = "provision",
        environment = environment_name,
        "Provisioning infrastructure"
    );

    let provision_result = runner
        .run_provision_command(environment_name)
        .map_err(|e| anyhow::anyhow!("Failed to execute provision command: {e}"))?;

    if !provision_result.success() {
        error!(
            step = "provision",
            exit_code = ?provision_result.exit_code(),
            stderr = %provision_result.stderr(),
            "Provision command failed"
        );

        // Try to cleanup even if provision failed
        if destroy_on_failure {
            warn!(
                step = "cleanup_after_failure",
                "Attempting to destroy infrastructure after provision failure"
            );
            // Ignore destroy result - we're already in an error state
            drop(runner.run_destroy_command(environment_name));
        }

        return Err(anyhow::anyhow!(
            "Provision failed with exit code {:?}",
            provision_result.exit_code()
        ));
    }

    info!(
        step = "provision",
        status = "success",
        "Infrastructure provisioned successfully"
    );

    Ok(())
}

/// Configures services on the provisioned infrastructure.
///
/// # Arguments
///
/// * `runner` - The process runner to execute CLI commands
/// * `environment_name` - The name of the environment to configure
/// * `destroy_on_failure` - If true, attempt to destroy infrastructure on failure
///
/// # Errors
///
/// Returns an error if the configure command fails.
fn configure_services(
    runner: &ProcessRunner,
    environment_name: &str,
    destroy_on_failure: bool,
) -> Result<()> {
    info!(
        step = "configure",
        environment = environment_name,
        "Configuring services"
    );

    let configure_result = runner
        .run_configure_command(environment_name)
        .map_err(|e| anyhow::anyhow!("Failed to execute configure command: {e}"))?;

    if !configure_result.success() {
        error!(
            step = "configure",
            exit_code = ?configure_result.exit_code(),
            stderr = %configure_result.stderr(),
            "Configure command failed"
        );

        // Try to cleanup even if configure failed
        if destroy_on_failure {
            warn!(
                step = "cleanup_after_failure",
                "Attempting to destroy infrastructure after configure failure"
            );
            // Ignore destroy result - we're already in an error state
            drop(runner.run_destroy_command(environment_name));
        }

        return Err(anyhow::anyhow!(
            "Configure failed with exit code {:?}",
            configure_result.exit_code()
        ));
    }

    info!(
        step = "configure",
        status = "success",
        "Services configured successfully"
    );

    Ok(())
}

/// Validates the deployment by running the test command.
///
/// # Arguments
///
/// * `runner` - The process runner to execute CLI commands
/// * `environment_name` - The name of the environment to validate
///
/// # Errors
///
/// Returns an error if the test command fails.
fn validate_deployment(runner: &ProcessRunner, environment_name: &str) -> Result<()> {
    info!(
        step = "test",
        environment = environment_name,
        "Validating deployment"
    );

    let test_result = runner
        .run_test_command(environment_name)
        .map_err(|e| anyhow::anyhow!("Failed to execute test command: {e}"))?;

    if !test_result.success() {
        error!(
            step = "test",
            exit_code = ?test_result.exit_code(),
            stderr = %test_result.stderr(),
            "Test command failed"
        );
        return Err(anyhow::anyhow!(
            "Test failed with exit code {:?}",
            test_result.exit_code()
        ));
    }

    info!(
        step = "test",
        status = "success",
        "Deployment validated successfully"
    );

    Ok(())
}

/// Destroys the infrastructure for the environment.
///
/// # Arguments
///
/// * `runner` - The process runner to execute CLI commands
/// * `environment_name` - The name of the environment to destroy
///
/// # Errors
///
/// Returns an error if the destroy command fails.
fn destroy_infrastructure(runner: &ProcessRunner, environment_name: &str) -> Result<()> {
    info!(
        step = "destroy",
        environment = environment_name,
        "Destroying infrastructure"
    );

    let destroy_result = runner
        .run_destroy_command(environment_name)
        .map_err(|e| anyhow::anyhow!("Failed to execute destroy command: {e}"))?;

    if !destroy_result.success() {
        error!(
            step = "destroy",
            exit_code = ?destroy_result.exit_code(),
            stderr = %destroy_result.stderr(),
            "Destroy command failed"
        );
        return Err(anyhow::anyhow!(
            "Destroy failed with exit code {:?}",
            destroy_result.exit_code()
        ));
    }

    info!(
        step = "destroy",
        status = "success",
        "Infrastructure destroyed successfully"
    );

    Ok(())
}
