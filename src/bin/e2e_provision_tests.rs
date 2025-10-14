//! End-to-End Provisioning Tests for Torrust Tracker Deployer
//!
//! This binary focuses exclusively on testing infrastructure provisioning.
//! It creates VMs/containers using `OpenTofu` and validates that the infrastructure
//! is properly provisioned and ready for configuration, but does NOT attempt
//! to configure or install software.
//!
//! ## Usage
//!
//! Run the E2E provisioning tests:
//!
//! ```bash
//! cargo run --bin e2e-provision-tests
//! ```
//!
//! Run with custom options:
//!
//! ```bash
//! # Keep test environment after completion (for debugging)
//! cargo run --bin e2e-provision-tests -- --keep
//!
//! # Change logging format
//! cargo run --bin e2e-provision-tests -- --log-format json
//!
//! # Show help
//! cargo run --bin e2e-provision-tests -- --help
//! ```
//!
//! ## Test Workflow
//!
//! 1. **Preflight cleanup** - Remove any artifacts from previous test runs that may have failed to clean up
//! 2. **Infrastructure provisioning** - Create VMs/containers using `OpenTofu`
//! 3. **Basic validation** - Verify VM is created and cloud-init completed
//! 4. **Test infrastructure cleanup** - Remove test resources created during this run
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
//! This split allows provisioning tests to run reliably on GitHub Actions
//! while configuration tests can be handled separately with different infrastructure.

use anyhow::Result;
use clap::Parser;
use std::net::IpAddr;
use std::time::Instant;
use tracing::{error, info};

// Import E2E testing infrastructure
use torrust_tracker_deployer_lib::domain::{Environment, EnvironmentName};
use torrust_tracker_deployer_lib::logging::{LogFormat, LogOutput, LoggingBuilder};
use torrust_tracker_deployer_lib::shared::{
    ssh::{SshCredentials, DEFAULT_SSH_PORT},
    Username,
};
use torrust_tracker_deployer_lib::testing::e2e::context::{TestContext, TestContextType};
use torrust_tracker_deployer_lib::testing::e2e::tasks::virtual_machine::{
    cleanup_infrastructure::cleanup_test_infrastructure,
    preflight_cleanup::preflight_cleanup_previous_resources,
    run_provision_command::run_provision_command,
};

#[derive(Parser)]
#[command(name = "e2e-provision-tests")]
#[command(about = "E2E provisioning tests for Torrust Tracker Deployer")]
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

/// Main entry point for the E2E provisioning test suite
///
/// This function orchestrates the complete provisioning test workflow:
/// 1. Initializes logging and test environment
/// 2. Performs pre-flight cleanup
/// 3. Runs provisioning tests (infrastructure creation only)
/// 4. Performs cleanup
/// 5. Reports results
///
/// Returns `Ok(())` if all tests pass, `Err` otherwise.
///
/// # Errors
///
/// This function may return errors in the following cases:
/// - Invalid environment name provided via CLI
/// - Test environment setup fails
/// - Pre-flight cleanup encounters issues
/// - Infrastructure provisioning fails
/// - Cleanup operations fail
///
/// # Panics
///
/// This function may panic if the hardcoded username "torrust" is invalid,
/// which should never happen in normal operation.
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
        test_suite = "e2e_provision_tests",
        log_format = ?cli.log_format,
        "Starting E2E provisioning tests"
    );

    // Create environment entity for e2e-provision testing
    let environment_name = EnvironmentName::new("e2e-provision".to_string())?;

    // Use absolute paths to project root for SSH keys to ensure they can be found by Ansible
    let project_root = std::env::current_dir().expect("Failed to get current directory");
    let ssh_private_key_path = project_root.join("fixtures/testing_rsa");
    let ssh_public_key_path = project_root.join("fixtures/testing_rsa.pub");
    let ssh_user = Username::new("torrust").expect("Valid hardcoded username");
    let ssh_credentials = SshCredentials::new(
        ssh_private_key_path.clone(),
        ssh_public_key_path.clone(),
        ssh_user.clone(),
    );

    let ssh_port = DEFAULT_SSH_PORT;
    let environment = Environment::new(environment_name, ssh_credentials, ssh_port);

    let mut test_context =
        TestContext::from_environment(cli.keep, environment, TestContextType::VirtualMachine)?
            .init()?;

    // Cleanup any artifacts from previous test runs that may have failed to clean up
    // This ensures a clean slate before starting new tests
    preflight_cleanup_previous_resources(&test_context)?;

    let test_start = Instant::now();

    let provision_result = run_provisioning_test(&mut test_context).await;

    // Always cleanup test infrastructure created during this test run
    // This ensures proper resource cleanup regardless of test success or failure
    cleanup_test_infrastructure(&test_context);

    let test_duration = test_start.elapsed();

    info!(
        performance = "test_execution",
        duration_secs = test_duration.as_secs_f64(),
        duration = ?test_duration,
        "Provisioning test execution completed"
    );

    // Handle provisioning test results
    match provision_result {
        Ok(_) => {
            info!(
                test_suite = "e2e_provision_tests",
                status = "success",
                "All provisioning tests passed and cleanup completed successfully"
            );
            Ok(())
        }
        Err(provision_err) => {
            error!(
                test_suite = "e2e_provision_tests",
                status = "failed",
                error = %provision_err,
                "Infrastructure provisioning failed"
            );
            Err(provision_err)
        }
    }
}

/// Runs the provisioning test workflow
///
/// This function focuses exclusively on infrastructure provisioning and validation.
/// It does NOT attempt to configure software or install applications.
///
/// # Test Phases
///
/// 1. **Provision Infrastructure**: Creates VMs/containers using `OpenTofu`
/// 2. **Basic Validation**: Verifies infrastructure is ready (cloud-init completed)
///
/// Returns the provisioned instance IP address on success.
async fn run_provisioning_test(env: &mut TestContext) -> Result<IpAddr> {
    info!(
        test_type = "provision_only",
        workflow = "infrastructure_provisioning",
        "Starting infrastructure provisioning E2E test"
    );

    run_provision_command(env)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Extract instance IP from the updated TestContext
    let instance_ip = env
        .environment
        .instance_ip()
        .expect("Instance IP must be set after successful provisioning");

    info!(
        status = "success",
        instance_ip = %instance_ip,
        "Infrastructure provisioning completed successfully"
    );

    info!(
        test_type = "provision_only",
        status = "success",
        note = "VM/container created and cloud-init completed - ready for configuration",
        "Provisioning E2E test completed successfully"
    );

    // Return the instance IP for potential future validation
    Ok(instance_ip)
}
