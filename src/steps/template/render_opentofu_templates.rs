use std::sync::Arc;

use tracing::info;

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
    pub async fn execute(&self) -> Result<(), ProvisionTemplateError> {
        info!(
            step = "render_opentofu_templates",
            stage = 1,
            "Rendering OpenTofu templates"
        );

        self.tofu_template_renderer.render().await?;

        info!(
            step = "render_opentofu_templates",
            stage = 1,
            status = "success",
            "OpenTofu templates rendered successfully"
        );

        Ok(())
    }
}
