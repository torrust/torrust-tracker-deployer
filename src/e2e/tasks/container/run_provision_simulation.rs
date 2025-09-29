//! Provision simulation task for container-based E2E testing
//!
//! This module provides the E2E testing task for simulating the provision phase
//! specifically for Docker container-based testing. When using containers,
//! infrastructure provisioning is handled by Docker, but we still need to render
//! Ansible templates and configurations as if a traditional provision had occurred.
//!
//! ## Key Operations
//!
//! - Renders Ansible inventory templates with container connection details
//! - Prepares configuration files for Ansible playbook execution on containers
//! - Simulates the post-provisioning state that would normally be created by `OpenTofu`
//!
//! ## Integration
//!
//! This task is specifically designed for container-based E2E testing scenarios
//! where Docker provides the infrastructure and we need to simulate the provision
//! phase that would normally be handled by infrastructure-as-code tools.

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::info;

use crate::application::steps::RenderAnsibleTemplatesStep;
use crate::config::SshCredentials;
use crate::container::Services;
use crate::e2e::containers::actions::{SshKeySetupAction, SshWaitAction};
use crate::e2e::containers::timeout::ContainerTimeouts;
use crate::e2e::containers::{RunningProvisionedContainer, StoppedProvisionedContainer};
use crate::e2e::environment::TestEnvironment;
use crate::infrastructure::ansible::AnsibleTemplateRenderer;

/// Run provision simulation to prepare templates for container-based testing
///
/// This function simulates the provision phase specifically for Docker containers
/// by setting up the container, establishing SSH connectivity, and rendering
/// Ansible templates with the container's connection details. Since Docker handles
/// the infrastructure creation, this task focuses on preparing the configuration
/// templates that would normally be generated after VM provisioning.
///
/// # Arguments
///
/// * `test_env` - Test environment containing configuration and services
///
/// # Returns
///
/// Returns `Ok(RunningProvisionedContainer)` when provision simulation is completed
/// successfully and the container is ready for further configuration.
///
/// # Errors
///
/// Returns an error if:
/// - Docker container setup fails
/// - SSH connectivity cannot be established  
/// - SSH credentials cannot be validated
/// - Ansible template rendering fails for container configuration
/// - Container services cannot be initialized
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deploy::e2e::tasks::container::run_provision_simulation::run_provision_simulation;
/// use torrust_tracker_deploy::e2e::environment::{TestEnvironment, TestEnvironmentType};
/// use torrust_tracker_deploy::config::InstanceName;
/// use torrust_tracker_deploy::shared::Username;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let instance_name = InstanceName::new("test-container".to_string())?;
///     let ssh_user = Username::new("torrust")?;
///     let ssh_private_key_path = std::path::PathBuf::from("fixtures/testing_rsa");
///     let ssh_public_key_path = std::path::PathBuf::from("fixtures/testing_rsa.pub");
///     let test_env = TestEnvironment::initialized(
///         false,
///         "./templates".to_string(),
///         &ssh_user,
///         instance_name,
///         ssh_private_key_path,
///         ssh_public_key_path,
///         TestEnvironmentType::Container
///     )?;
///     
///     let running_container = run_provision_simulation(&test_env).await?;
///     println!("Container provision simulation completed: {}", running_container.ssh_socket_addr());
///     Ok(())
/// }
/// ```
pub async fn run_provision_simulation(
    test_env: &TestEnvironment,
) -> Result<RunningProvisionedContainer> {
    info!("Running provision simulation to prepare container configuration templates");

    // Step 1: Setup Docker container
    let running_container =
        create_and_start_container(test_env.config.instance_name.as_str().to_string()).await?;

    let socket_addr = running_container.ssh_socket_addr();

    // Step 2: Establish SSH connectivity
    establish_ssh_connectivity(
        socket_addr,
        &test_env.config.ssh_credentials,
        Some(&running_container),
    )
    .await?;

    // Step 3: Initialize services from test environment configuration
    let services = Services::new(&test_env.config);

    // Step 4: Render Ansible configuration templates with container connection details
    render_ansible_configuration(
        Arc::clone(&services.ansible_template_renderer),
        test_env.config.ssh_credentials.clone(),
        socket_addr,
    )
    .await
    .context("Failed to complete container provision simulation")?;

    info!(
        socket_addr = %socket_addr,
        container_id = %running_container.container_id(),
        status = "complete",
        "Container provision simulation completed - Ansible templates rendered with container details"
    );

    Ok(running_container)
}

