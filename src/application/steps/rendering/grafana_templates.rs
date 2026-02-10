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

use crate::application::services::rendering::GrafanaTemplateRenderingService;
use crate::application::services::rendering::GrafanaTemplateRenderingServiceError;
use crate::domain::environment::Environment;
use crate::shared::clock::Clock;

/// Step that renders Grafana provisioning templates to the build directory
///
/// This step handles the preparation of Grafana provisioning configuration files
/// by rendering templates to the build directory. The rendered files are
/// then ready to be deployed to the remote host.
pub struct RenderGrafanaTemplatesStep<S> {
    environment: Arc<Environment<S>>,
    templates_dir: PathBuf,
    build_dir: PathBuf,
    clock: Arc<dyn Clock>,
}

impl<S> RenderGrafanaTemplatesStep<S> {
    /// Creates a new `RenderGrafanaTemplatesStep`
    ///
    /// # Arguments
    ///
    /// * `environment` - The deployment environment
    /// * `templates_dir` - The templates directory
    /// * `build_dir` - The build directory where templates will be rendered
    /// * `clock` - Clock service for generating timestamps
    #[must_use]
    pub fn new(
        environment: Arc<Environment<S>>,
        templates_dir: PathBuf,
        build_dir: PathBuf,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            environment,
            templates_dir,
            build_dir,
            clock,
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
    pub fn execute(&self) -> Result<Option<PathBuf>, GrafanaTemplateRenderingServiceError> {
        let grafana_configured = self.environment.context().user_inputs.grafana().is_some();
        let prometheus_config = self.environment.context().user_inputs.prometheus();

        // Check if Grafana is configured
        if !grafana_configured {
            info!(
                step = "render_grafana_templates",
                status = "skipped",
                reason = "grafana_not_configured",
                "Skipping Grafana template rendering - not configured"
            );
            return Ok(None);
        }

        // Check if Prometheus is configured (required for datasource)
        if prometheus_config.is_none() {
            info!(
                step = "render_grafana_templates",
                status = "skipped",
                reason = "prometheus_not_configured",
                "Skipping Grafana template rendering - Prometheus datasource requires Prometheus to be configured"
            );
            return Ok(None);
        }

        info!(
            step = "render_grafana_templates",
            templates_dir = %self.templates_dir.display(),
            build_dir = %self.build_dir.display(),
            "Rendering Grafana provisioning templates"
        );

        let service = GrafanaTemplateRenderingService::from_paths(
            self.templates_dir.clone(),
            self.build_dir.clone(),
            self.clock.clone(),
        );

        // Render all Grafana provisioning files (datasource + dashboards)
        let Some(grafana_build_dir) = service.render(grafana_configured, prometheus_config)? else {
            return Ok(None);
        };

        info!(
            step = "render_grafana_templates",
            grafana_build_dir = %grafana_build_dir.display(),
            status = "success",
            "Grafana provisioning templates rendered successfully"
        );

        Ok(Some(grafana_build_dir))
    }
}
