//! Instance information retrieval step
//!
//! This module provides the `GetInstanceInfoStep` which retrieves instance
//! information from `OpenTofu` outputs. This step extracts essential instance
//! data like IP addresses and metadata from the infrastructure state.
//!
//! ## Key Features
//!
//! - Instance information extraction from `OpenTofu` state outputs
//! - Provider-agnostic interface through standardized outputs
//! - JSON parsing and data validation
//! - Integration with `OpenTofuClient` for state queries
//!
//! ## Information Retrieval
//!
//! The step uses `tofu output` to retrieve instance information including:
//! - IP addresses for network connectivity
//! - Instance names and identifiers
//! - Status and configuration metadata
//! - Provider-specific details as needed
//!
//! This provides a consistent interface for accessing instance information
//! regardless of the underlying infrastructure provider.

use std::sync::Arc;

use tracing::{info, instrument};

use crate::adapters::tofu::client::{InstanceInfo, OpenTofuClient, OpenTofuError};
use crate::application::traits::CommandProgressListener;

/// Simple step that retrieves instance information from `OpenTofu` outputs
///
/// This step gets the instance IP from `OpenTofu` outputs rather than provider-specific methods
/// to provide a consistent interface across all providers. If we add more providers in the future,
/// the `OpenTofu` output provides a contract that always returns the expected instance info.
pub struct GetInstanceInfoStep {
    opentofu_client: Arc<OpenTofuClient>,
}

impl GetInstanceInfoStep {
    #[must_use]
    pub fn new(opentofu_client: Arc<OpenTofuClient>) -> Self {
        Self { opentofu_client }
    }

    /// Execute the get instance info step
    ///
    /// # Arguments
    ///
    /// * `listener` - Optional progress listener for reporting details
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * The `OpenTofu` output command fails
    /// * The output cannot be parsed as JSON
    /// * The `instance_info` section is missing or malformed
    /// * The working directory does not exist or is not accessible
    #[instrument(
        name = "get_instance_info",
        skip_all,
        fields(step_type = "infrastructure", operation = "info")
    )]
    pub fn execute(&self, listener: Option<&dyn CommandProgressListener>) -> Result<InstanceInfo, OpenTofuError> {
        info!(
            step = "get_instance_info",
            "Getting instance information from OpenTofu outputs"
        );

        // Get the instance IP from OpenTofu outputs
        // NOTE: We prefer OpenTofu outputs over provider-specific methods because:
        // - If we add more providers (different than LXD) in the future, we have two options:
        //   1. Use the method that each provider provides to get the IP
        //   2. Use OpenTofu for all of them, so the OpenTofu output has a contract with this app.
        //      It has to return always the instance info we expect.
        // Using OpenTofu outputs provides a consistent interface across all providers.
        let opentofu_instance_info = self.opentofu_client.get_instance_info()?;

        if let Some(l) = listener {
            l.on_detail(&format!("Instance IP: {}", opentofu_instance_info.ip_address));
        }

        info!(
            step = "get_instance_info",
            status = "success",
            ip_address = %opentofu_instance_info.ip_address,
            instance_name = %opentofu_instance_info.name,
            "Instance information retrieved successfully from OpenTofu outputs"
        );

        // Log output for debugging if needed
        tracing::debug!(instance_info = ?opentofu_instance_info, "OpenTofu instance info");

        Ok(opentofu_instance_info)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::adapters::tofu::client::OpenTofuClient;

    use super::*;

    #[test]
    fn it_should_create_get_instance_info_step() {
        let opentofu_client = Arc::new(OpenTofuClient::new("/tmp"));

        let _step = GetInstanceInfoStep::new(opentofu_client);

        // If we reach this point, the step was created successfully
    }
}
