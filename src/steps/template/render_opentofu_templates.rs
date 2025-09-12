use crate::container::Services;
use crate::tofu::template_renderer::ProvisionTemplateError;

/// Simple step that renders `OpenTofu` templates to the build directory
pub struct RenderOpenTofuTemplatesStep<'a> {
    services: &'a Services,
    verbose: bool,
}

impl<'a> RenderOpenTofuTemplatesStep<'a> {
    #[must_use]
    pub fn new(services: &'a Services, verbose: bool) -> Self {
        Self { services, verbose }
    }

    /// Execute the template rendering step
    ///
    /// # Errors
    ///
    /// Returns an error if the template rendering fails or if there are issues
    /// with the template manager or renderer.
    pub async fn execute(&self) -> Result<(), ProvisionTemplateError> {
        if self.verbose {
            println!("📝 Stage 1: Rendering OpenTofu templates...");
        }

        self.services
            .tofu_template_renderer
            .render(&self.services.template_manager)
            .await?;

        if self.verbose {
            println!("✅ Stage 1: OpenTofu templates rendered successfully");
        }

        Ok(())
    }
}
