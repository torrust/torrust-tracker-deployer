//! Full End-to-End Testing Binary for Torrust Tracker Deployer (LOCAL DEVELOPMENT ONLY)
//!
//! This binary provides complete end-to-end testing by combining infrastructure provisioning,
//! configuration management, and validation in a single LXD VM. It's designed for local
//! development and comprehensive testing workflows.
//!
//! ⚠️ **IMPORTANT**: This binary cannot run on GitHub Actions due to network connectivity
//! issues within LXD VMs on GitHub runners. For CI environments, use the split test suites:
//! - `cargo run --bin e2e-infrastructure-lifecycle-tests` - Infrastructure provisioning only
//! - `cargo run --bin e2e-deployment-workflow-tests` - Configuration, release, and run workflows
//!
//! ## Usage
//!
//! Run the full E2E test suite:
//!
//! ```bash
//! cargo run --bin e2e-complete-workflow-tests
//! ```
//!
//! Run with custom options:
//!
//! ```bash
//! # Keep test environment after completion (for debugging)
//! cargo run --bin e2e-complete-workflow-tests -- --keep
//!
//! # Change logging format
//! cargo run --bin e2e-complete-workflow-tests -- --log-format json
//!
//! # Show help
//! cargo run --bin e2e-complete-workflow-tests -- --help
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
use std::time::Instant;
use torrust_dependency_installer::Dependency;
use tracing::{error, info};

use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput, LoggingBuilder};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::{
    generate_environment_config, run_preflight_cleanup, verify_required_dependencies, E2eTestRunner,
};

// Constants for the e2e-complete environment
const ENVIRONMENT_NAME: &str = "e2e-complete";

#[derive(Parser)]
#[command(name = "e2e-complete-workflow-tests")]
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

    LoggingBuilder::new(std::path::Path::new("./data/e2e-complete/logs"))
        .with_format(cli.log_format.clone())
        .with_output(LogOutput::FileAndStderr)
        .init();

    info!(
        application = "torrust_tracker_deployer",
        test_suite = "e2e_complete_workflow_tests",
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
                test_suite = "e2e_complete_workflow_tests",
                status = "success",
                "All full E2E tests passed successfully"
            );

            // Clean up test state after successful run (only if destroy happened)
            // If --keep flag is used, we skip purge to preserve state for debugging
            if !cli.keep {
                let test_runner = E2eTestRunner::new(ENVIRONMENT_NAME);
                // Note: Purge failures are logged but don't fail the test since it already succeeded
                if let Err(purge_error) = test_runner.purge_environment() {
                    tracing::warn!(
                        operation = "post_test_cleanup",
                        environment = ENVIRONMENT_NAME,
                        error = %purge_error,
                        "Failed to purge test state after successful run (test still passed)"
                    );
                } else {
                    info!(
                        operation = "post_test_cleanup",
                        environment = ENVIRONMENT_NAME,
                        "Test state purged successfully"
                    );
                }
            } else {
                info!(
                    operation = "post_test_cleanup",
                    environment = ENVIRONMENT_NAME,
                    status = "skipped",
                    reason = "keep flag is set",
                    "Skipping post-test purge to preserve state for debugging"
                );
            }
        }
        Err(e) => {
            error!(
                test_suite = "e2e_complete_workflow_tests",
                status = "failed",
                error = %e,
                "Full E2E test failed"
            );
        }
    }

    test_result
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
    let test_runner = E2eTestRunner::new(environment_name).with_cleanup_on_failure(destroy);

    let config_path = generate_environment_config(environment_name)?;

    test_runner.create_environment(&config_path)?;

    test_runner.provision_infrastructure()?;

    test_runner.configure_services()?;

    test_runner.release_software()?;

    test_runner.run_services()?;

    test_runner.validate_deployment()?;

    if destroy {
        test_runner.destroy_infrastructure()?;
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
