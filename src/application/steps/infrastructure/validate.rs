//! `OpenTofu` infrastructure validation step
//!
//! This module provides the `ValidateInfrastructureStep` which handles `OpenTofu`
//! validation by executing `tofu validate`. This step validates the syntax and
//! internal consistency of configuration files without creating a plan or applying changes.
//!
//! ## Key Features
//!
//! - Configuration syntax validation and error detection
//! - Internal consistency checks for resource definitions
//! - Provider schema validation against installed providers
//! - Integration with `OpenTofuClient` for command execution
//!
//! ## Validation Process
//!
//! The step executes `tofu validate` which:
//! - Validates syntax of all `.tf` configuration files
//! - Checks for missing required arguments and invalid attribute names
//! - Validates resource and data source configurations against provider schemas
//! - Ensures internal consistency of variable references and expressions
//!
//! This step should be run after initialization but before planning to catch
//! configuration errors early in the workflow.

use std::sync::Arc;

use tracing::{info, instrument};

use crate::adapters::tofu::client::OpenTofuClient;
use crate::application::traits::CommandProgressListener;
use crate::shared::command::CommandError;

/// Simple step that validates `OpenTofu` configuration by executing `tofu validate`
pub struct ValidateInfrastructureStep {
    opentofu_client: Arc<OpenTofuClient>,
}

impl ValidateInfrastructureStep {
    #[must_use]
    pub fn new(opentofu_client: Arc<OpenTofuClient>) -> Self {
        Self { opentofu_client }
    }

    /// Execute the `OpenTofu` validation step
    ///
    /// # Arguments
    ///
    /// * `listener` - Optional progress listener for reporting details
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The `OpenTofu` validation fails due to syntax or consistency errors
    /// * The working directory does not exist or is not accessible
    /// * The `OpenTofu` command execution fails
    /// * The configuration is not initialized (providers not installed)
    #[instrument(
        name = "validate_infrastructure",
        skip_all,
        fields(step_type = "infrastructure", operation = "validate")
    )]
    pub fn execute(
        &self,
        listener: Option<&dyn CommandProgressListener>,
    ) -> Result<(), CommandError> {
        info!(
            step = "validate_infrastructure",
            "Validating OpenTofu configuration"
        );

        // Execute tofu validate command
        let output = self.opentofu_client.validate()?;

        if let Some(l) = listener {
            l.on_detail("Configuration is valid ✓");
        }

        info!(
            step = "validate_infrastructure",
            status = "success",
            "OpenTofu configuration validated successfully"
        );

        // Log output for debugging if needed
        tracing::debug!(output = %output, "OpenTofu validate output");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::adapters::tofu::client::OpenTofuClient;

    use super::*;

    #[test]
    fn it_should_create_validate_infrastructure_step() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let _step = ValidateInfrastructureStep::new(opentofu_client);

        // If we reach this point, the step was created successfully
    }
}
