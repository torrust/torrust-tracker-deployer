//! `OpenTofu` infrastructure planning step
//!
//! This module provides the `PlanInfrastructureStep` which handles `OpenTofu`
//! planning by executing `tofu plan`. This step creates an execution plan
//! showing what changes will be made to the infrastructure.
//!
//! ## Key Features
//!
//! - Infrastructure change planning and preview
//! - Resource dependency analysis and ordering
//! - Plan validation and error detection
//! - Integration with `OpenTofuClient` for command execution
//!
//! ## Planning Process
//!
//! The step executes `tofu plan` which:
//! - Analyzes current state vs desired configuration
//! - Determines what resources need to be created, modified, or destroyed
//! - Validates configuration and dependencies
//! - Provides a preview of changes before application
//!
//! This step is crucial for validating infrastructure changes before applying them.

use std::sync::Arc;

use tracing::{info, instrument};

use crate::infrastructure::adapters::opentofu::client::OpenTofuClient;
use crate::shared::executor::CommandError;

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
    #[instrument(
        name = "plan_infrastructure",
        skip_all,
        fields(step_type = "infrastructure", operation = "plan")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!(
            step = "plan_infrastructure",
            "Planning OpenTofu infrastructure"
        );

        // Execute tofu plan command with variables file
        let output = self.opentofu_client.plan(&["-var-file=variables.tfvars"])?;

        info!(
            step = "plan_infrastructure",
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

    use crate::infrastructure::adapters::opentofu::client::OpenTofuClient;

    use super::*;

    #[test]
    fn it_should_create_plan_infrastructure_step() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let _step = PlanInfrastructureStep::new(opentofu_client);

        // If we reach this point, the step was created successfully
    }
}
