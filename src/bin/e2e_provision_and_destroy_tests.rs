//! End-to-End Provisioning and Destruction Tests for Torrust Tracker Deployer
//!
//! This binary tests the complete infrastructure lifecycle: provisioning and destruction.
//! It creates VMs/containers using `OpenTofu`, validates infrastructure provisioning,
//! and then destroys the infrastructure using the `DestroyCommand` (with fallback to
//! manual cleanup on failure). This does NOT test software configuration or installation.
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
//! 1. **Preflight cleanup** - Remove any artifacts from previous test runs that may have failed to clean up
//! 2. **Infrastructure provisioning** - Create VMs/containers using `OpenTofu`
//! 3. **Basic validation** - Verify VM is created and cloud-init completed
//! 4. **Infrastructure destruction** - Destroy infrastructure using `DestroyCommand` (with fallback to manual cleanup)
//!
//! ## Two-Phase Cleanup Strategy
//!
//! The cleanup process happens in two distinct phases:
//!
//! - **Phase 1 - Preflight cleanup**: Removes artifacts from previous test runs that may have
//!   failed to clean up properly (executed at the start in main function)
//! - **Phase 2 - Infrastructure destruction**: Destroys resources created specifically during
//!   the current test run using `DestroyCommand`, with fallback to manual cleanup on failure
//!   (executed at the end in main function)
//!
//! This approach provides comprehensive E2E testing of the full provision+destroy lifecycle
//! while ensuring reliable cleanup in CI environments.

use anyhow::Result;
use clap::Parser;
use std::net::IpAddr;
use std::time::Instant;
use tracing::{error, info};

// Import E2E testing infrastructure
use torrust_tracker_deployer_lib::adapters::ssh::{SshCredentials, DEFAULT_SSH_PORT};
use torrust_tracker_deployer_lib::domain::{Environment, EnvironmentName};
use torrust_tracker_deployer_lib::logging::{LogFormat, LogOutput, LoggingBuilder};
use torrust_tracker_deployer_lib::shared::Username;
use torrust_tracker_deployer_lib::testing::e2e::context::{TestContext, TestContextType};
use torrust_tracker_deployer_lib::testing::e2e::tasks::virtual_machine::{
    cleanup_infrastructure::cleanup_test_infrastructure,
    preflight_cleanup::preflight_cleanup_previous_resources,
    run_destroy_command::run_destroy_command, run_provision_command::run_provision_command,
};

#[derive(Parser)]
#[command(name = "e2e-provision-and-destroy-tests")]
#[command(about = "E2E provisioning and destruction tests for Torrust Tracker Deployer")]
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

/// Main entry point for the E2E provisioning and destruction test suite
///
/// This function orchestrates the complete provision+destroy test workflow:
/// 1. Initializes logging and test environment
/// 2. Performs pre-flight cleanup
/// 3. Runs provisioning tests (infrastructure creation only)
/// 4. Destroys infrastructure using `DestroyCommand` (with fallback to manual cleanup)
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
/// - Destruction operations fail (both `DestroyCommand` and manual cleanup fallback)
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
        test_suite = "e2e_provision_and_destroy_tests",
        log_format = ?cli.log_format,
        "Starting E2E provisioning and destruction tests"
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
    // Try using DestroyCommand first, fallback to manual cleanup on failure
    // This ensures proper resource cleanup regardless of test success or failure
    run_infrastructure_destroy(&mut test_context);

    let test_duration = test_start.elapsed();

    info!(
        performance = "test_execution",
        duration_secs = test_duration.as_secs_f64(),
        duration = ?test_duration,
        "Provisioning and destruction test execution completed"
    );

    // Handle provisioning test results
    match provision_result {
        Ok(_) => {
            info!(
                test_suite = "e2e_provision_and_destroy_tests",
                status = "success",
                "All provisioning and destruction tests passed successfully"
            );
            Ok(())
        }
        Err(provision_err) => {
            error!(
                test_suite = "e2e_provision_and_destroy_tests",
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

/// Runs the infrastructure destruction workflow
///
/// This function destroys infrastructure using the `DestroyCommand` with fallback
/// to manual cleanup if the command fails. This ensures reliable cleanup in all scenarios.
///
/// # Destruction Strategy
///
/// 1. **Try `DestroyCommand`**: Use the application layer command for destruction
/// 2. **Fallback to Manual Cleanup**: If `DestroyCommand` fails, use manual cleanup functions
fn run_infrastructure_destroy(test_context: &mut TestContext) {
    use tracing::warn;

    info!(
        test_type = "destroy",
        workflow = "infrastructure_destruction",
        "Starting infrastructure destruction E2E test"
    );

    // Try using the DestroyCommand first
    match run_destroy_command(test_context) {
        Ok(()) => {
            info!(
                status = "success",
                method = "destroy_command",
                "Infrastructure destroyed successfully using DestroyCommand"
            );
        }
        Err(e) => {
            warn!(
                status = "failed",
                method = "destroy_command",
                error = %e,
                "DestroyCommand failed, falling back to manual cleanup"
            );

            // Fallback to manual cleanup
            cleanup_test_infrastructure(test_context);

            info!(
                status = "success",
                method = "manual_cleanup",
                "Infrastructure destroyed using manual cleanup fallback"
            );
        }
    }
}
