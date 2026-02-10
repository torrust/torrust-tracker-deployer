//! Prometheus Template Rendering Service
//!
//! This service is responsible for rendering Prometheus configuration templates.
//! It's used by multiple contexts (render command, release steps) to prepare
//! prometheus.yml configuration files.

use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tracing::info;

use crate::domain::prometheus::PrometheusConfig;
use crate::domain::template::TemplateManager;
use crate::domain::tracker::TrackerConfig;
use crate::infrastructure::templating::prometheus::{
    PrometheusProjectGenerator, PrometheusProjectGeneratorError,
};
use crate::shared::Clock;

/// Errors that can occur during Prometheus template rendering
#[derive(Error, Debug)]
pub enum PrometheusTemplateRenderingServiceError {
    /// Template rendering failed
    #[error("Failed to render Prometheus templates: {reason}")]
    RenderingFailed {
        /// Detailed reason for the failure
        reason: String,
    },
}

impl From<PrometheusProjectGeneratorError> for PrometheusTemplateRenderingServiceError {
    fn from(error: PrometheusProjectGeneratorError) -> Self {
        Self::RenderingFailed {
            reason: error.to_string(),
        }
    }
}

/// Service for rendering Prometheus configuration templates
///
/// This service encapsulates the logic for rendering prometheus.yml configuration
/// files. It's designed to be shared across command handlers and steps that need
/// to prepare Prometheus configuration.
pub struct PrometheusTemplateRenderingService {
    build_dir: PathBuf,
    template_manager: Arc<TemplateManager>,
    clock: Arc<dyn Clock>,
}

impl PrometheusTemplateRenderingService {
    /// Build a `PrometheusTemplateRenderingService` from environment paths
    ///
    /// # Arguments
    ///
    /// * `templates_dir` - Directory containing the source templates
    /// * `build_dir` - Directory where rendered templates will be written
    /// * `clock` - The clock for generating timestamps
    ///
    /// # Returns
    ///
    /// Returns a configured `PrometheusTemplateRenderingService` ready for template rendering
    #[must_use]
    pub fn from_paths(templates_dir: PathBuf, build_dir: PathBuf, clock: Arc<dyn Clock>) -> Self {
        let template_manager = Arc::new(TemplateManager::new(templates_dir));

        Self {
            build_dir,
            template_manager,
            clock,
        }
    }

    /// Render Prometheus configuration templates
    ///
    /// This renders the prometheus.yml configuration file to the build directory.
    /// Returns `None` if Prometheus is not configured.
    ///
    /// # Arguments
    ///
    /// * `prometheus_config` - Prometheus configuration from user inputs (optional)
    /// * `tracker_config` - Tracker configuration (needed for API token and port)
    ///
    /// # Returns
    ///
    /// Returns the path to the rendered Prometheus build directory, or `None` if not configured
    ///
    /// # Errors
    ///
    /// Returns `PrometheusTemplateRenderingServiceError::RenderingFailed` if template rendering fails.
    pub fn render(
        &self,
        prometheus_config: Option<&PrometheusConfig>,
        tracker_config: &TrackerConfig,
    ) -> Result<Option<PathBuf>, PrometheusTemplateRenderingServiceError> {
        let Some(prometheus_config) = prometheus_config else {
            info!(
                reason = "prometheus_not_configured",
                "Skipping Prometheus template rendering - not configured"
            );
            return Ok(None);
        };

        info!(
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            "Rendering Prometheus configuration templates"
        );

        let generator = PrometheusProjectGenerator::new(
            &self.build_dir,
            self.template_manager.clone(),
            self.clock.clone(),
        );

        generator.render(prometheus_config, tracker_config)?;

        let prometheus_build_dir = self.build_dir.join("storage/prometheus/etc");

        info!(
            prometheus_build_dir = %prometheus_build_dir.display(),
            "Prometheus configuration templates rendered successfully"
        );

        Ok(Some(prometheus_build_dir))
    }
}
