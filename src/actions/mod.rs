use anyhow::Result;

pub mod cloud_init;
pub mod docker;

pub use cloud_init::CloudInitValidator;
pub use docker::DockerValidator;

/// Trait for remote actions that can be executed on a server via SSH
///
/// Remote actions are lightweight scripts that connect to a provisioned
/// server via SSH to perform various operations such as:
///
/// - Validating server state and configuration
/// - Retrieving server information (hostname, installed packages, etc.)
/// - Executing maintenance tasks (updates, cleanup, etc.)
/// - Installing or configuring software components
#[allow(async_fn_in_trait)]
pub trait RemoteAction {
    /// Get the name of this action for logging purposes
    fn name(&self) -> &'static str;

    /// Execute the action against the specified server
    ///
    /// # Arguments
    /// * `server_ip` - The IP address of the server to execute the action on
    ///
    /// # Returns
    /// * `Ok(())` if the action executes successfully
    /// * `Err(anyhow::Error)` if the action fails or encounters an error
    async fn execute(&self, server_ip: &str) -> Result<()>;
}
