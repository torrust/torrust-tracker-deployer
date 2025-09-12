use std::sync::Arc;

use tracing::info;

use crate::template::TemplateManager;
use crate::tofu::template_renderer::{ProvisionTemplateError, TofuTemplateRenderer};

/// Simple step that renders `OpenTofu` templates to the build directory
pub struct RenderOpenTofuTemplatesStep {
    tofu_template_renderer: Arc<TofuTemplateRenderer>,
    template_manager: Arc<TemplateManager>,
}

impl RenderOpenTofuTemplatesStep {
    #[must_use]
    pub fn new(
        tofu_template_renderer: Arc<TofuTemplateRenderer>,
        template_manager: Arc<TemplateManager>,
    ) -> Self {
        Self {
            tofu_template_renderer,
            template_manager,
        }
    }

    /// Execute the template rendering step
    ///
    /// # Errors
    ///
    /// Returns an error if the template rendering fails or if there are issues
    /// with the template manager or renderer.
    pub async fn execute(&self) -> Result<(), ProvisionTemplateError> {
        info!("📝 Stage 1: Rendering OpenTofu templates...");

        self.tofu_template_renderer
            .render(&self.template_manager)
            .await?;

        info!("✅ Stage 1: OpenTofu templates rendered successfully");

        Ok(())
    }
}
