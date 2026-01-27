//! Caddy Project Generator
//!
//! Orchestrates the rendering of Caddy configuration templates following
//! the Project Generator pattern.
//!
//! ## Architecture
//!
//! This follows the three-layer Project Generator pattern:
//! - **Context** (`CaddyContext`) - Defines variables needed by templates
//! - **Renderer** (`CaddyfileRenderer`) - Renders Caddyfile.tera template
//! - **`ProjectGenerator`** (this file) - Orchestrates all renderers
//!
//! ## Data Flow
//!
//! Environment Config → `CaddyContext` (with pre-extracted ports) → Template Rendering

use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use crate::domain::template::TemplateManager;
use crate::infrastructure::templating::caddy::template::{
    renderer::{CaddyfileRenderer, CaddyfileRendererError},
    wrapper::CaddyContext,
};

/// Errors that can occur during Caddy project generation
#[derive(Error, Debug)]
pub enum CaddyProjectGeneratorError {
    /// Failed to create the build directory
    #[error("Failed to create build directory '{directory}': {source}")]
    DirectoryCreationFailed {
        directory: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to render Caddyfile
    #[error("Failed to render Caddyfile: {0}")]
    RendererFailed(#[from] CaddyfileRendererError),

    /// No TLS configuration provided
    #[error("Caddy project generator called but no TLS configuration provided. Use `CaddyContext::has_any_tls()` to check before calling.")]
    NoTlsConfigured,
}

/// Orchestrates Caddy configuration template rendering
///
/// This is the Project Generator that coordinates Caddy template rendering.
/// It follows the standard pattern:
/// 1. Check if any TLS configuration exists
/// 2. Create build directory structure
/// 3. Call `CaddyfileRenderer` to render Caddyfile.tera
///
/// # Conditional Deployment
///
/// Caddy is only deployed when at least one service has TLS configured.
/// Use `CaddyContext::has_any_tls()` to check before calling this generator.
pub struct CaddyProjectGenerator {
    build_dir: PathBuf,
    caddyfile_renderer: CaddyfileRenderer,
}

impl CaddyProjectGenerator {
    /// Default relative path for Caddy configuration files
    const CADDY_BUILD_PATH: &'static str = "caddy";

    /// Creates a new Caddy project generator
    ///
    /// # Arguments
    ///
    /// * `build_dir` - The destination directory where templates will be rendered
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new<P: AsRef<Path>>(build_dir: P, template_manager: Arc<TemplateManager>) -> Self {
        let caddyfile_renderer = CaddyfileRenderer::new(template_manager);

        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
            caddyfile_renderer,
        }
    }

    /// Renders Caddy configuration templates to the build directory
    ///
    /// This method:
    /// 1. Verifies that at least one service has TLS configured
    /// 2. Creates the build directory structure for Caddy config
    /// 3. Renders Caddyfile.tera template with the provided context
    /// 4. Writes the rendered content to Caddyfile
    ///
    /// # Arguments
    ///
    /// * `context` - The `CaddyContext` containing services and email configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No TLS configuration is provided (use `has_any_tls()` first)
    /// - Build directory creation fails
    /// - Template loading fails
    /// - Template rendering fails
    /// - Writing output file fails
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::infrastructure::templating::caddy::{
    ///     CaddyProjectGenerator, CaddyContext, CaddyService,
    /// };
    ///
    /// let generator = CaddyProjectGenerator::new(&build_dir, template_manager);
    ///
    /// let context = CaddyContext::new("admin@example.com", false)
    ///     .with_tracker_api(CaddyService::new("api.example.com", 1212));
    ///
    /// // Only render if TLS is configured
    /// if context.has_any_tls() {
    ///     generator.render(&context)?;
    /// }
    /// ```
    #[instrument(
        name = "caddy_project_generator_render",
        skip(self, context),
        fields(
            build_dir = %self.build_dir.display(),
            has_tls = context.has_any_tls()
        )
    )]
    pub fn render(&self, context: &CaddyContext) -> Result<(), CaddyProjectGeneratorError> {
        // Validate that TLS is configured
        if !context.has_any_tls() {
            return Err(CaddyProjectGeneratorError::NoTlsConfigured);
        }

        // Create build directory for Caddy templates
        let caddy_build_dir = self.build_dir.join(Self::CADDY_BUILD_PATH);
        std::fs::create_dir_all(&caddy_build_dir).map_err(|source| {
            CaddyProjectGeneratorError::DirectoryCreationFailed {
                directory: caddy_build_dir.display().to_string(),
                source,
            }
        })?;

        // Render Caddyfile using CaddyfileRenderer
        self.caddyfile_renderer.render(context, &caddy_build_dir)?;

        Ok(())
    }

