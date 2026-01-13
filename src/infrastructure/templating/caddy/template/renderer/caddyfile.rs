//! Caddyfile configuration renderer
//!
//! Renders Caddyfile.tera template using `CaddyContext`.

use std::path::Path;
use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use crate::domain::template::{TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::caddy::template::wrapper::CaddyContext;

/// Errors that can occur during Caddyfile rendering
#[derive(Error, Debug)]
pub enum CaddyfileRendererError {
    /// Failed to get template path from template manager
    #[error("Failed to get template path for 'Caddyfile.tera': {0}")]
    TemplatePathFailed(#[from] TemplateManagerError),

    /// Failed to read template file
    #[error("Failed to read template file at '{path}': {source}")]
    TemplateReadFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to create Tera instance
    #[error("Failed to create Tera template engine: {0}")]
    TeraCreationFailed(#[source] tera::Error),

    /// Failed to render template
    #[error("Failed to render Caddyfile template: {0}")]
    RenderFailed(#[source] tera::Error),

    /// Failed to write output file
    #[error("Failed to write Caddyfile to '{path}': {source}")]
    OutputWriteFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Renders Caddyfile.tera template to Caddyfile configuration
///
/// This renderer follows the Project Generator pattern:
/// 1. Loads Caddyfile.tera from the template manager
/// 2. Renders the template with `CaddyContext`
/// 3. Writes output to the specified directory
pub struct CaddyfileRenderer {
    template_manager: Arc<TemplateManager>,
}

impl CaddyfileRenderer {
    /// Template filename for the Caddyfile Tera template
    const CADDYFILE_TEMPLATE_FILE: &'static str = "Caddyfile.tera";

    /// Output filename for the rendered Caddyfile
    const CADDYFILE_OUTPUT_FILE: &'static str = "Caddyfile";

    /// Directory path for Caddy templates
    const CADDY_TEMPLATE_DIR: &'static str = "caddy";

    /// Creates a new Caddyfile renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to load templates from
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>) -> Self {
        Self { template_manager }
    }

    /// Renders the Caddyfile to a file
    ///
    /// # Arguments
    ///
    /// * `context` - The rendering context with services and configuration
    /// * `output_dir` - Directory where Caddyfile will be written
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Template file cannot be loaded
    /// - Template file cannot be read
    /// - Template rendering fails
    /// - Output file cannot be written
    #[instrument(skip(self, context), fields(output_dir = %output_dir.display()))]
    pub fn render(
        &self,
        context: &CaddyContext,
        output_dir: &Path,
    ) -> Result<(), CaddyfileRendererError> {
        // 1. Load template from template manager
        let template_path = self.template_manager.get_template_path(&format!(
            "{}/{}",
            Self::CADDY_TEMPLATE_DIR,
            Self::CADDYFILE_TEMPLATE_FILE
        ))?;

        // 2. Read template content
        let template_content = std::fs::read_to_string(&template_path).map_err(|source| {
            CaddyfileRendererError::TemplateReadFailed {
                path: template_path.display().to_string(),
                source,
            }
        })?;

        // 3. Create Tera instance and add template
        let mut tera = tera::Tera::default();
        tera.add_raw_template(Self::CADDYFILE_TEMPLATE_FILE, &template_content)
            .map_err(CaddyfileRendererError::TeraCreationFailed)?;

        // 4. Convert context to Tera context
        let tera_context =
            tera::Context::from_serialize(context).map_err(CaddyfileRendererError::RenderFailed)?;

        // 5. Render template
        let rendered = tera
            .render(Self::CADDYFILE_TEMPLATE_FILE, &tera_context)
            .map_err(CaddyfileRendererError::RenderFailed)?;

        // 6. Write output file
        let output_path = output_dir.join(Self::CADDYFILE_OUTPUT_FILE);
        std::fs::write(&output_path, rendered).map_err(|source| {
            CaddyfileRendererError::OutputWriteFailed {
                path: output_path.display().to_string(),
                source,
            }
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;
    use crate::infrastructure::templating::caddy::template::wrapper::CaddyService;

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
    fn it_should_render_caddyfile_with_all_services() {
        let (template_manager, _temp_dir) = create_test_template_manager();
        let caddyfile_renderer = CaddyfileRenderer::new(template_manager);

        let output_dir = TempDir::new().expect("Failed to create output dir");
        let caddy_ctx = CaddyContext::new("admin@example.com", false)
            .with_tracker_api(CaddyService::new("api.example.com", 1212))
            .with_http_tracker(CaddyService::new("http1.example.com", 7070))
            .with_grafana(CaddyService::new("grafana.example.com", 3000));

        caddyfile_renderer
            .render(&caddy_ctx, output_dir.path())
            .expect("Failed to render");

        let caddyfile_path = output_dir.path().join("Caddyfile");
        assert!(caddyfile_path.exists());

        let file_content = fs::read_to_string(&caddyfile_path).expect("Failed to read");
        assert!(file_content.contains("email admin@example.com"));
        assert!(file_content.contains("api.example.com"));
        assert!(file_content.contains("reverse_proxy tracker:1212"));
        assert!(file_content.contains("http1.example.com"));
        assert!(file_content.contains("reverse_proxy tracker:7070"));
        assert!(file_content.contains("grafana.example.com"));
        assert!(!file_content.contains("acme_ca")); // Not staging
    }

    #[test]
    fn it_should_render_caddyfile_with_staging_ca() {
        let (template_manager, _temp_dir) = create_test_template_manager();
        let caddyfile_renderer = CaddyfileRenderer::new(template_manager);

        let output_dir = TempDir::new().expect("Failed to create output dir");
        let caddy_ctx = CaddyContext::new("admin@example.com", true)
            .with_tracker_api(CaddyService::new("api.example.com", 1212));

        caddyfile_renderer
            .render(&caddy_ctx, output_dir.path())
            .expect("Failed to render");

        let file_content =
            fs::read_to_string(output_dir.path().join("Caddyfile")).expect("Failed to read");
        assert!(file_content.contains("acme-staging-v02.api.letsencrypt.org"));
    }

    #[test]
    fn it_should_render_caddyfile_with_multiple_http_trackers() {
        let (template_manager, _temp_dir) = create_test_template_manager();
        let caddyfile_renderer = CaddyfileRenderer::new(template_manager);

        let output_dir = TempDir::new().expect("Failed to create output dir");
        let caddy_ctx = CaddyContext::new("admin@example.com", false)
            .with_http_tracker(CaddyService::new("http1.example.com", 7070))
            .with_http_tracker(CaddyService::new("http2.example.com", 7071))
            .with_http_tracker(CaddyService::new("http3.example.com", 7072));

        caddyfile_renderer
            .render(&caddy_ctx, output_dir.path())
            .expect("Failed to render");

        let file_content =
            fs::read_to_string(output_dir.path().join("Caddyfile")).expect("Failed to read");
        assert!(file_content.contains("http1.example.com"));
        assert!(file_content.contains("reverse_proxy tracker:7070"));
        assert!(file_content.contains("http2.example.com"));
        assert!(file_content.contains("reverse_proxy tracker:7071"));
        assert!(file_content.contains("http3.example.com"));
        assert!(file_content.contains("reverse_proxy tracker:7072"));
    }

    #[test]
    fn it_should_render_caddyfile_without_optional_services() {
        let (template_manager, _temp_dir) = create_test_template_manager();
        let caddyfile_renderer = CaddyfileRenderer::new(template_manager);

        let output_dir = TempDir::new().expect("Failed to create output dir");
        // Only API, no HTTP trackers or Grafana
        let caddy_ctx = CaddyContext::new("admin@example.com", false)
            .with_tracker_api(CaddyService::new("api.example.com", 1212));

        caddyfile_renderer
            .render(&caddy_ctx, output_dir.path())
            .expect("Failed to render");

        let file_content =
            fs::read_to_string(output_dir.path().join("Caddyfile")).expect("Failed to read");
        assert!(file_content.contains("api.example.com"));
        assert!(!file_content.contains("grafana"));
    }
}
