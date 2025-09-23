//! End-to-End Configuration Testing Binary for Torrust Tracker Deploy
//!
//! This binary orchestrates configuration, release, and run phase testing of the deployment
//! infrastructure using Docker containers instead of VMs. It replaces infrastructure
//! provisioning with Docker container setup to test Ansible configuration in a controlled
//! environment.
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
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;
use tracing::{error, info};

use torrust_tracker_deploy::application::commands::{ConfigureCommand, TestCommand};
use torrust_tracker_deploy::config::{Config, InstanceName, SshCredentials};
use torrust_tracker_deploy::container::Services;
use torrust_tracker_deploy::e2e::environment::TestEnvironment;
use torrust_tracker_deploy::e2e::provisioned_container::StoppedProvisionedContainer;
use torrust_tracker_deploy::e2e::tasks::preflight_cleanup;
use torrust_tracker_deploy::e2e::tasks::provision_docker_infrastructure::provision_docker_infrastructure;
use torrust_tracker_deploy::logging::{self, LogFormat};

#[derive(Parser)]
#[command(name = "e2e-config-tests")]
#[command(about = "E2E configuration tests for Torrust Tracker Deploy using Docker containers")]
struct CliArgs {
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
pub fn main() -> Result<()> {
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

    let test_result = run_configuration_tests();

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

/// Run the complete configuration tests
fn run_configuration_tests() -> Result<()> {
    info!("Starting configuration tests with Docker container");

    // Step 0: Preflight cleanup to ensure fresh state
    info!("Running preflight cleanup for Docker-based E2E tests");
    let instance_name = InstanceName::new("torrust-tracker-vm".to_string())
        .context("Failed to create instance name")?;
    let test_env = TestEnvironment::new(false, "./data/templates", instance_name)
        .context("Failed to create test environment")?;

    preflight_cleanup::cleanup_lingering_resources_docker(&test_env)
        .context("Failed to complete preflight cleanup")?;

    // Step 1: Setup Docker container - start with stopped state
    let stopped_container = StoppedProvisionedContainer::default();
    let running_container = stopped_container
        .start()
        .context("Failed to start provisioned instance container")?;

    // Step 2: Wait for SSH server and setup connectivity (only available when running)
    running_container
        .wait_for_ssh()
        .context("SSH server failed to start")?;

    running_container
        .setup_ssh_keys()
        .context("Failed to setup SSH authentication")?;

    let (ssh_host, ssh_port) = running_container.ssh_details();
    info!(
        ssh_host = %ssh_host,
        ssh_port = ssh_port,
        container_id = %running_container.container_id(),
        "Container ready for Ansible configuration"
    );

    // Step 2.5: Run provision simulation to render Ansible templates
    info!("Running provision simulation to prepare container configuration");
    let rt = Runtime::new().context("Failed to create tokio runtime")?;
    rt.block_on(run_provision_simulation(&running_container))?;

    // Step 3: Run configuration tasks (Ansible playbooks)
    info!("Running Ansible configuration tasks");
    run_ansible_configuration(&running_container)?;

    // Step 4: Validate deployment
    info!("Validating service deployment");
    let rt = Runtime::new().context("Failed to create tokio runtime")?;
    rt.block_on(run_deployment_validation(&running_container))?;

    // Step 5: Cleanup - transition back to stopped state
    let _stopped_container = running_container.stop();

    info!("Configuration tests completed successfully");

    Ok(())
}

/// Run provision simulation to prepare templates for container configuration
async fn run_provision_simulation(
    running_container: &torrust_tracker_deploy::e2e::provisioned_container::RunningProvisionedContainer,
) -> Result<()> {
    let (ssh_host, ssh_port) = running_container.ssh_details();

    info!(
        ssh_host = %ssh_host,
        ssh_port = ssh_port,
        "Running provision simulation for container"
    );

    // Create SSH credentials and configuration for the container
    let ssh_credentials = create_container_ssh_credentials()?;
    let config = create_container_config()?;
    let services = Services::new(&config);

    // Run the Docker infrastructure provision simulation
    provision_docker_infrastructure(
        Arc::clone(&services.ansible_template_renderer),
        ssh_credentials,
        ssh_host.parse()?,
        ssh_port,
    )
    .await
    .context("Failed to complete Docker infrastructure provision simulation")?;

    info!(
        status = "complete",
        "Provision simulation completed - Ansible templates rendered with container details"
    );

    Ok(())
}

/// Run Ansible configuration tasks on the container
fn run_ansible_configuration(
    running_container: &torrust_tracker_deploy::e2e::provisioned_container::RunningProvisionedContainer,
) -> Result<()> {
    let (ssh_host, ssh_port) = running_container.ssh_details();

    info!(
        ssh_host = %ssh_host,
        ssh_port = ssh_port,
        "Running Ansible configuration on container"
    );

    // NOTE: This demonstrates the configuration workflow structure, but currently
    // the ConfigureCommand uses LXD-based inventory that tries to connect to
    // 10.140.190.171 instead of 127.0.0.1:mapped_port for containers.
    //
    // To fully implement container-based configuration, we need to:
    // 1. Create container-specific Ansible inventory templates
    // 2. Modify Config/Services to support container-specific templates
    // 3. Update template rendering to use container host/port
    //
    // For now, we'll catch the expected connection error and log it:

    let config_result = create_container_config();
    match config_result {
        Ok(config) => {
            let services = Services::new(&config);
            let configure_command = ConfigureCommand::new(Arc::clone(&services.ansible_client));

            match configure_command.execute().map_err(anyhow::Error::from) {
                Ok(()) => {
                    info!(
                        status = "complete",
                        "Container configuration completed successfully"
                    );
                }
                Err(e) => {
                    // Expected failure due to inventory mismatch - log and return error
                    info!(
                        status = "expected_failure",
                        error = %e,
                        note = "ConfigureCommand failed as expected - needs container-specific inventory"
                    );
                    return Err(e.context(
                        "Configuration failed (expected - needs container-specific inventory)",
                    ));
                }
            }
        }
        Err(e) => {
            return Err(e.context("Failed to create container configuration"));
        }
    }

    info!(
        status = "structural_complete",
        "Configuration workflow structure implemented"
    );

    Ok(())
}

/// Run deployment validation tests on the container  
async fn run_deployment_validation(
    running_container: &torrust_tracker_deploy::e2e::provisioned_container::RunningProvisionedContainer,
) -> Result<()> {
    let (ssh_host, ssh_port) = running_container.ssh_details();

    info!(
        ssh_host = %ssh_host,
        ssh_port = ssh_port,
        "Running deployment validation on container"
    );

    // NOTE: Similar to configuration, validation needs container-specific setup.
    // The TestCommand expects standard SSH credentials but needs to connect to
    // 127.0.0.1:mapped_port instead of the provisioned instance IP.
    //
    // For now, we'll demonstrate the validation workflow structure:

    let credentials_result = create_container_ssh_credentials();
    match credentials_result {
        Ok(ssh_credentials) => {
            let instance_ip: IpAddr = ssh_host
                .parse()
                .context("Failed to parse SSH host as IP address")?;
            let test_command = TestCommand::new(ssh_credentials, instance_ip);

            match test_command.execute().await.map_err(anyhow::Error::from) {
                Ok(()) => {
                    info!(status = "success", "All deployment validations passed");
                }
                Err(e) => {
                    // Expected failure due to SSH connection issues - log and return error
                    info!(
                        status = "expected_failure",
                        error = %e,
                        note = "TestCommand failed as expected - needs container-specific SSH setup"
                    );
                    return Err(e.context(
                        "Validation failed (expected - needs container-specific SSH setup)",
                    ));
                }
            }
        }
        Err(e) => {
            return Err(e.context("Failed to create container SSH credentials"));
        }
    }

    info!(
        status = "structural_complete",
        "Validation workflow structure implemented"
    );

    Ok(())
}

/// Create a minimal configuration for container-based testing
fn create_container_config() -> Result<Config> {
    // For container testing, we use fixed test SSH keys from fixtures/
    let project_root = std::env::current_dir().context("Failed to get current directory")?;
    let ssh_credentials = SshCredentials::new(
        project_root.join("fixtures/testing_rsa"),
        project_root.join("fixtures/testing_rsa.pub"),
        "torrust".to_string(),
    );

    let instance_name = InstanceName::new("torrust-tracker-container".to_string())
        .context("Failed to create instance name")?;

    let project_root = std::env::current_dir().context("Failed to determine current directory")?;

    let build_dir = project_root.join("build");
    let templates_dir = "data/templates".to_string();

    Ok(Config::new(
        false, // Don't keep environment - cleanup after tests
        ssh_credentials,
        instance_name,
        templates_dir,
        project_root,
        build_dir,
    ))
}

/// Create SSH credentials for connecting to the container
fn create_container_ssh_credentials() -> Result<SshCredentials> {
    // Use the same test SSH keys as the configuration
    let project_root = std::env::current_dir().context("Failed to get current directory")?;
    let ssh_credentials = SshCredentials::new(
        project_root.join("fixtures/testing_rsa"),
        project_root.join("fixtures/testing_rsa.pub"),
        "torrust".to_string(),
    );

    Ok(ssh_credentials)
}
