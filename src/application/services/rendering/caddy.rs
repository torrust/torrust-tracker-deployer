//! Caddy template rendering service
//!
//! This service handles rendering of Caddy TLS proxy configuration templates,
//! including automatic extraction of TLS-enabled services from tracker configuration.

use std::path::PathBuf;
use std::sync::Arc;

use tracing::{info, instrument};

use crate::domain::TemplateManager;
use crate::infrastructure::templating::caddy::{
    CaddyContext, CaddyProjectGenerator, CaddyProjectGeneratorError, CaddyService,
};
use crate::infrastructure::templating::TemplateMetadata;
use crate::shared::Clock;

use crate::domain::environment::user_inputs::UserInputs;

/// Service for rendering Caddy TLS proxy templates
///
/// This service encapsulates the logic for building Caddy contexts from
/// user configuration, including:
/// - Extracting TLS-enabled services (Tracker API, HTTP Trackers, Health Check API, Grafana)
/// - Building `CaddyContext` with Let's Encrypt configuration
/// - Conditional rendering (only when HTTPS + TLS services are configured)
pub struct CaddyTemplateRenderingService {
    templates_dir: PathBuf,
    build_dir: PathBuf,
    clock: Arc<dyn Clock>,
}

impl CaddyTemplateRenderingService {
    /// Create a new service with explicit dependencies
    ///
    /// # Arguments
    ///
    /// * `templates_dir` - Directory containing template source files
    /// * `build_dir` - Directory where rendered templates will be written
    /// * `clock` - Clock service for timestamps
    #[must_use]
    pub fn from_paths(templates_dir: PathBuf, build_dir: PathBuf, clock: Arc<dyn Clock>) -> Self {
        Self {
            templates_dir,
            build_dir,
            clock,
        }
    }

    /// Render Caddy templates if HTTPS and TLS services are configured
    ///
    /// This method builds the complete Caddy context by extracting all
    /// TLS-enabled services from the user configuration. Returns `None`
    /// if HTTPS is not configured or no services have TLS enabled.
    ///
    /// # Arguments
    ///
    /// * `user_inputs` - Complete user configuration
    ///
    /// # Returns
    ///
    /// `Some(PathBuf)` with path to the rendered Caddy build directory if
    /// HTTPS + TLS services are configured, or `None` if Caddy should not
    /// be deployed.
    ///
    /// # Errors
    ///
    /// Returns error if template rendering fails
    #[instrument(
        name = "caddy_rendering_service",
        skip_all,
        fields(
            templates_dir = %self.templates_dir.display(),
            build_dir = %self.build_dir.display()
        )
    )]
    pub fn render(
        &self,
        user_inputs: &UserInputs,
    ) -> Result<Option<PathBuf>, CaddyTemplateRenderingServiceError> {
        // Check if HTTPS is configured
        let Some(https_config) = user_inputs.https() else {
            info!(
                reason = "https_not_configured",
                "Skipping Caddy template rendering - HTTPS not configured"
            );
            return Ok(None);
        };

        // Build CaddyContext from environment configuration
        let caddy_context = self.build_caddy_context(user_inputs, https_config);

        // Check if any service has TLS configured
        if !caddy_context.has_any_tls() {
            info!(
                reason = "no_tls_services",
                "Skipping Caddy template rendering - no services have TLS configured"
            );
            return Ok(None);
        }

        info!(
            templates_dir = %self.templates_dir.display(),
            build_dir = %self.build_dir.display(),
            admin_email = %https_config.admin_email(),
            use_staging = https_config.use_staging(),
            "Rendering Caddy configuration templates"
        );

        let template_manager = Arc::new(TemplateManager::new(self.templates_dir.clone()));
        let generator = CaddyProjectGenerator::new(&self.build_dir, template_manager);

        generator
            .render(&caddy_context)
            .map_err(CaddyTemplateRenderingServiceError::RenderingFailed)?;

        let caddy_build_dir = self.build_dir.join("caddy");

        info!(
            caddy_build_dir = %caddy_build_dir.display(),
            "Caddy templates rendered successfully"
        );

        Ok(Some(caddy_build_dir))
    }

    /// Build a `CaddyContext` from the user configuration
    ///
    /// Extracts TLS-enabled services from tracker config and builds
    /// the context with pre-extracted ports.
    fn build_caddy_context(
        &self,
        user_inputs: &UserInputs,
        https_config: &crate::domain::https::HttpsConfig,
    ) -> CaddyContext {
        let tracker = user_inputs.tracker();

        let metadata = TemplateMetadata::new(self.clock.now());

        let mut context = CaddyContext::new(
            metadata,
            https_config.admin_email(),
            https_config.use_staging(),
        );

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

/// Errors that can occur during Caddy template rendering
#[derive(Debug, thiserror::Error)]
pub enum CaddyTemplateRenderingServiceError {
    /// Template rendering failed
    #[error("Caddy template rendering failed: {0}")]
    RenderingFailed(#[from] CaddyProjectGeneratorError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use crate::domain::environment::testing::EnvironmentTestBuilder;
    use crate::shared::SystemClock;

    #[test]
    fn it_should_create_service_with_from_paths() {
        let templates_dir = TempDir::new().expect("Failed to create temp dir");
        let build_dir = TempDir::new().expect("Failed to create temp dir");
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);

        let service = CaddyTemplateRenderingService::from_paths(
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
            clock,
        );

        assert_eq!(service.templates_dir, templates_dir.path());
        assert_eq!(service.build_dir, build_dir.path());
    }

    #[test]
    fn it_should_return_none_when_https_not_configured() {
        let templates_dir = TempDir::new().expect("Failed to create temp dir");
        let build_dir = TempDir::new().expect("Failed to create temp dir");
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);

        let service = CaddyTemplateRenderingService::from_paths(
            templates_dir.path().to_path_buf(),
            build_dir.path().to_path_buf(),
            clock,
        );

        let (environment, _, _, _temp_dir) =
            EnvironmentTestBuilder::new().build_with_custom_paths();
        let user_inputs = &environment.context().user_inputs;

        let result = service.render(user_inputs);

        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // TODO: Add test cases for HTTPS + TLS configured when EnvironmentTestBuilder supports it
}
