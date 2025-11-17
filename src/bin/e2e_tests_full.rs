//! Full End-to-End Testing Binary for Torrust Tracker Deployer (LOCAL DEVELOPMENT ONLY)
//!
//! This binary provides complete end-to-end testing by combining infrastructure provisioning
//! and configuration management in a single LXD VM. It's designed for local development
//! and comprehensive testing workflows.
//!
//! ⚠️ **IMPORTANT**: This binary cannot run on GitHub Actions due to network connectivity
//! issues within LXD VMs on GitHub runners. For CI environments, use the split test suites:
//! - `cargo run --bin e2e-provision-tests` - Infrastructure provisioning only
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
//! # Use specific environment name
//! cargo run --bin e2e-tests-full -- --environment e2e-staging
//!
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
//! 1. **Preflight cleanup** - Remove any artifacts from previous test runs that may have failed to clean up
//! 2. **Infrastructure provisioning** - Create LXD VMs using `OpenTofu`
//! 3. **Configuration** - Apply Ansible playbooks for software installation
//! 4. **Validation** - Verify deployments are working correctly
//! 5. **Test infrastructure cleanup** - Remove test resources created during this run
//!
//! ## Two-Phase Cleanup Strategy
//!
//! The cleanup process happens in two distinct phases:
//!
//! - **Phase 1 - Preflight cleanup**: Removes artifacts from previous test runs that may have
//!   failed to clean up properly (executed at the start in main function)
//! - **Phase 2 - Test infrastructure cleanup**: Destroys resources created specifically during
//!   the current test run (executed at the end in main function)
//!
//! The test suite supports different VM providers (LXD, Multipass) and includes
//! comprehensive logging and error reporting.

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use std::time::{Duration, Instant};
use torrust_dependency_installer::{verify_dependencies, Dependency};
use tracing::{error, info};

// Import E2E testing infrastructure
use torrust_tracker_deployer_lib::adapters::ssh::DEFAULT_SSH_PORT;
use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput, LoggingBuilder};
use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
use torrust_tracker_deployer_lib::shared::{Clock, SystemClock};
use torrust_tracker_deployer_lib::testing::e2e::context::{TestContext, TestContextType};
use torrust_tracker_deployer_lib::testing::e2e::tasks::{
    preflight_cleanup::cleanup_previous_test_data,
    run_configure_command::run_configure_command,
    run_create_command::run_create_command,
    run_test_command::run_test_command,
    virtual_machine::{
        preflight_cleanup::preflight_cleanup_previous_resources,
        run_destroy_command::run_destroy_command, run_provision_command::run_provision_command,
    },
};

#[derive(Parser)]
#[command(name = "e2e-tests")]
#[command(about = "E2E tests for Torrust Tracker Deployer")]
struct Cli {
    /// Keep the test environment after completion
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

/// Main entry point for E2E tests.
///
/// Runs the full deployment workflow: provision infrastructure, configure services,
/// validate deployment, and cleanup resources.
///
/// # Errors
///
/// Returns an error if:
/// - Required dependencies are missing
/// - Invalid environment name provided via CLI
/// - Pre-flight cleanup fails
/// - Infrastructure provisioning fails  
/// - Service configuration fails
/// - Deployment validation fails
/// - Resource cleanup fails (when enabled)
///
/// # Panics
///
/// May panic during the match statement if unexpected error combinations occur
/// that are not handled by the current error handling logic.
#[allow(clippy::too_many_lines)]
#[tokio::main]
pub async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on the chosen format with stderr output for test visibility
    // E2E tests use production log location: ./data/logs using the builder pattern
    LoggingBuilder::new(std::path::Path::new("./data/logs"))
        .with_format(cli.log_format.clone())
        .with_output(LogOutput::FileAndStderr)
        .init();

    info!(
        application = "torrust_tracker_deployer",
        test_suite = "e2e_tests",
        log_format = ?cli.log_format,
        "Starting E2E tests"
    );

    verify_required_dependencies()?;

    // Use absolute paths to project root for SSH keys to ensure they can be found by Ansible
    let project_root = std::env::current_dir().expect("Failed to get current directory");
    let ssh_private_key_path = project_root.join("fixtures/testing_rsa");
    let ssh_public_key_path = project_root.join("fixtures/testing_rsa.pub");

    // Cleanup any artifacts from previous test runs BEFORE creating the environment
    // This prevents "environment already exists" errors from stale state
    // We do this before CreateCommandHandler because it checks if environment exists in repository
    cleanup_previous_test_data("e2e-full").map_err(|e| anyhow::anyhow!("{e}"))?;