/// Create and start a Docker container for E2E testing
///
/// This function creates a new Docker container from the provisioned instance image
/// and starts it, making it ready for SSH connectivity and configuration testing.
///
/// # Arguments
///
/// * `container_name` - Name for the container. The container will be created with this name.
///
/// # Returns
///
/// Returns a `RunningProvisionedContainer` that can be used for:
/// - SSH connectivity testing
/// - Ansible configuration
/// - Service validation
/// - Container cleanup
///
/// # Errors
///
/// Returns an error if:
/// - Container creation fails
/// - Container startup fails
/// - Docker daemon is not available
async fn create_and_start_container(container_name: String) -> Result<RunningProvisionedContainer> {
    info!(container_name = %container_name, "Creating and starting Docker container for E2E testing");

    let stopped_container = StoppedProvisionedContainer::default();

    let running_container = stopped_container
        .start(Some(container_name.clone()))
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
/// This function handles the complete SSH connectivity establishment process for containers:
/// 1. Waits for SSH server to become available on the container
/// 2. Sets up SSH key authentication for container access
/// 3. Validates connectivity is ready for Ansible operations
///
/// # Arguments
///
/// * `socket_addr` - Socket address (IP and port) where the container's SSH server is running
/// * `ssh_credentials` - SSH credentials containing keys and username
/// * `container` - Optional running container reference for key setup
///
/// # Returns
///
/// Returns `Ok(())` when SSH connectivity is fully established and ready for container operations.
///
/// # Errors
///
/// Returns an error if:
/// - Container SSH server fails to start within timeout
/// - SSH key setup fails on the container
/// - Authentication cannot be established with the container
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

    // Step 1: Wait for SSH server to become available
    let timeouts = ContainerTimeouts::default();
    let ssh_wait_action = SshWaitAction::new(timeouts.ssh_ready, 10);
    ssh_wait_action
        .execute(socket_addr)
        .context("SSH server failed to start")?;

    // Step 2: Setup SSH key authentication (only for containers currently)
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
        "SSH connectivity established successfully - ready for Ansible operations"
    );

    Ok(())
}

/// Render Ansible configuration templates for container-based E2E testing
///
/// This function renders Ansible templates with the container's connection details,
/// preparing the configuration files needed for Ansible playbook execution.
/// SSH connectivity is assumed to be already established by the container startup process.
///
/// # Arguments
///
/// * `ansible_template_renderer` - Renderer for creating Ansible inventory and configuration
/// * `ssh_credentials` - SSH credentials for connecting to the container
/// * `socket_addr` - Socket address (IP and port) where the container can be reached
///
/// # Errors
///
/// Returns an error if:
/// - Ansible template rendering fails
async fn render_ansible_configuration(
    ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
    ssh_credentials: SshCredentials,
    socket_addr: SocketAddr,
) -> Result<()> {
    info!(
        socket_addr = %socket_addr,
        "Rendering Ansible configuration templates"
    );

    // Step 1: Render Ansible templates with container connection details
    info!("Rendering Ansible templates for container");
    RenderAnsibleTemplatesStep::new(ansible_template_renderer, ssh_credentials, socket_addr)
        .execute()
        .await
        .context("Failed to render Ansible templates for container")?;

    // Note: SSH connectivity check is skipped for Docker containers since
    // the container setup process already ensures SSH is ready and accessible

    info!(
        socket_addr = %socket_addr,
        "Ansible configuration templates rendered successfully"
    );

    Ok(())
}
