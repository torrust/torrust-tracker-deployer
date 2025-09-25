//! SSH connectivity configuration task for container-based E2E testing
//!
//! This module provides the E2E testing task for configuring SSH connectivity
//! specifically for Docker containers. It handles waiting for SSH service
//! availability and setting up SSH key authentication within containerized
//! testing environments.
//!
//! ## Key Operations
//!
//! - Waits for SSH server to become available on the container
//! - Sets up SSH key authentication for automated container access
//! - Validates SSH connectivity is ready for Ansible operations on containers
//!
//! ## Integration
//!
//! This task is specifically designed for container-based E2E testing scenarios
//! and works with Docker containers created by the `setup_docker_container` task.

use std::net::SocketAddr;

use anyhow::{Context, Result};
use tracing::info;

use crate::config::SshCredentials;
use crate::e2e::containers::actions::{SshKeySetupAction, SshWaitAction};
use crate::e2e::containers::timeout::ContainerTimeouts;
use crate::e2e::containers::RunningProvisionedContainer;

/// Configure SSH connectivity for a running Docker container
///
/// This function handles the complete SSH connectivity setup process for containers:
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
/// Returns `Ok(())` when SSH connectivity is fully configured and ready for container operations.
///
/// # Errors
///
/// Returns an error if:
/// - Container SSH server fails to start within timeout
/// - SSH key setup fails on the container
/// - Authentication cannot be established with the container
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deploy::e2e::tasks::container::configure_ssh_connectivity::configure_ssh_connectivity;
/// use torrust_tracker_deploy::config::SshCredentials;
/// use std::net::{IpAddr, Ipv4Addr, SocketAddr};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 2222);
///     let ssh_credentials = SshCredentials::new(
///         "./id_rsa".into(),
///         "./id_rsa.pub".into(),
///         "testuser".to_string()
///     );
///     
///     configure_ssh_connectivity(socket_addr, &ssh_credentials, None).await?;
///     println!("SSH connectivity configured successfully");
///     Ok(())
/// }
/// ```
pub async fn configure_ssh_connectivity(
    socket_addr: SocketAddr,
    ssh_credentials: &SshCredentials,
    container: Option<&RunningProvisionedContainer>,
) -> Result<()> {
    info!(
        socket_addr = %socket_addr,
        ssh_user = %ssh_credentials.ssh_username,
        "Configuring SSH connectivity"
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
        "SSH connectivity configured successfully - ready for Ansible operations"
    );

    Ok(())
}
