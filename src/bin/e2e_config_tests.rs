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

use anyhow::{Context, Result};
use clap::Parser;
use std::time::Instant;
use tracing::{error, info};

use torrust_tracker_deploy::config::InstanceName;
use torrust_tracker_deploy::e2e::environment::{TestEnvironment, TestEnvironmentType};
use torrust_tracker_deploy::e2e::tasks::{
    container::{
        cleanup_docker_container::cleanup_docker_container,
        configure_ssh_connectivity::configure_ssh_connectivity,
        run_provision_simulation::run_provision_simulation,
        setup_docker_container::setup_docker_container,
    },
    preflight_cleanup,
    run_ansible_configuration::run_ansible_configuration,
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

    // Initialize logging based on the chosen format
    logging::init_with_format(&cli.log_format);

    info!(
        application = "torrust_tracker_deploy",
        test_suite = "e2e_config_tests",
        log_format = ?cli.log_format,
        "Starting E2E configuration tests with Docker containers"
    );

    let test_start = Instant::now();

    // Instance name for the test environment - consistent with provision tests
    let instance_name =
        InstanceName::new("torrust-tracker-vm".to_string()).expect("Valid hardcoded instance name");

    // Setup test environment with preflight cleanup
    let test_env = setup_test_environment(false, cli.templates_dir, instance_name)?;

    let test_result = run_configuration_tests(&test_env).await;

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

/// Setup test environment with preflight cleanup
fn setup_test_environment(
    keep_env: bool,
    templates_dir: String,
    instance_name: InstanceName,
) -> Result<TestEnvironment> {
    info!("Running preflight cleanup for Docker-based E2E tests");
    let test_env = TestEnvironment::with_ssh_user_and_init(
        keep_env,
        templates_dir,
        "torrust",
        instance_name,
        TestEnvironmentType::Container,
    )
    .context("Failed to create test environment")?;

    preflight_cleanup::cleanup_lingering_resources_docker(&test_env)
        .context("Failed to complete preflight cleanup")?;

    Ok(test_env)
}

/// Run the complete configuration tests using extracted tasks
async fn run_configuration_tests(test_env: &TestEnvironment) -> Result<()> {
    info!("Starting configuration tests with Docker container");

    // Step 1: Setup Docker container
    let running_container = setup_docker_container().await?;
    let socket_addr = running_container.ssh_socket_addr();

    // Step 2: Configure SSH connectivity
    configure_ssh_connectivity(
        socket_addr,
        &test_env.config.ssh_credentials,
        Some(&running_container),
    )
    .await?;

    // Step 3: Run provision simulation
    run_provision_simulation(socket_addr, &test_env.config.ssh_credentials, test_env).await?;

    // Step 4: Run Ansible configuration (expect failure due to inventory mismatch)
    run_ansible_configuration(socket_addr, test_env, false)?;

    // Step 5: Run deployment validation
    run_deployment_validation(socket_addr, &test_env.config.ssh_credentials).await?;

    // Step 6: Cleanup container
    cleanup_docker_container(running_container);

    info!("Configuration tests completed successfully");
    Ok(())
}
