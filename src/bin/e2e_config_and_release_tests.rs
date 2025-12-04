//! End-to-End Configuration and Release Testing Binary for Torrust Tracker Deployer (Black-box)
//!
//! This binary orchestrates configuration and release testing of the deployment infrastructure using
//! Docker containers instead of VMs. It uses a black-box approach, executing CLI commands
//! as external processes rather than importing internal application logic.
//!
//! ## Usage
//!
//! Run the E2E configuration and release tests:
//!
//! ```bash
//! cargo run --bin e2e-config-and-release-tests
//! ```
//!
//! Run with custom options:
//!
//! ```bash
//! # Change logging format
//! cargo run --bin e2e-config-and-release-tests -- --log-format json
//!
//! # Show help
//! cargo run --bin e2e-config-and-release-tests -- --help
//! ```
//!
//! ## Test Workflow
//!
//! This test uses the register command (instead of provision) since infrastructure
//! (Docker container) is created externally:
//!
//! 1. **Preflight cleanup** - Remove any artifacts from previous test runs
//! 2. **Container setup** - Build and start Docker container using `docker/provisioned-instance`
//! 3. **SSH verification** - Ensure container is ready for SSH connectivity
//! 4. **Create environment** - Create environment from config file (CLI: `create environment`)
//! 5. **Register instance** - Register the container's IP (CLI: `register --instance-ip`)
//! 6. **Configure** - Apply Ansible playbooks to configure services (CLI: `configure`)
//! 7. **Validation** - Verify deployments are working correctly
//! 8. **Stop container** - Stop the test container (deletion handled automatically by testcontainers)
//!
//! ## Black-box Testing Approach
//!
//! This test executes CLI commands as external processes, testing the full user-facing
//! interface. This is ideal for integration and acceptance testing.
//!
//! ## Container vs VM Management
//!
//! The container is managed outside the deployer workflow:
//! - **Container starting**: Happens before deployer commands (in test setup)
//! - **Register command**: Registers the container's IP as an existing instance
//! - **Container stopping**: Happens after tests (cleanup phase)
//!
//! This approach tests the `register` command path, which is designed for existing
//! infrastructure (VMs, physical servers, containers).

use std::net::SocketAddr;
use std::time::Instant;

use anyhow::{Context, Result};
use clap::Parser;
use torrust_dependency_installer::Dependency;
use tracing::{error, info};

use torrust_tracker_deployer_lib::adapters::ssh::{SshCredentials, DEFAULT_SSH_PORT};
use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput, LoggingBuilder};
use torrust_tracker_deployer_lib::shared::Username;
use torrust_tracker_deployer_lib::testing::e2e::containers::actions::{
    SshKeySetupAction, SshWaitAction,
};
use torrust_tracker_deployer_lib::testing::e2e::containers::timeout::ContainerTimeouts;
use torrust_tracker_deployer_lib::testing::e2e::containers::{
    RunningProvisionedContainer, StoppedProvisionedContainer,
};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::{
    generate_environment_config_with_port, run_container_preflight_cleanup,
    verify_required_dependencies, E2eTestRunner,
};
use torrust_tracker_deployer_lib::testing::e2e::tasks::container::cleanup_infrastructure::stop_test_infrastructure;
use torrust_tracker_deployer_lib::testing::e2e::tasks::run_configuration_validation::run_configuration_validation;

/// Environment name for this E2E test
const ENVIRONMENT_NAME: &str = "e2e-config";

#[derive(Parser)]
#[command(name = "e2e-config-and-release-tests")]
#[command(
    about = "E2E configuration and release tests using black-box approach with Docker containers"
)]
struct CliArgs {
    /// Logging format to use
    #[arg(
        long,
        default_value = "pretty",
        help = "Logging format: pretty, json, or compact"
    )]
    log_format: LogFormat,
}

/// Main entry point for E2E configuration tests (black-box approach).
///
/// Tests the register → configure workflow using Docker containers
/// instead of VMs to avoid network connectivity issues on GitHub Actions.
///
/// # Errors
///
/// Returns an error if:
/// - Docker container setup fails
/// - SSH connectivity cannot be established
/// - Any CLI command fails (create, register, configure)
/// - Configuration validation fails
///
/// # Panics
///
/// This function does not panic under normal operation. All error conditions
/// are handled through the `Result` return type.
#[tokio::main]
pub async fn main() -> Result<()> {
    let cli = CliArgs::parse();

    // SAFETY: These environment variables are set before any async runtime or
    // spawned threads are created, so no concurrent access to the environment
    // is possible at this point. This is the earliest point in main() after
    // argument parsing.
    unsafe {
        // Set environment variable to skip firewall configuration in container-based tests
        // UFW/iptables requires kernel capabilities not available in unprivileged containers
        std::env::set_var("TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER", "true");

        // Skip Docker installation in container-based tests since Docker is already
        // installed via the Dockerfile (Docker CE from get.docker.com script).
        // This avoids package conflicts between docker.io and containerd.io.
        std::env::set_var("TORRUST_TD_SKIP_DOCKER_INSTALL_IN_CONTAINER", "true");
    }

    // Note: Docker-in-Docker is now enabled via privileged mode in the container,
    // so we can test the run command that starts Docker Compose services.

    // Initialize logging with production log location for E2E tests using the builder pattern
    LoggingBuilder::new(std::path::Path::new("./data/logs"))
        .with_format(cli.log_format.clone())
        .with_output(LogOutput::FileAndStderr)
        .init();

    info!(
        application = "torrust_tracker_deployer",
        test_suite = "e2e_config_and_release_tests",
        log_format = ?cli.log_format,
        "Starting E2E configuration and release tests (black-box) with Docker containers"
    );

    // Verify required dependencies before running tests
    verify_required_dependencies(&[Dependency::Ansible])?;

    let test_start = Instant::now();

    // Cleanup any artifacts from previous test runs (including Docker containers)
    run_container_preflight_cleanup(ENVIRONMENT_NAME)?;

    let test_result = run_configuration_tests().await;

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
                test_suite = "e2e_config_and_release_tests",
                status = "success",
                "All configuration and release tests passed successfully"
            );
            Ok(())
        }
        Err(error) => {
            error!(
                test_suite = "e2e_config_and_release_tests",
                status = "failed",
                error = %error,
                "Configuration and release tests failed"
            );
            Err(error)
        }
    }
}

