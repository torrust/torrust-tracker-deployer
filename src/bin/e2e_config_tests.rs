//! End-to-End Configuration Testing Binary for Torrust Tracker Deploy
//!
//! This binary orchestrates configuration, release, and run phase testing of the deployment
//! infrastructure using Docker containers instead of VMs. It replaces infrastructure
//! provisioning with Docker container setup to test Ansible configuration in a controlled
//! environment.
//!
//! ## Usage
//!
//! Run the E2E configuration tests:
//!
//! ```bash
//! cargo run --bin e2e-config-tests
//! ```
//!
//! Run with custom options:
//!
//! ```bash
//! # Use custom templates directory
//! cargo run --bin e2e-config-tests -- --templates-dir ./custom/templates
//!
//! # Change logging format
//! cargo run --bin e2e-config-tests -- --log-format json
//!
//! # Show help
//! cargo run --bin e2e-config-tests -- --help
//! ```
//!
//! ## Test Workflow
//!
//! 1. **Container setup** - Build and start Docker container using `docker/provisioned-instance`
//! 2. **SSH verification** - Ensure container is ready for Ansible connectivity
//! 3. **Configuration** - Apply Ansible playbooks to configure services
//! 4. **Validation** - Verify deployments are working correctly  
//! 5. **Cleanup** - Stop and remove test containers
//!
//! This approach addresses network connectivity issues with LXD VMs on GitHub Actions
//! while maintaining comprehensive testing of the configuration and deployment phases.
//!
//! ## State Machine Pattern
//!
//! The container follows a state machine pattern similar to the Torrust Tracker `MySQL` driver:
//! - `StoppedProvisionedContainer` - Initial state, can only be started
//! - `RunningProvisionedContainer` - Running state, can be queried, configured, and stopped
//! - State transitions are enforced at compile time through different types

use anyhow::Result;
use clap::Parser;
use std::time::Instant;
use torrust_tracker_deploy::e2e::tasks::run_configure_command::run_configure_command;
use tracing::{error, info};

use torrust_tracker_deploy::config::InstanceName;
use torrust_tracker_deploy::e2e::environment::{TestEnvironment, TestEnvironmentType};
use torrust_tracker_deploy::e2e::tasks::{
    container::{
        cleanup_infrastructure::cleanup_infrastructure,
        run_provision_simulation::run_provision_simulation,
    },
    preflight_cleanup,
    run_deployment_validation::run_deployment_validation,
};
use torrust_tracker_deploy::logging::{self, LogFormat};

#[derive(Parser)]
#[command(name = "e2e-config-tests")]
#[command(about = "E2E configuration tests for Torrust Tracker Deploy using Docker containers")]
struct CliArgs {
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

/// Main entry point for E2E configuration tests.
///
/// Tests the configuration, release, and run phases using Docker containers
/// instead of VMs to avoid network connectivity issues on GitHub Actions.
///
/// # Errors
///
/// Returns an error if:
/// - Docker container setup fails
/// - SSH connectivity cannot be established  
/// - Configuration tests fail
/// - Container cleanup fails (when enabled)
///
/// # Panics
///
/// This function does not panic under normal operation. All error conditions
/// are handled through the `Result` return type.
#[tokio::main]
pub async fn main() -> Result<()> {
    let cli = CliArgs::parse();

    logging::init_with_format(&cli.log_format);

    info!(
        application = "torrust_tracker_deploy",
        test_suite = "e2e_config_tests",
        log_format = ?cli.log_format,
        "Starting E2E configuration tests with Docker containers"
    );

    let test_start = Instant::now();

    let instance_name =
        InstanceName::new("torrust-tracker-vm".to_string()).expect("Valid hardcoded instance name");

    let env = TestEnvironment::with_ssh_user_and_init(
        false,
        cli.templates_dir,
        "torrust",
        instance_name,
        TestEnvironmentType::Container,
    )?;

    preflight_cleanup::cleanup_lingering_resources(&env)?;

    let test_result = run_configuration_tests(&env).await;

    let test_duration = test_start.elapsed();

    info!(
        performance = "test_execution",
        duration_secs = test_duration.as_secs_f64(),
        duration = ?test_duration,
        "Configuration test execution completed"
    );

    // Handle test results
    match test_result {
        Ok(()) => {
            info!(
                test_suite = "e2e_config_tests",
                status = "success",
                "All configuration tests passed successfully"
            );
            Ok(())
        }
        Err(error) => {
            error!(
                test_suite = "e2e_config_tests",
                status = "failed",
                error = %error,
                "Configuration tests failed"
            );
            Err(error)
        }
    }
}

/// Run the complete configuration tests using extracted tasks
async fn run_configuration_tests(test_env: &TestEnvironment) -> Result<()> {
    info!("Starting configuration tests with Docker container");

    // Step 1: Run provision simulation (includes container setup and SSH connectivity)
    let running_container = run_provision_simulation(test_env).await?;

    // Step 2: Run Ansible configuration
    run_configure_command(test_env)?;

    // Step 3: Run deployment validation
    let socket_addr = running_container.ssh_socket_addr();
    run_deployment_validation(socket_addr, &test_env.config.ssh_credentials).await?;

    // Step 4: Cleanup container
    cleanup_infrastructure(running_container);

    info!("Configuration tests completed successfully");

    Ok(())
}
