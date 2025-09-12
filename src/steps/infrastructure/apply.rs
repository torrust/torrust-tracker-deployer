use std::sync::Arc;

use tracing::info;

use crate::command::CommandError;
use crate::command_wrappers::opentofu::client::OpenTofuClient;

/// Simple step that applies `OpenTofu` configuration by executing `tofu apply`
pub struct ApplyInfrastructureStep {
    opentofu_client: Arc<OpenTofuClient>,
    auto_approve: bool,
}

impl ApplyInfrastructureStep {
    #[must_use]
    pub fn new(opentofu_client: Arc<OpenTofuClient>) -> Self {
        Self {
            opentofu_client,
            auto_approve: true, // Default to auto-approve for automation
        }
    }

    #[must_use]
    pub fn with_auto_approve(mut self, auto_approve: bool) -> Self {
        self.auto_approve = auto_approve;
        self
    }

    /// Execute the `OpenTofu` apply step
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The `OpenTofu` apply fails
    /// * The working directory does not exist or is not accessible
    /// * The `OpenTofu` command execution fails
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "apply_infrastructure",
            stage = 2,
            auto_approve = self.auto_approve,
            "Applying OpenTofu infrastructure"
        );

        // Execute tofu apply command
        let output = self.opentofu_client.apply(self.auto_approve)?;

        info!(
            step = "apply_infrastructure",
            stage = 2,
            status = "success",
            "OpenTofu infrastructure applied successfully"
        );

        // Log output for debugging if needed
        tracing::debug!(output = %output, "OpenTofu apply output");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::command_wrappers::opentofu::client::OpenTofuClient;

    use super::*;

    #[test]
    fn it_should_create_apply_infrastructure_step() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let _step = ApplyInfrastructureStep::new(opentofu_client);

        // If we reach this point, the step was created successfully
    }

    #[test]
    fn it_should_create_step_with_custom_auto_approve() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let step = ApplyInfrastructureStep::new(opentofu_client).with_auto_approve(false);

        assert!(!step.auto_approve);
    }

    #[test]
    fn it_should_default_to_auto_approve() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let step = ApplyInfrastructureStep::new(opentofu_client);

        assert!(step.auto_approve);
    }
}
