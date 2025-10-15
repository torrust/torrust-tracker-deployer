//! `OpenTofu` infrastructure initialization step
//!
//! This module provides the `InitializeInfrastructureStep` which handles `OpenTofu`
//! initialization by executing `tofu init`. This step prepares the working directory
//! for infrastructure operations by downloading providers and initializing state.
//!
//! ## Key Features
//!
//! - `OpenTofu` working directory initialization
//! - Provider plugin downloading and installation
//! - Backend configuration and state initialization
//! - Integration with `OpenTofuClient` for command execution
//!
//! ## Initialization Process
//!
//! The step executes `tofu init` which performs:
//! - Provider plugin resolution and download
//! - Backend initialization (local or remote state)
//! - Module downloading if applicable
//! - Working directory setup for subsequent operations
//!
//! This is typically the first step in any infrastructure provisioning workflow.

use std::sync::Arc;

use tracing::{info, instrument};

use crate::adapters::tofu::client::OpenTofuClient;
use crate::shared::command::CommandError;

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
    #[instrument(
        name = "initialize_infrastructure",
        skip_all,
        fields(step_type = "infrastructure", operation = "init")
    )]
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

    use crate::adapters::tofu::client::OpenTofuClient;

    use super::*;

    #[test]
    fn it_should_create_initialize_infrastructure_step() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let _step = InitializeInfrastructureStep::new(opentofu_client);

        // If we reach this point, the step was created successfully
    }
}
