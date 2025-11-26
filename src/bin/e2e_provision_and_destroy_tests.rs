//! End-to-End Provisioning and Destruction Tests for Torrust Tracker Deployer
//!
//! This binary tests the complete infrastructure lifecycle: provisioning and destruction.
//! It executes the CLI commands as a black box, testing the public interface exactly as
//! end-users would use it. This does NOT test software configuration or installation.
//!
//! ## Usage
//!
//! Run the E2E provisioning and destruction tests:
//!
//! ```bash
//! cargo run --bin e2e-provision-and-destroy-tests
//! ```
//!
//! Run with custom options:
//!
//! ```bash
//! # Keep test environment after completion (for debugging)
//! cargo run --bin e2e-provision-and-destroy-tests -- --keep
//!
//! # Change logging format
//! cargo run --bin e2e-provision-and-destroy-tests -- --log-format json
//!
//! # Show help
//! cargo run --bin e2e-provision-and-destroy-tests -- --help
//! ```
//!
//! ## Test Workflow
//!
//! 1. **Preflight cleanup** - Remove any artifacts from previous test runs
//! 2. **Create environment** - Execute `create environment` CLI command
//! 3. **Provision infrastructure** - Execute `provision` CLI command
//! 4. **Destroy infrastructure** - Execute `destroy` CLI command
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

// Constants for the e2e-provision environment
const ENVIRONMENT_NAME: &str = "e2e-provision";

#[derive(Parser)]
#[command(name = "e2e-provision-and-destroy-tests")]
#[command(about = "E2E provisioning and destruction tests for Torrust Tracker Deployer")]
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

/// Main entry point for the E2E provisioning and destruction test suite
///
/// This function orchestrates the complete provision+destroy test workflow:
/// 1. Initializes logging
/// 2. Verifies required dependencies
/// 3. Performs pre-flight cleanup
/// 4. Executes CLI commands: create → provision → destroy
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

    LoggingBuilder::new(std::path::Path::new("./data/e2e-provision/logs"))
        .with_format(cli.log_format.clone())
        .with_output(LogOutput::FileAndStderr)
        .init();

    info!(
        application = "torrust_tracker_deployer",
        test_suite = "e2e_provision_and_destroy_tests",
        log_format = ?cli.log_format,
        "Starting E2E provisioning and destruction tests (black-box)"
    );

    verify_required_dependencies(&[Dependency::Ansible])?;

    run_preflight_cleanup(ENVIRONMENT_NAME)?;

    let test_start = Instant::now();

    let test_result = run_e2e_test_workflow(ENVIRONMENT_NAME, !cli.keep);

    let test_duration = test_start.elapsed();

    info!(
        performance = "test_execution",
        duration_secs = test_duration.as_secs_f64(),
        duration = ?test_duration,
        "Provisioning and destruction test execution completed"
    );

    // Report final results
    match &test_result {
        Ok(()) => {
            info!(
                test_suite = "e2e_provision_and_destroy_tests",
                status = "success",
                "All provisioning and destruction tests passed successfully"
            );
        }
        Err(e) => {
            error!(
                test_suite = "e2e_provision_and_destroy_tests",
                status = "failed",
                error = %e,
                "E2E test failed"
            );
        }
    }

    test_result
}

/// Runs the E2E test workflow using CLI commands.
///
/// Executes the following commands in sequence:
/// 1. `create environment` - Create the environment from config
/// 2. `provision` - Provision the infrastructure
/// 3. `destroy` - Destroy the infrastructure (if `destroy` is true)
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to test
/// * `destroy` - If true, destroy the infrastructure after provisioning; if false, keep it for debugging
///
/// # Errors
///
/// Returns an error if any command fails.
fn run_e2e_test_workflow(environment_name: &str, destroy: bool) -> Result<()> {
    let test_runner = E2eTestRunner::new(environment_name).with_cleanup_on_failure(destroy);

    let config_path = generate_environment_config(environment_name)?;

    test_runner.create_environment(&config_path)?;

    test_runner.provision_infrastructure()?;

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
