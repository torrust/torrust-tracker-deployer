//! End-to-End Provisioning Tests for Torrust Tracker Deploy
//!
//! This binary focuses exclusively on testing infrastructure provisioning.
//! It creates VMs/containers using `OpenTofu` and validates that the infrastructure
//! is properly provisioned and ready for configuration, but does NOT attempt
//! to configure or install software.
//!
//! ## Test Workflow
//!
//! 1. **Preflight cleanup** - Remove any lingering test resources
//! 2. **Infrastructure provisioning** - Create VMs/containers using `OpenTofu`
//! 3. **Basic validation** - Verify VM is created and cloud-init completed
//! 4. **Cleanup** - Remove test resources
//!
//! This split allows provisioning tests to run reliably on GitHub Actions
//! while configuration tests can be handled separately with different infrastructure.

use anyhow::Result;
use clap::Parser;
use std::net::IpAddr;
use std::time::Instant;
use tracing::{error, info};

// Import E2E testing infrastructure
use torrust_tracker_deploy::config::InstanceName;
use torrust_tracker_deploy::domain::Username;
use torrust_tracker_deploy::e2e::environment::{TestEnvironment, TestEnvironmentType};
use torrust_tracker_deploy::e2e::tasks::{
    preflight_cleanup::cleanup_lingering_resources,
    virtual_machine::{
        cleanup_infrastructure::cleanup_infrastructure,
        run_provision_command::run_provision_command,
    },
};
use torrust_tracker_deploy::logging::{self, LogFormat};

#[derive(Parser)]
#[command(name = "e2e-provision-tests")]
#[command(about = "E2E provisioning tests for Torrust Tracker Deploy")]
struct Cli {
    /// Keep the test environment after completion
    #[arg(long)]
    keep: bool,

    /// Templates directory path (default: ./data/templates)
    #[arg(long, default_value = "./data/templates")]
    templates_dir: String,

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
/// - Test environment setup fails
/// - Pre-flight cleanup encounters issues
/// - Infrastructure provisioning fails
/// - Cleanup operations fail
///
/// # Panics
///
/// May panic if the hardcoded instance name "torrust-tracker-vm" is invalid,
/// which should never happen in normal operation.
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
        test_suite = "e2e_provision_tests",
        log_format = ?cli.log_format,
        "Starting E2E provisioning tests"
    );

    // Instance name for the test environment - not user configurable for now
    let instance_name =
        InstanceName::new("torrust-tracker-vm".to_string()).expect("Valid hardcoded instance name");

    let ssh_user = Username::new("torrust").expect("Valid hardcoded username");

    let env = TestEnvironment::initialized(
        cli.keep,
        cli.templates_dir,
        &ssh_user,
        instance_name,
        TestEnvironmentType::VirtualMachine,
    )?;

    // Perform pre-flight cleanup to remove any lingering resources from interrupted tests
    cleanup_lingering_resources(&env)?;

    let test_start = Instant::now();

    let provision_result = run_provisioning_test(&env).await;

    cleanup_infrastructure(&env);

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
async fn run_provisioning_test(env: &TestEnvironment) -> Result<IpAddr> {
    info!(
        test_type = "provision_only",
        workflow = "infrastructure_provisioning",
        "Starting infrastructure provisioning E2E test"
    );

    let instance_ip = run_provision_command(env).await?;

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
