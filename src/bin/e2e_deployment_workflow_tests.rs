//! End-to-End Deployment Workflow Testing Binary for Torrust Tracker Deployer (Black-box)
//!
//! This binary orchestrates configuration and release testing of the deployment infrastructure using
//! Docker containers instead of VMs. It uses a black-box approach, executing CLI commands
//! as external processes rather than importing internal application logic.
//!
//! ## Usage
//!
//! Run the E2E deployment workflow tests:
//!
//! ```bash
//! cargo run --bin e2e-deployment-workflow-tests
//! ```
//!
//! Run with custom options:
//!
//! ```bash
//! # Change logging format
//! cargo run --bin e2e-deployment-workflow-tests -- --log-format json
//!
//! # Show help
//! cargo run --bin e2e-deployment-workflow-tests -- --help
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

use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
use torrust_tracker_deployer_lib::bootstrap::logging::{LogFormat, LogOutput, LoggingBuilder};
use torrust_tracker_deployer_lib::shared::Username;
use torrust_tracker_deployer_lib::testing::e2e::containers::actions::{
    SshKeySetupAction, SshWaitAction,
};
use torrust_tracker_deployer_lib::testing::e2e::containers::timeout::ContainerTimeouts;
use torrust_tracker_deployer_lib::testing::e2e::containers::tracker_ports::{
    ContainerPorts, E2eConfigEnvironment, E2eRuntimeEnvironment,
};
use torrust_tracker_deployer_lib::testing::e2e::containers::{
    RunningProvisionedContainer, StoppedProvisionedContainer,
};
use torrust_tracker_deployer_lib::testing::e2e::tasks::black_box::{
    build_e2e_test_config, run_container_preflight_cleanup, verify_required_dependencies,
    write_environment_config, E2eTestRunner,
};
use torrust_tracker_deployer_lib::testing::e2e::tasks::container::cleanup_infrastructure::stop_test_infrastructure;
use torrust_tracker_deployer_lib::testing::e2e::tasks::run_configuration_validation::run_configuration_validation;
use torrust_tracker_deployer_lib::testing::e2e::tasks::run_release_validation::{
    run_release_validation, ServiceValidation,
};
use torrust_tracker_deployer_lib::testing::e2e::tasks::run_run_validation::{
    run_run_validation, ServiceValidation as RunServiceValidation,
};

/// Environment name for this E2E test
const ENVIRONMENT_NAME: &str = "e2e-deployment";

#[derive(Parser)]
#[command(name = "e2e-deployment-workflow-tests")]
#[command(about = "E2E deployment workflow tests using black-box approach with Docker containers")]
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
    LoggingBuilder::new(std::path::Path::new("./data/e2e-deployment/logs"))
        .with_format(cli.log_format.clone())
        .with_output(LogOutput::FileAndStderr)
        .init();

    info!(
        application = "torrust_tracker_deployer",
        test_suite = "e2e_deployment_workflow_tests",
        log_format = ?cli.log_format,
        "Starting E2E configuration and release tests (black-box) with Docker containers"
    );

    // Verify required dependencies before running tests
    verify_required_dependencies(&[Dependency::Ansible])?;

    let test_start = Instant::now();

    // Cleanup any artifacts from previous test runs (including Docker containers)
    run_container_preflight_cleanup(ENVIRONMENT_NAME)?;

    let test_result = run_configure_release_run_tests().await;

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
                test_suite = "e2e_deployment_workflow_tests",
                status = "success",
                "All configuration and release tests passed successfully"
            );
            Ok(())
        }
        Err(error) => {
            error!(
                test_suite = "e2e_deployment_workflow_tests",
                status = "failed",
                error = %error,
                "Configuration and release tests failed"
            );
            Err(error)
        }
    }
}

/// Run the complete configure → release → run workflow tests using black-box CLI commands
///
/// This function orchestrates the full software deployment workflow:
/// 1. Generate environment configuration with all E2E info (name, ports, config path)
/// 2. Create Docker container with ports from environment config (host networking)
/// 3. Establish SSH connectivity
/// 4. Register the container's IP as an existing instance
/// 5. Configure services via Ansible (install Docker, etc.)
/// 6. Release software (deploy Docker Compose files)
/// 7. Run services (start Docker Compose)
///
/// With host networking, ports are identical inside and outside the container,
/// eliminating the cyclic dependency between config generation and container creation.
///
/// Each step is followed by validation to ensure correctness.
async fn run_configure_release_run_tests() -> Result<()> {
    info!("Starting configure → release → run tests with Docker container (black-box approach)");

    // Build SSH credentials
    let ssh_credentials = build_test_ssh_credentials();

    // Step 1: Build E2E test configuration in-memory
    // This creates the configuration structure without file I/O
    let config_env = build_e2e_test_config(ENVIRONMENT_NAME);

    // Step 2: Create and start Docker container
    // With bridge networking, Docker assigns random mapped ports
    // Returns runtime environment with both config and actual mapped ports
    let (runtime_env, running_container) = create_and_start_container(&config_env).await?;

    // Get SSH socket address from runtime environment (using actual mapped port)
    let ssh_socket_address = runtime_env.ssh_socket_addr();

    // Step 3: Establish SSH connectivity using the mapped SSH port
    establish_ssh_connectivity(
        ssh_socket_address,
        &ssh_credentials,
        Some(&running_container),
    )
    .await?;

    // Step 4: Run deployer commands (black-box via CLI)
    let test_result = run_deployer_workflow(&config_env, &runtime_env, &ssh_credentials).await;

    // Step 5: Stop container regardless of test result
    stop_test_infrastructure(running_container);

    test_result
}

