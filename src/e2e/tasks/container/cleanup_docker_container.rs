//! Container cleanup task for E2E testing
//!
//! This module provides the E2E testing task for cleaning up Docker containers
//! after test completion. It handles the transition from running to stopped
//! container state and ensures proper resource cleanup.
//!
//! ## Key Operations
//!
//! - Stops running Docker containers
//! - Cleans up container resources
//! - Provides logging for cleanup operations
//!
//! ## Integration
//!
//! This task is specifically designed for container-based E2E testing workflows
//! and should be used as the final step in container test scenarios.

use tracing::info;

use crate::e2e::containers::RunningProvisionedContainer;

/// Cleanup a running Docker container
///
/// This function stops a running Docker container and performs cleanup operations.
/// It transitions the container from running to stopped state and ensures proper
/// resource cleanup.
///
/// # Arguments
///
/// * `running_container` - The running container to be cleaned up
///
/// # Returns
///
/// This function consumes the running container and returns nothing, ensuring
/// the container cannot be used after cleanup.
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deploy::e2e::tasks::container::cleanup_docker_container::cleanup_docker_container;
/// use torrust_tracker_deploy::e2e::containers::StoppedProvisionedContainer;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let stopped_container = StoppedProvisionedContainer::default();
///     let running_container = stopped_container.start().await?;
///     
///     // ... perform tests ...
///     
///     cleanup_docker_container(running_container);
///     println!("Container cleanup completed");
///     Ok(())
/// }
/// ```
pub fn cleanup_docker_container(running_container: RunningProvisionedContainer) {
    let container_id = running_container.container_id().to_string();

    info!(
        container_id = %container_id,
        "Starting container cleanup"
    );

    // Transition container from running to stopped state
    let _stopped_container = running_container.stop();

    info!(
        container_id = %container_id,
        status = "success",
        "Container cleanup completed successfully"
    );
}
