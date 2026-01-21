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

use crate::domain::environment::Environment;
use crate::domain::template::TemplateManager;
use crate::infrastructure::templating::caddy::{
    CaddyContext, CaddyProjectGenerator, CaddyProjectGeneratorError, CaddyService,
};

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
    template_manager: Arc<TemplateManager>,
    build_dir: PathBuf,
}

impl<S> RenderCaddyTemplatesStep<S> {
    /// Creates a new `RenderCaddyTemplatesStep`
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
    pub fn execute(&self) -> Result<Option<PathBuf>, CaddyProjectGeneratorError> {
        // Check if HTTPS is configured
        let Some(https_config) = self.environment.context().user_inputs.https() else {
            info!(
                step = "render_caddy_templates",
                status = "skipped",
                reason = "https_not_configured",
                "Skipping Caddy template rendering - HTTPS not configured"
            );
            return Ok(None);
        };

        // Build CaddyContext from environment configuration
        let caddy_context = self.build_caddy_context(https_config);

        // Check if any service has TLS configured
        if !caddy_context.has_any_tls() {
            info!(
                step = "render_caddy_templates",
                status = "skipped",
                reason = "no_tls_services",
                "Skipping Caddy template rendering - no services have TLS configured"
            );
            return Ok(None);
        }

        info!(
            step = "render_caddy_templates",
            templates_dir = %self.template_manager.templates_dir().display(),
            build_dir = %self.build_dir.display(),
            admin_email = %https_config.admin_email(),
            use_staging = https_config.use_staging(),
            "Rendering Caddy configuration templates"
        );

        let generator = CaddyProjectGenerator::new(&self.build_dir, self.template_manager.clone());

        generator.render(&caddy_context)?;

        let caddy_build_dir = self.build_dir.join("caddy");

        info!(
            step = "render_caddy_templates",
            caddy_build_dir = %caddy_build_dir.display(),
            status = "success",
            "Caddy templates rendered successfully"
        );

        Ok(Some(caddy_build_dir))
    }

    /// Build a `CaddyContext` from the environment configuration
    ///
    /// Extracts TLS-enabled services from tracker config and builds
    /// the context with pre-extracted ports.
    fn build_caddy_context(
        &self,
        https_config: &crate::domain::https::HttpsConfig,
    ) -> CaddyContext {
        let user_inputs = &self.environment.context().user_inputs;
        let tracker = user_inputs.tracker();

        let mut context = CaddyContext::new(https_config.admin_email(), https_config.use_staging());

        // Add Tracker HTTP API if TLS configured
        if let Some(tls_config) = tracker.http_api_tls_domain() {
            let port = tracker.http_api_port();
            context = context.with_tracker_api(CaddyService::new(tls_config, port));
        }

        // Add HTTP Trackers with TLS configured
        for (domain, port) in tracker.http_trackers_with_tls() {
            context = context.with_http_tracker(CaddyService::new(domain, port));
        }

        // Add Health Check API if TLS configured
        if let Some(tls_domain) = tracker.health_check_api_tls_domain() {
            let port = tracker.health_check_api_port();
            context = context.with_health_check_api(CaddyService::new(tls_domain, port));
        }

        // Add Grafana if TLS configured
        if let Some(grafana) = user_inputs.grafana() {
            if let Some(tls_domain) = grafana.tls_domain() {
                // Grafana default port is 3000
                context = context.with_grafana(CaddyService::new(tls_domain, 3000));
            }
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::domain::environment::testing::EnvironmentTestBuilder;

    #[test]
    fn it_should_create_render_caddy_templates_step() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let step = RenderCaddyTemplatesStep::new(
            environment.clone(),
            template_manager.clone(),
            build_dir.path().to_path_buf(),
        );

        assert_eq!(step.build_dir, build_dir.path());
        assert_eq!(step.template_manager.templates_dir(), templates_dir.path());
    }

    #[test]
    fn it_should_skip_rendering_when_https_not_configured() {
        let templates_dir = TempDir::new().expect("Failed to create templates dir");
        let build_dir = TempDir::new().expect("Failed to create build dir");

        // Build environment without HTTPS config (default)
        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let environment = Arc::new(environment);

        let template_manager = Arc::new(TemplateManager::new(templates_dir.path().to_path_buf()));
        let step = RenderCaddyTemplatesStep::new(
            environment,
            template_manager,
            build_dir.path().to_path_buf(),
        );

        let result = step.execute();
        assert!(result.is_ok(), "Should succeed when HTTPS not configured");
        assert!(
            result.unwrap().is_none(),
            "Should return None when HTTPS not configured"
        );
    }
}
