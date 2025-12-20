//! Grafana template rendering step
//!
//! This module provides the `RenderGrafanaTemplatesStep` which handles rendering
//! of Grafana provisioning templates to the build directory. This step prepares
//! Grafana datasource and dashboard configurations for deployment to the remote host.
//!
//! ## Key Features
//!
//! - Template rendering for Grafana provisioning configurations
//! - Integration with the `GrafanaProjectGenerator` for file generation
//! - Build directory preparation for deployment operations
//! - Comprehensive error handling for template processing
//!
//! ## Usage Context
//!
//! This step is typically executed during the release workflow, after
//! infrastructure provisioning and software installation, to prepare
//! the Grafana provisioning files for deployment.
//!
//! ## Architecture
//!
//! This step follows the three-level architecture:
//! - **Command** (Level 1): `ReleaseCommandHandler` orchestrates the release workflow
//! - **Step** (Level 2): This `RenderGrafanaTemplatesStep` handles template rendering
//! - The templates are rendered locally, no remote action is needed

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{info, instrument};

use crate::domain::environment::Environment;
use crate::domain::template::TemplateManager;
use crate::infrastructure::templating::grafana::template::renderer::{
    GrafanaProjectGenerator, GrafanaProjectGeneratorError,
};
use crate::infrastructure::templating::grafana::template::GrafanaContext;

/// Step that renders Grafana provisioning templates to the build directory
///
/// This step handles the preparation of Grafana provisioning configuration files
/// by rendering templates to the build directory. The rendered files are
/// then ready to be deployed to the remote host.
pub struct RenderGrafanaTemplatesStep<S> {
    environment: Arc<Environment<S>>,
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
}

impl<S> RenderGrafanaTemplatesStep<S> {
    /// Creates a new `RenderGrafanaTemplatesStep`
    ///
    /// # Arguments
    ///
    /// * `environment` - The deployment environment
    /// * `template_manager` - The template manager for accessing templates
    /// * `build_dir` - The build directory where templates will be rendered
    #[must_use]
    pub fn new(
        environment: Arc<Environment<S>>,
        template_manager: Arc<TemplateManager>,
        build_dir: PathBuf,
    ) -> Self {
        Self {
            environment,
            template_manager,
            build_dir,
        }
    }

    /// Execute the template rendering step
    ///
    /// This will render Grafana provisioning templates to the build directory if Grafana
    /// configuration is present in the environment.
    ///
    /// # Returns
    ///
    /// Returns the path to the Grafana provisioning build directory on success, or `None`
    /// if Grafana is not configured.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Template rendering fails
    /// * Directory creation fails
    /// * File writing fails
    #[instrument(
        name = "render_grafana_templates",
        skip_all,
        fields(
            step_type = "rendering",
            template_type = "grafana",
            build_dir = %self.build_dir.display()
        )
    )]
    pub fn execute(&self) -> Result<Option<PathBuf>, GrafanaProjectGeneratorError> {
        // Check if Grafana is configured
        if self.environment.context().user_inputs.grafana.is_none() {
            info!(
                step = "render_grafana_templates",
                status = "skipped",
                reason = "grafana_not_configured",
                "Skipping Grafana template rendering - not configured"
            );
            return Ok(None);
        }

        // Check if Prometheus is configured (required for datasource)
        let Some(prometheus_config) = &self.environment.context().user_inputs.prometheus else {
            info!(
                step = "render_grafana_templates",
                status = "skipped",
                reason = "prometheus_not_configured",
                "Skipping Grafana template rendering - Prometheus datasource requires Prometheus to be configured"
            );
            return Ok(None);
        };

        info!(
            step = "render_grafana_templates",
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            "Rendering Grafana provisioning templates"
        );

        let generator =
            GrafanaProjectGenerator::new(&self.build_dir, self.template_manager.clone());

        // Build context from Prometheus config
        let context = GrafanaContext::new(prometheus_config.scrape_interval_in_secs());
        generator.render(&context)?;

        let grafana_build_dir = self.build_dir.join("grafana/provisioning");

        info!(
            step = "render_grafana_templates",
            grafana_build_dir = %grafana_build_dir.display(),
            status = "success",
            "Grafana provisioning templates rendered successfully"
        );

        Ok(Some(grafana_build_dir))
    }
}
