//! Grafana Template Rendering Service
//!
//! This service is responsible for rendering Grafana provisioning templates.
//! It's used by multiple contexts (render command, release steps) to prepare
//! Grafana datasource and dashboard configurations.

use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tracing::info;

use crate::domain::prometheus::PrometheusConfig;
use crate::domain::template::TemplateManager;
use crate::infrastructure::templating::grafana::template::renderer::{
    GrafanaProjectGenerator, GrafanaProjectGeneratorError,
};
use crate::shared::Clock;

/// Errors that can occur during Grafana template rendering
#[derive(Error, Debug)]
pub enum GrafanaTemplateRenderingServiceError {
    /// Template rendering failed
    #[error("Failed to render Grafana templates: {reason}")]
    RenderingFailed {
        /// Detailed reason for the failure
        reason: String,
    },
}

impl From<GrafanaProjectGeneratorError> for GrafanaTemplateRenderingServiceError {
    fn from(error: GrafanaProjectGeneratorError) -> Self {
        Self::RenderingFailed {
            reason: error.to_string(),
        }
    }
}

/// Service for rendering Grafana provisioning templates
///
/// This service encapsulates the logic for rendering Grafana datasource and
/// dashboard configuration files. It's designed to be shared across command
/// handlers and steps that need to prepare Grafana provisioning.
pub struct GrafanaTemplateRenderingService {
    build_dir: PathBuf,
    template_manager: Arc<TemplateManager>,
    clock: Arc<dyn Clock>,
}

impl GrafanaTemplateRenderingService {
    /// Build a `GrafanaTemplateRenderingService` from environment paths
    ///
    /// # Arguments
    ///
    /// * `templates_dir` - Directory containing the source templates
    /// * `build_dir` - Directory where rendered templates will be written
    /// * `clock` - The clock for generating timestamps
    ///
    /// # Returns
    ///
    /// Returns a configured `GrafanaTemplateRenderingService` ready for template rendering
    #[must_use]
    pub fn from_paths(templates_dir: PathBuf, build_dir: PathBuf, clock: Arc<dyn Clock>) -> Self {
        let template_manager = Arc::new(TemplateManager::new(templates_dir));

        Self {
            build_dir,
            template_manager,
            clock,
        }
    }

    /// Render Grafana provisioning templates
    ///
    /// This renders Grafana datasource and dashboard provisioning files to the build directory.
    /// Returns `None` if Grafana or Prometheus is not configured (Prometheus is required as datasource).
    ///
    /// # Arguments
    ///
    /// * `grafana_configured` - Whether Grafana is configured in user inputs
    /// * `prometheus_config` - Prometheus configuration (required for datasource, optional)
    ///
    /// # Returns
    ///
    /// Returns the path to the rendered Grafana provisioning directory, or `None` if not configured
    ///
    /// # Errors
    ///
    /// Returns `GrafanaTemplateRenderingServiceError::RenderingFailed` if template rendering fails.
    pub fn render(
        &self,
        grafana_configured: bool,
        prometheus_config: Option<&PrometheusConfig>,
    ) -> Result<Option<PathBuf>, GrafanaTemplateRenderingServiceError> {
        if !grafana_configured {
            info!(
                reason = "grafana_not_configured",
                "Skipping Grafana template rendering - not configured"
            );
            return Ok(None);
        }

        let Some(prometheus_config) = prometheus_config else {
            info!(
                reason = "prometheus_not_configured",
                "Skipping Grafana template rendering - Prometheus datasource requires Prometheus to be configured"
            );
            return Ok(None);
        };

        info!(
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            "Rendering Grafana provisioning templates"
        );

        let generator = GrafanaProjectGenerator::new(
            &self.build_dir,
            self.template_manager.clone(),
            self.clock.clone(),
        );

        generator.render(prometheus_config)?;

        let grafana_provisioning_dir = self.build_dir.join("storage/grafana/provisioning");

        info!(
            grafana_provisioning_dir = %grafana_provisioning_dir.display(),
            "Grafana provisioning templates rendered successfully"
        );

        Ok(Some(grafana_provisioning_dir))
    }
}
