use std::sync::Arc;

use tracing::info;

use crate::command::CommandError;
use crate::command_wrappers::opentofu::client::OpenTofuClient;

/// Simple step that plans `OpenTofu` configuration by executing `tofu plan`
pub struct PlanInfrastructureStep {
    opentofu_client: Arc<OpenTofuClient>,
}

impl PlanInfrastructureStep {
    #[must_use]
    pub fn new(opentofu_client: Arc<OpenTofuClient>) -> Self {
        Self { opentofu_client }
    }

    /// Execute the `OpenTofu` plan step
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The `OpenTofu` plan fails
    /// * The working directory does not exist or is not accessible
    /// * The `OpenTofu` command execution fails
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "plan_infrastructure",
            stage = 2,
            "Planning OpenTofu infrastructure"
        );

        // Execute tofu plan command
        let output = self.opentofu_client.plan()?;

        info!(
            step = "plan_infrastructure",
            stage = 2,
            status = "success",
            "OpenTofu infrastructure planned successfully"
        );

        // Log output for debugging if needed
        tracing::debug!(output = %output, "OpenTofu plan output");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::command_wrappers::opentofu::client::OpenTofuClient;

    use super::*;

    #[test]
    fn it_should_create_plan_infrastructure_step() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let _step = PlanInfrastructureStep::new(opentofu_client);

        // If we reach this point, the step was created successfully
    }
}
