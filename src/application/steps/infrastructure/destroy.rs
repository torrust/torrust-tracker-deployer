//! `OpenTofu` infrastructure destruction step
//!
//! This module provides the `DestroyInfrastructureStep` which handles `OpenTofu`
//! destroy operations by executing `tofu destroy`. This step destroys all
//! infrastructure resources managed by the `OpenTofu` configuration.
//!
//! ## Key Features
//!
//! - Infrastructure teardown and resource destruction
//! - Configurable auto-approval for automation scenarios
//! - Progress tracking and status reporting
//! - Integration with `OpenTofuClient` for command execution
//!
//! ## Destroy Process
//!
//! The step executes `tofu destroy` which:
//! - Destroys all resources managed by the `OpenTofu` state
//! - Manages resource dependencies and proper destruction order
//! - Removes infrastructure state after successful destruction
//! - Provides detailed progress and completion status
//!
//! This step is where actual infrastructure destruction occurs.

use std::sync::Arc;

use tracing::{info, instrument};

use crate::adapters::tofu::client::OpenTofuClient;
use crate::shared::command::CommandError;

/// Simple step that destroys `OpenTofu` infrastructure by executing `tofu destroy`
pub struct DestroyInfrastructureStep {
    opentofu_client: Arc<OpenTofuClient>,
    auto_approve: bool,
}

impl DestroyInfrastructureStep {
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

    /// Execute the `OpenTofu` destroy step
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The `OpenTofu` destroy fails
    /// * The working directory does not exist or is not accessible
    /// * The `OpenTofu` command execution fails
    #[instrument(
        name = "destroy_infrastructure",
        skip_all,
        fields(
            step_type = "infrastructure",
            operation = "destroy",
            auto_approve = %self.auto_approve
        )
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "destroy_infrastructure",
            auto_approve = self.auto_approve,
            "Destroying OpenTofu infrastructure"
        );

        // Execute tofu destroy command with variables file
        let output = self
            .opentofu_client
            .destroy(self.auto_approve, &["-var-file=variables.tfvars"])?;

        info!(
            step = "destroy_infrastructure",
            status = "success",
            "OpenTofu infrastructure destroyed successfully"
        );

        // Log output for debugging if needed
        tracing::debug!(output = %output, "OpenTofu destroy output");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::adapters::tofu::client::OpenTofuClient;

    use super::*;

    #[test]
    fn it_should_create_destroy_infrastructure_step() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let _step = DestroyInfrastructureStep::new(opentofu_client);

        // If we reach this point, the step was created successfully
    }

    #[test]
    fn it_should_create_step_with_custom_auto_approve() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let step = DestroyInfrastructureStep::new(opentofu_client).with_auto_approve(false);

        assert!(!step.auto_approve);
    }

    #[test]
    fn it_should_default_to_auto_approve() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let step = DestroyInfrastructureStep::new(opentofu_client);

        assert!(step.auto_approve);
    }
}
