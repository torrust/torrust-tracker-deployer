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

use crate::config::SshCredentials;
use crate::container::Services;
use crate::e2e::environment::TestEnvironment;
use crate::e2e::tasks::container::provision_docker_infrastructure::provision_docker_infrastructure;

/// Run provision simulation to prepare templates for container-based testing
///
/// This function simulates the provision phase specifically for Docker containers
/// by rendering Ansible templates with the container's connection details. Since
/// Docker handles the infrastructure creation, this task focuses on preparing
/// the configuration templates that would normally be generated after VM provisioning.
///
/// # Arguments
///
/// * `socket_addr` - Socket address where the Docker container can be reached
/// * `ssh_credentials` - SSH credentials for connecting to the container
/// * `test_env` - Test environment containing configuration and services
///
/// # Returns
///
/// Returns `Ok(())` when provision simulation is completed successfully.
///
/// # Errors
///
/// Returns an error if:
/// - SSH credentials cannot be validated
/// - Ansible template rendering fails for container configuration
/// - Container services cannot be initialized
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deploy::e2e::tasks::container::run_provision_simulation::run_provision_simulation;
/// use torrust_tracker_deploy::e2e::environment::TestEnvironment;
/// use torrust_tracker_deploy::config::{InstanceName, SshCredentials};
/// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Container typically runs on 127.0.0.1 with a mapped port
///     let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2222);
///     let instance_name = InstanceName::new("test-container".to_string())?;
///     let test_env = TestEnvironment::new(false, "./templates".to_string(), instance_name)?;
///     
///     let ssh_credentials = SshCredentials::new(
///         "./id_rsa".into(),
///         "./id_rsa.pub".into(),
///         "testuser".to_string()
///     );
///     
///     run_provision_simulation(socket_addr, &ssh_credentials, &test_env).await?;
///     println!("Container provision simulation completed");
///     Ok(())
/// }
/// ```
pub async fn run_provision_simulation(
    socket_addr: SocketAddr,
    ssh_credentials: &SshCredentials,
    test_env: &TestEnvironment,
) -> Result<()> {
    info!(
        socket_addr = %socket_addr,
        ssh_user = %ssh_credentials.ssh_username,
        "Running provision simulation to prepare container configuration templates"
    );

    // Initialize services from test environment configuration
    let services = Services::new(&test_env.config);

    // Run the container infrastructure provision simulation using existing Docker task
    // This renders Ansible templates with the container connection details
    provision_docker_infrastructure(
        Arc::clone(&services.ansible_template_renderer),
        ssh_credentials.clone(),
        socket_addr,
    )
    .await
    .context("Failed to complete container provision simulation")?;

    info!(
        socket_addr = %socket_addr,
        status = "complete",
        "Container provision simulation completed - Ansible templates rendered with container details"
    );

    Ok(())
}
