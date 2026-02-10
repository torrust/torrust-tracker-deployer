//! `OpenTofu` Template Rendering Service
//!
//! This service is responsible for rendering `OpenTofu` (Terraform) infrastructure templates.
//! It's used by multiple contexts (render command, provision steps) to prepare
//! infrastructure-as-code files.

use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tracing::info;

use crate::adapters::ssh::SshCredentials;
use crate::domain::provider::ProviderConfig;
use crate::domain::InstanceName;
use crate::domain::TemplateManager;
use crate::infrastructure::templating::tofu::{TofuProjectGenerator, TofuProjectGeneratorError};
use crate::shared::Clock;

/// Errors that can occur during `OpenTofu` template rendering
#[derive(Error, Debug)]
pub enum OpenTofuTemplateRenderingServiceError {
    /// Template rendering failed
    #[error("Failed to render OpenTofu templates: {reason}")]
    RenderingFailed {
        /// Detailed reason for the failure
        reason: String,
    },
}

impl From<TofuProjectGeneratorError> for OpenTofuTemplateRenderingServiceError {
    fn from(error: TofuProjectGeneratorError) -> Self {
        Self::RenderingFailed {
            reason: error.to_string(),
        }
    }
}

/// Service for rendering `OpenTofu` infrastructure templates
///
/// This service encapsulates the logic for rendering `OpenTofu` (Terraform)
/// configuration files. It's designed to be shared across command handlers
/// and steps that need to prepare infrastructure templates.
///
/// Note: `OpenTofu` requires more configuration than other template types because
/// it needs provider-specific settings, SSH credentials, and instance metadata.
pub struct OpenTofuTemplateRenderingService {
    generator: TofuProjectGenerator,
}

impl OpenTofuTemplateRenderingService {
    /// Build an `OpenTofuTemplateRenderingService` from configuration parameters
    ///
    /// # Arguments
    ///
    /// * `templates_dir` - Directory containing the source templates
    /// * `build_dir` - Directory where rendered templates will be written
    /// * `ssh_credentials` - SSH credentials for accessing the provisioned instance
    /// * `ssh_port` - SSH port for the instance
    /// * `instance_name` - Name of the instance to provision
    /// * `provider_config` - Provider-specific configuration (LXD, Docker, etc.)
    /// * `clock` - The clock for generating timestamps
    ///
    /// # Returns
    ///
    /// Returns a configured `OpenTofuTemplateRenderingService` ready for template rendering
    #[must_use]
    pub fn from_params(
        templates_dir: PathBuf,
        build_dir: PathBuf,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
        instance_name: InstanceName,
        provider_config: ProviderConfig,
        clock: Arc<dyn Clock>,
    ) -> Self {
        let template_manager = Arc::new(TemplateManager::new(templates_dir));

        let generator = TofuProjectGenerator::new(
            template_manager,
            build_dir,
            ssh_credentials,
            ssh_port,
            instance_name,
            provider_config,
            clock,
        );

        Self { generator }
    }

    /// Render `OpenTofu` infrastructure templates
    ///
    /// This renders the `OpenTofu` configuration files (main.tf, variables.tf, etc.)
    /// to the build directory.
    ///
    /// # Errors
    ///
    /// Returns `OpenTofuTemplateRenderingServiceError::RenderingFailed` if template rendering fails.
    pub async fn render(&self) -> Result<(), OpenTofuTemplateRenderingServiceError> {
        info!("Rendering OpenTofu infrastructure templates");

        self.generator.render().await?;

        info!("OpenTofu infrastructure templates rendered successfully");

        Ok(())
    }
}
