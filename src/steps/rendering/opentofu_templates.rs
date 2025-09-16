//! `OpenTofu` template rendering step
//!
//! This module provides the `RenderOpenTofuTemplatesStep` which handles rendering
//! of `OpenTofu` configuration templates to the build directory. This step prepares
//! infrastructure configuration files for `OpenTofu` operations.
//!
//! ## Key Features
//!
//! - Static template rendering for `OpenTofu` configurations
//! - Integration with the `TofuTemplateRenderer` for file generation
//! - Build directory preparation for infrastructure operations
//! - Comprehensive error handling for template processing
//!
//! ## Usage Context
//!
//! This step is typically executed early in the deployment workflow, before
//! infrastructure provisioning, to prepare the `OpenTofu` configuration files
//! needed for infrastructure operations.

use std::sync::Arc;

use tracing::{info, instrument};

use crate::tofu::template_renderer::{ProvisionTemplateError, TofuTemplateRenderer};

/// Simple step that renders `OpenTofu` templates to the build directory
pub struct RenderOpenTofuTemplatesStep {
    tofu_template_renderer: Arc<TofuTemplateRenderer>,
}

impl RenderOpenTofuTemplatesStep {
    #[must_use]
    pub fn new(tofu_template_renderer: Arc<TofuTemplateRenderer>) -> Self {
        Self {
            tofu_template_renderer,
        }
    }

    /// Execute the template rendering step
    ///
    /// # Errors
    ///
    /// Returns an error if the template rendering fails or if there are issues
    /// with the template manager or renderer.
    #[instrument(
        name = "render_opentofu_templates",
        skip_all,
        fields(step_type = "rendering", template_type = "opentofu")
    )]
    pub async fn execute(&self) -> Result<(), ProvisionTemplateError> {
        info!(
            step = "render_opentofu_templates",
            "Rendering OpenTofu templates"
        );

        self.tofu_template_renderer.render().await?;

        info!(
            step = "render_opentofu_templates",
            status = "success",
            "OpenTofu templates rendered successfully"
        );

        Ok(())
    }
}
