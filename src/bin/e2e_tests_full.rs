//! Full End-to-End Testing Binary for Torrust Tracker Deploy (LOCAL DEVELOPMENT ONLY)
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
//! 1. **Preflight cleanup** - Remove any lingering test resources
//! 2. **Infrastructure provisioning** - Create LXD VMs using `OpenTofu`
//! 3. **Configuration** - Apply Ansible playbooks for software installation
//! 4. **Validation** - Verify deployments are working correctly
//! 5. **Cleanup** - Remove test resources
//!
//! The test suite supports different VM providers (LXD, Multipass) and includes
//! comprehensive logging and error reporting.

use anyhow::Result;
use clap::Parser;
use std::net::IpAddr;
use std::time::Instant;
use tracing::{error, info};

// Import E2E testing infrastructure
use torrust_tracker_deploy::domain::{Environment, EnvironmentName};
use torrust_tracker_deploy::e2e::context::{TestContext, TestContextType};
use torrust_tracker_deploy::e2e::tasks::{
    run_configure_command::run_configure_command,
    run_test_command::run_test_command,
    virtual_machine::{
        cleanup_infrastructure::cleanup_infrastructure,
        preflight_cleanup::cleanup_lingering_resources,
        run_provision_command::run_provision_command,
    },
};
use torrust_tracker_deploy::logging::{self, LogFormat};
use torrust_tracker_deploy::shared::Username;

#[derive(Parser)]
#[command(name = "e2e-tests")]
#[command(about = "E2E tests for Torrust Tracker Deploy")]
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
#[tokio::main]
pub async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging based on the chosen format
    logging::init_with_format(&cli.log_format);

    info!(
        application = "torrust_tracker_deploy",
        test_suite = "e2e_tests",
        log_format = ?cli.log_format,
        "Starting E2E tests"
    );

    // Create environment entity for e2e-full testing
    let environment_name = EnvironmentName::new("e2e-full".to_string())?;

    // Use absolute paths to project root for SSH keys to ensure they can be found by Ansible
    let project_root = std::env::current_dir().expect("Failed to get current directory");
    let ssh_private_key_path = project_root.join("fixtures/testing_rsa");
    let ssh_public_key_path = project_root.join("fixtures/testing_rsa.pub");
    let ssh_user = Username::new("torrust").expect("Valid hardcoded username");

    let environment = Environment::new(
        environment_name,
        ssh_user.clone(),
        ssh_private_key_path.clone(),
        ssh_public_key_path.clone(),
    );

    let test_context =
        TestContext::from_environment(cli.keep, environment, TestContextType::VirtualMachine)?
            .init()?;

    // Perform pre-flight cleanup to remove any lingering resources from interrupted tests
    cleanup_lingering_resources(&test_context)?;

    let test_start = Instant::now();

    let deployment_result = run_full_deployment_test(&test_context).await;

    let validation_result = match &deployment_result {
        Ok(instance_ip) => run_test_command(&test_context, instance_ip).await,
        Err(_) => Ok(()), // Skip validation if deployment failed
    };

    cleanup_infrastructure(&test_context);

    let test_duration = test_start.elapsed();

    info!(
        performance = "test_execution",
        duration_secs = test_duration.as_secs_f64(),
        duration = ?test_duration,
        "Test execution completed"
    );

    // Handle all 4 combinations of deployment and validation results
    match (deployment_result, validation_result) {
        (Ok(_), Ok(())) => {
            info!(
                test_suite = "e2e_tests",
                status = "success",
                "All tests passed and cleanup completed successfully"
            );
            Ok(())
        }
        (Ok(_), Err(validation_err)) => {
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

async fn run_full_deployment_test(env: &TestContext) -> Result<IpAddr> {
    info!(
        test_type = "full_deployment",
        workflow = "template_based",
        "Starting full deployment E2E test"
    );

    let instance_ip = run_provision_command(env).await?;

    run_configure_command(env)?;

    info!(status = "success", "Deployment completed successfully");

    info!(
        test_type = "full_deployment",
        status = "success",
        note = "Docker/Docker Compose installation status varies based on network connectivity",
        "Full deployment E2E test completed successfully"
    );

    // Return the instance IP for validation in main
    Ok(instance_ip)
}
