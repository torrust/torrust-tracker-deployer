//! Container Command Executor Trait
//!
//! This module defines the trait that allows decoupled container actions
//! to execute commands inside running containers.

use testcontainers::core::ExecCommand;
use testcontainers::TestcontainersError;

/// Trait for containers that can execute commands inside themselves
///
/// This trait provides a standardized interface for executing commands within
/// running containers, enabling decoupled container actions that don't need
/// to know about the specific container implementation details.
///
/// ## Usage
///
/// Container actions can use this trait to execute commands without being
/// tightly coupled to the container implementation:
///
/// ```rust,no_run
/// use torrust_tracker_deploy::e2e::containers::ContainerExecutor;
/// use testcontainers::core::ExecCommand;
///
/// async fn setup_something<T: ContainerExecutor>(container: &T) -> Result<(), Box<dyn std::error::Error>> {
///     let result = container.exec(ExecCommand::new(["echo", "hello"])).await?;
///     Ok(())
/// }
/// ```
#[allow(async_fn_in_trait)]
pub trait ContainerExecutor {
    /// Execute a command inside the container
    ///
    /// # Arguments
    ///
    /// * `command` - The command to execute inside the container
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the command was executed successfully
    /// * `Err(TestcontainersError)` - If the command execution failed
    ///
    /// # Errors
    ///
    /// Returns an error if the command execution fails due to container issues,
    /// network problems, or other testcontainers-related errors.
    ///
    /// # Note
    ///
    /// The command execution may succeed even if the command itself fails
    /// (returns non-zero exit code). The caller should check the exit code
    /// in the returned result if needed.
    async fn exec(&self, command: ExecCommand) -> std::result::Result<(), TestcontainersError>;
}

#[cfg(test)]
mod tests {
    // Note: ContainerExecutor trait is no longer object-safe due to impl Future return type
    // This is expected since async traits can't be used as trait objects without boxing futures

    #[test]
    fn it_should_define_executor_trait_with_exec_method() {
        // Test that trait definition compiles correctly
        // The actual implementation will be tested with concrete types
    }
}