/// Run the deployer workflow using CLI commands (black-box approach)
///
/// This executes the create → register → configure → release → run workflow
/// via CLI commands, with validation after each major step.
///
/// # Arguments
/// * `config_env` - Configuration environment with desired ports and settings
/// * `runtime_env` - Runtime environment with actual mapped ports from Docker
/// * `ssh_credentials` - SSH credentials for container access
async fn run_deployer_workflow(
    config_env: &E2eConfigEnvironment,
    runtime_env: &E2eRuntimeEnvironment,
    ssh_credentials: &SshCredentials,
) -> Result<()> {
    let test_runner = E2eTestRunner::new(ENVIRONMENT_NAME);

    // Write environment configuration to disk (needed by create command)
    write_environment_config(config_env)?;

    // Create environment (CLI: cargo run -- create environment --env-file <file>)
    test_runner.create_environment(&config_env.config_file_path)?;

    // Register the container's IP as an existing instance with custom SSH port
    // (CLI: cargo run -- register <env> --instance-ip <ip> --ssh-port <port>)
    // With bridge networking, we pass the actual mapped SSH port from Docker
    let socket_addr = runtime_env.ssh_socket_addr();
    let instance_ip = socket_addr.ip().to_string();
    let ssh_port = runtime_env.container_ports.ssh_port;
    test_runner.register_instance(&instance_ip, Some(ssh_port))?;

    // Configure services via Ansible
    // (CLI: cargo run -- configure <env>)
    test_runner.configure_services()?;

    // Validate the configuration (Docker and Docker Compose installed correctly)
    run_configuration_validation(socket_addr, ssh_credentials)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Release software to the configured infrastructure
    // (CLI: cargo run -- release <env>)
    test_runner.release_software()?;

    // Validate the release (Docker Compose files deployed correctly)
    // Note: E2E deployment environment has Prometheus and Grafana enabled
    let services = ServiceValidation {
        prometheus: true,
        grafana: true,
    };
    run_release_validation(socket_addr, ssh_credentials, Some(services))
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    // Run services on the released infrastructure
    // (CLI: cargo run -- run <env>)
    test_runner.run_services()?;

    // Validate services are running using actual mapped ports from runtime environment
    // Note: E2E deployment environment has Prometheus and Grafana enabled
    let run_services = RunServiceValidation {
        prometheus: true,
        grafana: true,
    };
    run_run_validation(
        socket_addr,
        ssh_credentials,
        runtime_env.container_ports.http_api_port,
        vec![runtime_env.container_ports.http_tracker_port],
        Some(run_services),
    )
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;

    info!("Configure → release → run workflow tests completed successfully");

    Ok(())
}

/// Build SSH credentials for E2E testing
///
/// Creates SSH credentials using the test fixtures located in the `fixtures/` directory.
/// These credentials are used to establish SSH connectivity with the test container.
///
/// # Returns
///
/// Returns `SshCredentials` configured with:
/// - Private key: `fixtures/testing_rsa`
/// - Public key: `fixtures/testing_rsa.pub`
/// - Username: `torrust`
fn build_test_ssh_credentials() -> SshCredentials {
    let project_root = std::env::current_dir().expect("Failed to get current directory");
    let ssh_private_key_path = project_root.join("fixtures/testing_rsa");
    let ssh_public_key_path = project_root.join("fixtures/testing_rsa.pub");
    let ssh_user = Username::new("torrust").expect("Valid hardcoded username");
    SshCredentials::new(ssh_private_key_path, ssh_public_key_path, ssh_user)
}

/// Create and start a Docker container for E2E testing
///
/// This function creates a new Docker container from the provisioned instance image
/// and starts it, making it ready for SSH connectivity and configuration testing.
///
/// With bridge networking (default Docker mode), ports are dynamically mapped.
/// The function returns both the configuration (desired ports) and runtime
/// (actual mapped ports) in an `E2eRuntimeEnvironment`.
///
/// # Arguments
/// * `config_env` - E2E configuration with desired ports and environment settings
///
/// # Returns
/// * `(E2eRuntimeEnvironment, RunningProvisionedContainer)` - Runtime environment and container reference
async fn create_and_start_container(
    config_env: &E2eConfigEnvironment,
) -> Result<(E2eRuntimeEnvironment, RunningProvisionedContainer)> {
    let additional_ports = config_env.tracker_ports.all_ports();

    info!(
        environment_name = %config_env.environment_name,
        ssh_port = %config_env.ssh_port,
        http_api_port = config_env.tracker_ports.http_api_port,
        http_tracker_port = config_env.tracker_ports.http_tracker_port,
        udp_tracker_port = config_env.tracker_ports.udp_tracker_port,
        "Creating and starting Docker container for E2E testing with tracker ports from config"
    );

    let stopped_container = StoppedProvisionedContainer::default();

    let running_container = stopped_container
        .start(
            Some(config_env.environment_name.clone()),
            config_env.ssh_port,
            &additional_ports,
        )
        .await
        .context("Failed to start provisioned instance container")?;

    // Get the actual mapped ports from Docker
    let ssh_mapped_port = running_container.ssh_socket_addr().port();
    let additional_mapped_ports = running_container.additional_mapped_ports();

    // Build runtime environment with both config and actual mapped ports
    let container_ports =
        ContainerPorts::from_mapped_ports(ssh_mapped_port, additional_mapped_ports);
    let runtime_env = E2eRuntimeEnvironment::new(config_env.clone(), container_ports);

    info!(
        environment_name = %config_env.environment_name,
        container_id = %running_container.container_id(),
        ssh_socket_addr = %running_container.ssh_socket_addr(),
        "Docker container setup completed successfully"
    );

    Ok((runtime_env, running_container))
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
