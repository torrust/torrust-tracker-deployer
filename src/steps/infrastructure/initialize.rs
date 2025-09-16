use std::sync::Arc;

use tracing::info;

use crate::command::CommandError;
use crate::command_wrappers::opentofu::client::OpenTofuClient;

/// Simple step that initializes `OpenTofu` configuration by executing `tofu init`
pub struct InitializeInfrastructureStep {
    opentofu_client: Arc<OpenTofuClient>,
}

impl InitializeInfrastructureStep {
    #[must_use]
    pub fn new(opentofu_client: Arc<OpenTofuClient>) -> Self {
        Self { opentofu_client }
    }

    /// Execute the `OpenTofu` initialization step
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The `OpenTofu` initialization fails
    /// * The working directory does not exist or is not accessible
    /// * The `OpenTofu` command execution fails
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "initialize_infrastructure",
            "Initializing OpenTofu infrastructure"
        );

        // Execute tofu init command
        let output = self.opentofu_client.init()?;

        info!(
            step = "initialize_infrastructure",
            status = "success",
            "OpenTofu infrastructure initialized successfully"
        );

        // Log output for debugging if needed
        tracing::debug!(output = %output, "OpenTofu init output");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::command_wrappers::opentofu::client::OpenTofuClient;

    use super::*;

    #[test]
    fn it_should_create_initialize_infrastructure_step() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let _step = InitializeInfrastructureStep::new(opentofu_client);

        // If we reach this point, the step was created successfully
    }
}
