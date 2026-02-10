//! Caddy template rendering step
//!
//! This module provides the `RenderCaddyTemplatesStep` which handles rendering
//! of Caddy configuration templates to the build directory. This step prepares
//! Caddy Caddyfile for deployment to the remote host.
//!
//! ## Key Features
//!
//! - Template rendering for Caddy TLS proxy configuration
//! - Integration with the `CaddyProjectGenerator` for file generation
//! - Build directory preparation for deployment operations
//! - Automatic extraction of TLS-enabled services from tracker config
//!
//! ## Usage Context
//!
//! This step is typically executed during the release workflow, after
//! infrastructure provisioning and software installation, to prepare
//! the Caddy configuration files for deployment.
//!
//! ## Architecture
//!
//! This step follows the three-level architecture:
//! - **Command** (Level 1): `ReleaseCommandHandler` orchestrates the release workflow
//! - **Step** (Level 2): This `RenderCaddyTemplatesStep` handles template rendering
//! - The templates are rendered locally, no remote action is needed

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{info, instrument};

use crate::application::services::rendering::CaddyTemplateRenderingService;
use crate::application::services::rendering::CaddyTemplateRenderingServiceError;
use crate::domain::environment::Environment;
use crate::shared::clock::Clock;

/// Step that renders Caddy templates to the build directory
///
/// This step handles the preparation of Caddy configuration files
/// by rendering templates to the build directory. The rendered files are
/// then ready to be deployed to the remote host.
///
/// Caddy is only rendered when:
/// 1. HTTPS configuration is present in the environment
/// 2. At least one service has TLS configured
pub struct RenderCaddyTemplatesStep<S> {
    environment: Arc<Environment<S>>,
    templates_dir: PathBuf,
    build_dir: PathBuf,
    clock: Arc<dyn Clock>,
}

impl<S> RenderCaddyTemplatesStep<S> {
    /// Creates a new `RenderCaddyTemplatesStep`
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
    /// This will render Caddy templates to the build directory if HTTPS
    /// configuration is present in the environment and at least one service
    /// has TLS configured.
    ///
    /// # Returns
    ///
    /// Returns the path to the Caddy build directory on success, or `None`
    /// if HTTPS/TLS is not configured.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Template rendering fails
    /// * Directory creation fails
    /// * File writing fails
    #[instrument(
        name = "render_caddy_templates",
        skip_all,
        fields(
            step_type = "rendering",
            template_type = "caddy",
            build_dir = %self.build_dir.display()
        )
    )]
    pub fn execute(&self) -> Result<Option<PathBuf>, CaddyTemplateRenderingServiceError> {
        // Check if HTTPS is configured
        if self.environment.context().user_inputs.https().is_none() {
            info!(
                step = "render_caddy_templates",
                status = "skipped",
                reason = "https_not_configured",
                "Skipping Caddy template rendering - HTTPS not configured"
            );
            return Ok(None);
        }

        info!(
            step = "render_caddy_templates",
            templates_dir = %self.templates_dir.display(),
            build_dir = %self.build_dir.display(),
            "Rendering Caddy configuration templates"
        );

        let service = CaddyTemplateRenderingService::from_paths(
            self.templates_dir.clone(),
            self.build_dir.clone(),
            self.clock.clone(),
        );

        let user_inputs = &self.environment.context().user_inputs;
        let Some(caddy_build_dir) = service.render(user_inputs)? else {
            info!(
                step = "render_caddy_templates",
                status = "skipped",
                reason = "no_tls_services",
                "Skipping Caddy template rendering - no services have TLS configured"
            );
            return Ok(None);
        };

        info!(
            step = "render_caddy_templates",
            caddy_build_dir = %caddy_build_dir.display(),
            status = "success",
            "Caddy templates rendered successfully"
        );

        Ok(Some(caddy_build_dir))
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::domain::environment::testing::EnvironmentTestBuilder;
    use crate::shared::clock::SystemClock;

    #[test]
    fn it_should_create_render_caddy_templates_step() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let clock = Arc::new(SystemClock);
        let step = RenderCaddyTemplatesStep::new(
            environment.clone(),
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
            clock,
        );

        assert_eq!(step.build_dir, build_dir.path());
        assert_eq!(step.templates_dir, templates_dir.path());
    }

    #[test]
    fn it_should_skip_rendering_when_https_not_configured() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        // Build environment without HTTPS config (default)
        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let clock = Arc::new(SystemClock);
        let step = RenderCaddyTemplatesStep::new(
            environment,
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
            clock,
        );

        let result = step.execute();
        assert!(result.is_ok(), "Should succeed when HTTPS not configured");
        assert!(
            result.unwrap().is_none(),
            "Should return None when HTTPS not configured"
        );
    }
}
