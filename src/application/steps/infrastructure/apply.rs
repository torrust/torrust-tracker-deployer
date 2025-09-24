//! `OpenTofu` infrastructure application step
//!
//! This module provides the `ApplyInfrastructureStep` which handles `OpenTofu`
//! application by executing `tofu apply`. This step applies the planned
//! infrastructure changes to provision or modify resources.
//!
//! ## Key Features
//!
//! - Infrastructure provisioning and resource creation
//! - Configurable auto-approval for automation scenarios
//! - Progress tracking and status reporting
//! - Integration with `OpenTofuClient` for command execution
//!
//! ## Application Process
//!
//! The step executes `tofu apply` which:
//! - Applies planned changes to create/modify/destroy resources
//! - Manages resource dependencies and ordering
//! - Updates infrastructure state with actual resource information
//! - Provides detailed progress and completion status
//!
//! This step is where actual infrastructure provisioning occurs.

use std::sync::Arc;

use tracing::{info, instrument};

use crate::infrastructure::adapters::opentofu::client::OpenTofuClient;
use crate::shared::executor::CommandError;

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
    #[instrument(
        name = "apply_infrastructure",
        skip_all,
        fields(
            step_type = "infrastructure",
            operation = "apply",
            auto_approve = %self.auto_approve
        )
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "apply_infrastructure",
            auto_approve = self.auto_approve,
            "Applying OpenTofu infrastructure"
        );

        // Execute tofu apply command with variables file
        let output = self
            .opentofu_client
            .apply(self.auto_approve, &["-var-file=variables.tfvars"])?;

        info!(
            step = "apply_infrastructure",
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

    use crate::infrastructure::adapters::opentofu::client::OpenTofuClient;

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