    // Create repository factory and clock for environment creation
    let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);

    // Create environment via CreateCommandHandler
    let environment = run_create_command(
        &repository_factory,
        clock,
        "e2e-full",
        ssh_private_key_path.to_string_lossy().to_string(),
        ssh_public_key_path.to_string_lossy().to_string(),
        "torrust",
        DEFAULT_SSH_PORT,
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Additional preflight cleanup for infrastructure (OpenTofu, LXD resources)
    // This handles any lingering infrastructure from interrupted previous runs
    // IMPORTANT: Must run BEFORE TestContext::init() which persists the environment
    let cleanup_context = TestContext::from_environment(
        cli.keep,
        environment.clone(),
        TestContextType::VirtualMachine,
    )?;
    
    preflight_cleanup_previous_resources(&cleanup_context)?;

    // Now initialize the test context and persist the environment after cleanup
    let mut test_context =
        TestContext::from_environment(cli.keep, environment, TestContextType::VirtualMachine)?
            .init()?;

    let test_start = Instant::now();

    let deployment_result = run_full_deployment_test(&mut test_context).await;

    let validation_result = match &deployment_result {
        Ok(()) => run_test_command(&test_context)
            .await
            .map_err(|e| anyhow::anyhow!("{e}")),
        Err(_) => Ok(()), // Skip validation if deployment failed
    };

    // Always cleanup test infrastructure created during this test run using DestroyCommandHandler
    // This ensures proper resource cleanup regardless of test success or failure
    // The keep_env flag is handled inside run_full_destroy_test
    let destroy_result = run_full_destroy_test(&mut test_context);

    let test_duration = test_start.elapsed();

    info!(
        performance = "test_execution",
        duration_secs = test_duration.as_secs_f64(),
        duration = ?test_duration,
        "Test execution completed"
    );

    // Handle all combinations of deployment, validation, and destroy results
    // Destroy failures are logged but don't override test results
    match destroy_result {
        Ok(()) => {
            info!(
                operation = "destroy",
                status = "success",
                "Infrastructure cleanup completed successfully"
            );
        }
        Err(destroy_err) => {
            error!(
                operation = "destroy",
                status = "failed",
                error = %destroy_err,
                "Infrastructure cleanup failed"
            );
            // Note: We don't fail the overall test just because cleanup failed
            // The test results are more important than cleanup results
        }
    }

    match (deployment_result, validation_result) {
        (Ok(()), Ok(())) => {
            info!(
                test_suite = "e2e_tests",
                status = "success",
                "All tests passed and cleanup completed successfully"
            );
            Ok(())
        }
        (Ok(()), Err(validation_err)) => {
            error!(
                test_suite = "e2e_tests",
                status = "failed",
                error = %validation_err,
                "Deployment succeeded but validation failed"
            );
            Err(validation_err)
        }
        (Err(deployment_err), Ok(())) => {
            error!(
                test_suite = "e2e_tests",
                status = "failed",
                error = %deployment_err,
                "Deployment failed"
            );
            Err(deployment_err)
        }
        (Err(deployment_err), Err(_)) => {
            error!(
                test_suite = "e2e_tests",
                status = "failed",
                error = %deployment_err,
                "Deployment failed (validation skipped)"
            );
            Err(deployment_err)
        }
    }
}

/// Verify that all required dependencies are installed for full E2E tests.
///
/// Full E2E tests require:
/// - `OpenTofu` (infrastructure provisioning)
/// - `Ansible` (configuration management)
/// - `LXD` (virtualization)
///
/// # Errors
///
/// Returns an error if any required dependencies are missing or cannot be detected.
fn verify_required_dependencies() -> Result<()> {
    let required_deps = &[Dependency::OpenTofu, Dependency::Ansible, Dependency::Lxd];

    if let Err(e) = verify_dependencies(required_deps) {
        error!(
            error = %e,
            "Dependency verification failed"
        );
        eprintln!("\n{}\n", e.actionable_message());
        return Err(anyhow::anyhow!("Missing required dependencies"));
    }

    Ok(())
}

async fn run_full_deployment_test(test_context: &mut TestContext) -> Result<()> {
    info!(
        test_type = "full_deployment",
        workflow = "template_based",
        "Starting full deployment E2E test"
    );

    // Provision infrastructure - updates TestContext with provisioned state
    run_provision_command(test_context)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Configure infrastructure - updates TestContext with configured state
    run_configure_command(test_context).map_err(|e| anyhow::anyhow!("{e}"))?;

    info!(status = "success", "Deployment completed successfully");

    info!(
        test_type = "full_deployment",
        status = "success",
        note = "Docker/Docker Compose installation status varies based on network connectivity",
        "Full deployment E2E test completed successfully"
    );

    Ok(())
}

fn run_full_destroy_test(test_context: &mut TestContext) -> Result<()> {
    info!(
        test_type = "full_destroy",
        workflow = "template_based",
        "Starting full destroy E2E test"
    );

    // Call the new run_destroy_command function
    run_destroy_command(test_context).map_err(|e| anyhow::anyhow!("{e}"))?;

    info!(
        status = "success",
        "Infrastructure destruction completed successfully"
    );
    Ok(())
}