/// Run the complete configuration tests using black-box CLI commands
async fn run_configuration_tests() -> Result<()> {
    info!("Starting configuration tests with Docker container (black-box approach)");

    // Build SSH credentials (same as used in e2e_config_tests)
    let project_root = std::env::current_dir().expect("Failed to get current directory");
    let ssh_private_key_path = project_root.join("fixtures/testing_rsa");
    let ssh_public_key_path = project_root.join("fixtures/testing_rsa.pub");
    let ssh_user = Username::new("torrust").expect("Valid hardcoded username");
    let ssh_credentials = SshCredentials::new(ssh_private_key_path, ssh_public_key_path, ssh_user);

    // Step 1: Start Docker container (infrastructure managed externally)
    let running_container =
        create_and_start_container(ENVIRONMENT_NAME.to_string(), DEFAULT_SSH_PORT).await?;

    let socket_addr = running_container.ssh_socket_addr();

    // Step 2: Establish SSH connectivity
    establish_ssh_connectivity(socket_addr, &ssh_credentials, Some(&running_container)).await?;

    // Step 3: Run deployer commands (black-box via CLI)
    let test_result =
        run_deployer_workflow(socket_addr, &ssh_credentials, &running_container).await;

    // Step 4: Stop container regardless of test result
    stop_test_infrastructure(running_container);

    test_result
}

/// Run the deployer workflow using CLI commands (black-box approach)
///
/// This executes the create → register → configure workflow via CLI commands,
/// followed by validation.
async fn run_deployer_workflow(
    socket_addr: SocketAddr,
    ssh_credentials: &SshCredentials,
    _running_container: &RunningProvisionedContainer,
) -> Result<()> {
    let test_runner = E2eTestRunner::new(ENVIRONMENT_NAME);

    // Generate environment configuration file with the container's mapped SSH port
    // The port must be specified because the container exposes SSH on a dynamic port
    let config_path =
        generate_environment_config_with_port(ENVIRONMENT_NAME, Some(socket_addr.port()))?;

    // Create environment (CLI: cargo run -- create environment --env-file <file>)
    test_runner.create_environment(&config_path)?;

    // Register the container's IP as an existing instance
    // (CLI: cargo run -- register <env> --instance-ip <ip>)
    let instance_ip = socket_addr.ip().to_string();
    test_runner.register_instance(&instance_ip)?;

    // Configure services via Ansible
    // (CLI: cargo run -- configure <env>)
    test_runner.configure_services()?;

    // Release software to the configured infrastructure
    // (CLI: cargo run -- release <env>)
    test_runner.release_software()?;

    // Run services on the released infrastructure
    // (CLI: cargo run -- run <env>)
    test_runner.run_services()?;

    // Validate the configuration
    run_configuration_validation(socket_addr, ssh_credentials)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    info!("Configuration tests completed successfully");

    Ok(())
}

/// Create and start a Docker container for E2E testing
///
/// This function creates a new Docker container from the provisioned instance image
/// and starts it, making it ready for SSH connectivity and configuration testing.
async fn create_and_start_container(
    container_name: String,
    ssh_port: u16,
) -> Result<RunningProvisionedContainer> {
    info!(
        container_name = %container_name,
        ssh_port = %ssh_port,
        "Creating and starting Docker container for E2E testing"
    );

    let stopped_container = StoppedProvisionedContainer::default();

    let running_container = stopped_container
        .start(Some(container_name.clone()), ssh_port)
        .await
        .context("Failed to start provisioned instance container")?;

    info!(
        container_name = %container_name,
        container_id = %running_container.container_id(),
        ssh_socket_addr = %running_container.ssh_socket_addr(),
        "Docker container setup completed successfully"
    );

    Ok(running_container)
}

/// Establish SSH connectivity for a running Docker container
///
/// This function handles the complete SSH connectivity establishment process:
/// 1. Waits for SSH server to become available on the container
/// 2. Sets up SSH key authentication for container access
async fn establish_ssh_connectivity(
    socket_addr: SocketAddr,
    ssh_credentials: &SshCredentials,
    container: Option<&RunningProvisionedContainer>,
) -> Result<()> {
    info!(
        socket_addr = %socket_addr,
        ssh_user = %ssh_credentials.ssh_username,
        "Establishing SSH connectivity"
    );

    // Wait for SSH server to become available
    let timeouts = ContainerTimeouts::default();
    let ssh_wait_action = SshWaitAction::new(timeouts.ssh_ready, 10);
    ssh_wait_action
        .execute(socket_addr)
        .context("SSH server failed to start")?;

    // Setup SSH key authentication
    if let Some(running_container) = container {
        let ssh_key_setup_action = SshKeySetupAction::new();
        ssh_key_setup_action
            .execute(running_container, ssh_credentials)
            .await
            .context("Failed to setup SSH authentication")?;
    }

    info!(
        socket_addr = %socket_addr,
        ssh_user = %ssh_credentials.ssh_username,
        "SSH connectivity established successfully"
    );

    Ok(())
}
