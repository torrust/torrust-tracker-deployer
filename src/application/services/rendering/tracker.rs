//! Tracker Template Rendering Service
//!
//! This service is responsible for rendering Tracker configuration templates.
//! It's used by multiple contexts (render command, release steps) to prepare
//! tracker.toml configuration files.

use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tracing::info;

use crate::domain::template::TemplateManager;
use crate::domain::tracker::TrackerConfig;
use crate::infrastructure::templating::tracker::{
    TrackerProjectGenerator, TrackerProjectGeneratorError,
};
use crate::shared::Clock;

/// Errors that can occur during Tracker template rendering
#[derive(Error, Debug)]
pub enum TrackerTemplateRenderingServiceError {
    /// Template rendering failed
    #[error("Failed to render Tracker templates: {reason}")]
    RenderingFailed {
        /// Detailed reason for the failure
        reason: String,
    },
}

impl From<TrackerProjectGeneratorError> for TrackerTemplateRenderingServiceError {
    fn from(error: TrackerProjectGeneratorError) -> Self {
        Self::RenderingFailed {
            reason: error.to_string(),
        }
    }
}

/// Service for rendering Tracker configuration templates
///
/// This service encapsulates the logic for rendering tracker.toml configuration
/// files. It's designed to be shared across command handlers and steps that need
/// to prepare Tracker configuration.
pub struct TrackerTemplateRenderingService {
    build_dir: PathBuf,
    template_manager: Arc<TemplateManager>,
    clock: Arc<dyn Clock>,
}

impl TrackerTemplateRenderingService {
    /// Build a `TrackerTemplateRenderingService` from environment paths
    ///
    /// # Arguments
    ///
    /// * `templates_dir` - Directory containing the source templates
    /// * `build_dir` - Directory where rendered templates will be written
    /// * `clock` - The clock for generating timestamps
    ///
    /// # Returns
    ///
    /// Returns a configured `TrackerTemplateRenderingService` ready for template rendering
    #[must_use]
    pub fn from_paths(templates_dir: PathBuf, build_dir: PathBuf, clock: Arc<dyn Clock>) -> Self {
        let template_manager = Arc::new(TemplateManager::new(templates_dir));

        Self {
            build_dir,
            template_manager,
            clock,
        }
    }

    /// Render Tracker configuration templates
    ///
    /// This renders the tracker.toml configuration file to the build directory.
    ///
    /// # Arguments
    ///
    /// * `tracker_config` - Tracker configuration from user inputs
    ///
    /// # Returns
    ///
    /// Returns the path to the rendered tracker build directory
    ///
    /// # Errors
    ///
    /// Returns `TrackerTemplateRenderingServiceError::RenderingFailed` if template rendering fails.
    pub fn render(
        &self,
        tracker_config: &TrackerConfig,
    ) -> Result<PathBuf, TrackerTemplateRenderingServiceError> {
        info!(
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            "Rendering Tracker configuration templates"
        );

        let generator = TrackerProjectGenerator::new(
            &self.build_dir,
            self.template_manager.clone(),
            self.clock.clone(),
        );

        generator.render(Some(tracker_config))?;

        let tracker_build_dir = self.build_dir.join("tracker");

        info!(
            tracker_build_dir = %tracker_build_dir.display(),
            "Tracker configuration templates rendered successfully"
        );

        Ok(tracker_build_dir)
    }
}
