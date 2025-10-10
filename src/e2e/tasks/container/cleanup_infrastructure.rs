//! Container management task for E2E testing
//!
//! This module provides container management functionality for E2E testing,
//! including stopping containers and handling automatic cleanup through testcontainers.
//! It distinguishes between stopping containers (immediate) and cleanup (automatic).
//!
//! ## Key Operations
//!
//! - **Stop**: Immediately stops running Docker containers
//! - **Cleanup**: Automatic deletion handled by testcontainers library
//! - Provides logging for container operations
//!
//! ## Container vs VM Difference
//!
//! Unlike VMs that require explicit infrastructure destruction, Docker containers
//! managed by testcontainers are automatically deleted when they go out of scope.
//! This module provides both stop and cleanup functions for API symmetry.
//!
//! ## Integration
//!
//! This task is specifically designed for container-based E2E testing workflows.

use tracing::info;

use crate::e2e::containers::RunningProvisionedContainer;

/// Stop a running Docker container
///
/// This function stops a running Docker container, transitioning it from running
/// to stopped state. The actual container deletion is handled automatically by
/// the testcontainers library when the container goes out of scope.
///
/// # Arguments
///
/// * `running_container` - The running container to be stopped
///
/// # Returns
///
/// This function consumes the running container and returns nothing, ensuring
/// the container cannot be used after stopping.
///
/// # Example
///
/// ```rust,no_run
/// use torrust_tracker_deployer_lib::e2e::tasks::container::cleanup_infrastructure::stop_test_infrastructure;
/// use torrust_tracker_deployer_lib::e2e::containers::StoppedProvisionedContainer;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let stopped_container = StoppedProvisionedContainer::default();
///     let running_container = stopped_container.start(None, 22).await?;
///     
///     // ... perform tests ...
///     
///     stop_test_infrastructure(running_container);
///     println!("Container stopped successfully");
///     Ok(())
/// }
/// ```
pub fn stop_test_infrastructure(running_container: RunningProvisionedContainer) {
    let container_id = running_container.container_id().to_string();

    info!(
        container_id = %container_id,
        "Stopping test container"
    );

    // Transition container from running to stopped state
    let _stopped_container = running_container.stop();

    info!(
        container_id = %container_id,
        status = "success",
        "Container stopped successfully - deletion handled automatically by testcontainers"
    );
}

/// Cleanup test infrastructure for containers (no-op)
///
/// For containers managed by testcontainers, cleanup (deletion) is handled automatically
/// when containers go out of scope. This function is provided for API symmetry with
/// VM-based workflows that require explicit infrastructure destruction.
///
/// # Note
///
/// This function does nothing for containers. The actual cleanup is automatic.
/// Use `stop_test_infrastructure()` to stop running containers.
pub fn cleanup_test_infrastructure() {
    info!(
        operation = "container_cleanup",
        "Container cleanup is automatic via testcontainers - no action needed"
    );
}
