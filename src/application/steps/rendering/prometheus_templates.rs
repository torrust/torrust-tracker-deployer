//! Prometheus template rendering step
//!
//! This module provides the `RenderPrometheusTemplatesStep` which handles rendering
//! of Prometheus configuration templates to the build directory. This step prepares
//! Prometheus configuration files for deployment to the remote host.
//!
//! ## Key Features
//!
//! - Template rendering for Prometheus configurations
//! - Integration with the `PrometheusProjectGenerator` for file generation
//! - Build directory preparation for deployment operations
//! - Comprehensive error handling for template processing
//!
//! ## Usage Context
//!
//! This step is typically executed during the release workflow, after
//! infrastructure provisioning and software installation, to prepare
//! the Prometheus configuration files for deployment.
//!
//! ## Architecture
//!
//! This step follows the three-level architecture:
//! - **Command** (Level 1): `ReleaseCommandHandler` orchestrates the release workflow
//! - **Step** (Level 2): This `RenderPrometheusTemplatesStep` handles template rendering
//! - The templates are rendered locally, no remote action is needed

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{info, instrument};

use crate::domain::environment::Environment;
use crate::domain::template::TemplateManager;
use crate::infrastructure::templating::prometheus::{
    PrometheusProjectGenerator, PrometheusProjectGeneratorError,
};

/// Step that renders Prometheus templates to the build directory
///
/// This step handles the preparation of Prometheus configuration files
/// by rendering templates to the build directory. The rendered files are
/// then ready to be deployed to the remote host.
pub struct RenderPrometheusTemplatesStep<S> {
    environment: Arc<Environment<S>>,
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
}

impl<S> RenderPrometheusTemplatesStep<S> {
    /// Creates a new `RenderPrometheusTemplatesStep`
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
    /// This will render Prometheus templates to the build directory if Prometheus
    /// configuration is present in the environment.
    ///
    /// # Returns
    ///
    /// Returns the path to the Prometheus build directory on success, or `None`
    /// if Prometheus is not configured.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Template rendering fails
    /// * Directory creation fails
    /// * File writing fails
    #[instrument(
        name = "render_prometheus_templates",
        skip_all,
        fields(
            step_type = "rendering",
            template_type = "prometheus",
            build_dir = %self.build_dir.display()
        )
    )]
    pub fn execute(&self) -> Result<Option<PathBuf>, PrometheusProjectGeneratorError> {
        // Check if Prometheus is configured
        let Some(prometheus_config) = &self.environment.context().user_inputs.prometheus else {
            info!(
                step = "render_prometheus_templates",
                status = "skipped",
                reason = "prometheus_not_configured",
                "Skipping Prometheus template rendering - not configured"
            );
            return Ok(None);
        };

        info!(
            step = "render_prometheus_templates",
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            "Rendering Prometheus configuration templates"
        );

        let generator =
            PrometheusProjectGenerator::new(&self.build_dir, self.template_manager.clone());

        // Extract tracker config for API token and port
        let tracker_config = &self.environment.context().user_inputs.tracker;
        generator.render(prometheus_config, tracker_config)?;

        let prometheus_build_dir = self.build_dir.join("storage/prometheus/etc");

        info!(
            step = "render_prometheus_templates",
            prometheus_build_dir = %prometheus_build_dir.display(),
            status = "success",
            "Prometheus templates rendered successfully"
        );

        Ok(Some(prometheus_build_dir))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::domain::environment::testing::EnvironmentTestBuilder;
    use crate::domain::prometheus::PrometheusConfig;

    #[test]
    fn it_should_create_render_prometheus_templates_step() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let step = RenderPrometheusTemplatesStep::new(
            environment.clone(),
            template_manager.clone(),
            build_dir.path().to_path_buf(),
        );

        assert_eq!(step.build_dir, build_dir.path());
        assert_eq!(step.template_manager.templates_dir(), templates_dir.path());
    }

    #[test]
    fn it_should_skip_rendering_when_prometheus_not_configured() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        // Build environment without Prometheus config
        let (environment, _, _, _temp_dir) = EnvironmentTestBuilder::new()
            .with_prometheus_config(None)
            .build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let step = RenderPrometheusTemplatesStep::new(
            environment,
            template_manager,
            build_dir.path().to_path_buf(),
        );

        let result = step.execute();
        assert!(
            result.is_ok(),
            "Should succeed when Prometheus not configured"
        );
        assert!(
            result.unwrap().is_none(),
            "Should return None when Prometheus not configured"
        );
    }

    #[test]
    fn it_should_render_templates_when_prometheus_configured() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        // Build environment with Prometheus config
        let (environment, _, _, _temp_dir) = EnvironmentTestBuilder::new()
            .with_prometheus_config(Some(PrometheusConfig::new(
                std::num::NonZeroU32::new(30).expect("30 is non-zero"),
            )))
            .build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let step = RenderPrometheusTemplatesStep::new(
            environment,
            template_manager,
            build_dir.path().to_path_buf(),
        );

        let result = step.execute();
        assert!(result.is_ok(), "Should render Prometheus templates");

        let prometheus_build_dir = result.unwrap();
        assert!(
            prometheus_build_dir.is_some(),
            "Should return build directory path"
        );

        let build_dir_path = prometheus_build_dir.unwrap();
        assert!(
            build_dir_path.to_string_lossy().contains("prometheus"),
            "Build directory should contain 'prometheus' in path"
        );
    }
}
