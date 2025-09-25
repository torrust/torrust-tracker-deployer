//! Docker container setup task for E2E testing
//!
//! This module provides the E2E testing task for setting up Docker containers
//! that can be used in configuration testing scenarios. It handles the complete
//! container lifecycle from stopped to running state.
//!
//! ## Key Operations
//!
//! - Creates and starts a Docker container using the provisioned instance image
//! - Returns a running container that can be used for SSH connectivity and configuration
//!
//! ## Integration
//!
//! This task creates the foundation for container-based E2E testing by providing
//! a controlled environment that can be configured with Ansible playbooks.

use anyhow::{Context, Result};
use tracing::info;

use crate::e2e::containers::{RunningProvisionedContainer, StoppedProvisionedContainer};

/// Setup and start a Docker container for E2E testing
///
/// This function creates a new Docker container from the provisioned instance image
/// and starts it, making it ready for SSH connectivity and configuration testing.
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
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deploy::e2e::tasks::container::setup_docker_container::setup_docker_container;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let running_container = setup_docker_container().await?;
///     println!("Container running on: {}", running_container.ssh_socket_addr());
///     Ok(())
/// }
/// ```
pub async fn setup_docker_container() -> Result<RunningProvisionedContainer> {
    info!("Setting up Docker container for E2E testing");

    let stopped_container = StoppedProvisionedContainer::default();
    let running_container = stopped_container
        .start()
        .await
        .context("Failed to start provisioned instance container")?;

    info!(
        container_id = %running_container.container_id(),
        ssh_socket_addr = %running_container.ssh_socket_addr(),
        "Docker container setup completed successfully"
    );

    Ok(running_container)
}