    /// Returns the path where Caddy files will be generated
    #[must_use]
    pub fn output_path(&self) -> PathBuf {
        self.build_dir.join(Self::CADDY_BUILD_PATH)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::{TimeZone, Utc};
    use tempfile::TempDir;

    use super::*;
    use crate::infrastructure::templating::caddy::template::wrapper::CaddyService;
    use crate::infrastructure::templating::TemplateMetadata;

    fn create_test_metadata() -> TemplateMetadata {
        TemplateMetadata::new(Utc.with_ymd_and_hms(2026, 1, 27, 13, 41, 56).unwrap())
    }

    fn create_test_template_manager() -> (Arc<TemplateManager>, TempDir) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let templates_dir = temp_dir.path().join("templates");
        let caddy_dir = templates_dir.join("caddy");

        fs::create_dir_all(&caddy_dir).expect("Failed to create caddy dir");

        let template_content = r"# Caddyfile for Torrust Tracker
{
	email {{ admin_email }}
{% if use_staging %}
	acme_ca https://acme-staging-v02.api.letsencrypt.org/directory
{% endif %}
}
{% if tracker_api %}

{{ tracker_api.domain }} {
	reverse_proxy tracker:{{ tracker_api.port }}
}
{% endif %}
{% for http_tracker in http_trackers %}

{{ http_tracker.domain }} {
	reverse_proxy tracker:{{ http_tracker.port }}
}
{% endfor %}
{% if grafana %}

{{ grafana.domain }} {
	reverse_proxy grafana:3000
}
{% endif %}
";

        fs::write(caddy_dir.join("Caddyfile.tera"), template_content)
            .expect("Failed to write template");

        (Arc::new(TemplateManager::new(templates_dir)), temp_dir)
    }

    #[test]
    fn it_should_create_caddy_build_directory() {
        let (template_manager, _temp_dir) = create_test_template_manager();
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let generator = CaddyProjectGenerator::new(build_dir.path(), template_manager);

        let caddy_ctx = CaddyContext::new(create_test_metadata(), "admin@example.com", false)
            .with_tracker_api(CaddyService::new("api.example.com", 1212));

        generator.render(&caddy_ctx).expect("Failed to render");

        let caddy_dir = build_dir.path().join("caddy");
        assert!(caddy_dir.exists());
        assert!(caddy_dir.is_dir());
    }

    #[test]
    fn it_should_render_caddyfile() {
        let (template_manager, _temp_dir) = create_test_template_manager();
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let project_gen = CaddyProjectGenerator::new(build_dir.path(), template_manager);

        let caddy_ctx = CaddyContext::new(create_test_metadata(), "admin@example.com", false)
            .with_tracker_api(CaddyService::new("api.example.com", 1212))
            .with_grafana(CaddyService::new("grafana.example.com", 3000));

        project_gen.render(&caddy_ctx).expect("Failed to render");

        let caddyfile_path = build_dir.path().join("caddy/Caddyfile");
        assert!(caddyfile_path.exists());

        let file_content = fs::read_to_string(&caddyfile_path).expect("Failed to read");
        assert!(file_content.contains("email admin@example.com"));
        assert!(file_content.contains("api.example.com"));
        assert!(file_content.contains("grafana.example.com"));
    }

    #[test]
    fn it_should_fail_when_no_tls_configured() {
        let (template_manager, _temp_dir) = create_test_template_manager();
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let project_gen = CaddyProjectGenerator::new(build_dir.path(), template_manager);

        // Empty context - no TLS configured
        let caddy_ctx = CaddyContext::new(create_test_metadata(), "admin@example.com", false);

        let result = project_gen.render(&caddy_ctx);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CaddyProjectGeneratorError::NoTlsConfigured
        ));
    }

    #[test]
    fn it_should_return_correct_output_path() {
        let (template_manager, _temp_dir) = create_test_template_manager();
        let build_dir = TempDir::new().expect("Failed to create build dir");

        let generator = CaddyProjectGenerator::new(build_dir.path(), template_manager);

        let expected = build_dir.path().join("caddy");
        assert_eq!(generator.output_path(), expected);
    }
}
