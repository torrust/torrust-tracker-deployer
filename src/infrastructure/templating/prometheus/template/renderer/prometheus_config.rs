//! Prometheus configuration renderer
//!
//! Renders prometheus.yml.tera template using `PrometheusContext` and `PrometheusTemplate` wrappers.

use std::path::Path;
use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use crate::domain::template::{TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::prometheus::template::wrapper::prometheus_config::{
    template::PrometheusTemplateError, PrometheusContext, PrometheusTemplate,
};

/// Errors that can occur during Prometheus configuration rendering
#[derive(Error, Debug)]
pub enum PrometheusConfigRendererError {
    /// Failed to get template path from template manager
    #[error("Failed to get template path for 'prometheus.yml.tera': {0}")]
    TemplatePathFailed(#[from] TemplateManagerError),

    /// Failed to read template file
    #[error("Failed to read template file at '{path}': {source}")]
    TemplateReadFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to create or render template
    #[error("Failed to process Prometheus template: {0}")]
    TemplateProcessingFailed(#[from] PrometheusTemplateError),
}

/// Renders prometheus.yml.tera template to prometheus.yml configuration file
///
/// This renderer follows the Project Generator pattern:
/// 1. Loads prometheus.yml.tera from the template manager
/// 2. Creates a `PrometheusTemplate` with `PrometheusContext`
/// 3. Renders the template to an output file
///
/// The `PrometheusContext` contains:
/// - `scrape_interval`: How often to scrape metrics (from prometheus config)
/// - `api_token`: Tracker HTTP API admin token (for authentication)
/// - `api_port`: Tracker HTTP API port (where metrics are exposed)
pub struct PrometheusConfigRenderer {
    template_manager: Arc<TemplateManager>,
}

impl PrometheusConfigRenderer {
    const PROMETHEUS_TEMPLATE_PATH: &'static str = "prometheus/prometheus.yml.tera";

    /// Creates a new Prometheus config renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to load templates from
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>) -> Self {
        Self { template_manager }
    }

    /// Renders the Prometheus configuration to a file
    ///
    /// # Arguments
    ///
    /// * `context` - The rendering context with `scrape_interval`, `api_token`, `api_port`
    /// * `output_dir` - Directory where prometheus.yml will be written
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
        context: &PrometheusContext,
        output_dir: &Path,
    ) -> Result<(), PrometheusConfigRendererError> {
        // 1. Load template from template manager
        let template_path = self
            .template_manager
            .get_template_path(Self::PROMETHEUS_TEMPLATE_PATH)?;

        // 2. Read template content
        let template_content = std::fs::read_to_string(&template_path).map_err(|source| {
            PrometheusConfigRendererError::TemplateReadFailed {
                path: template_path.display().to_string(),
                source,
            }
        })?;

        // 3. Create PrometheusTemplate with context
        let template = PrometheusTemplate::new(template_content, context.clone())?;

        // 4. Render to output file
        let output_path = output_dir.join("prometheus.yml");
        template.render_to_file(&output_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_template_manager() -> Arc<TemplateManager> {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let templates_dir = temp_dir.path().join("templates");
        let prometheus_dir = templates_dir.join("prometheus");

        fs::create_dir_all(&prometheus_dir).expect("Failed to create prometheus dir");

        let template_content = r#"global:
  scrape_interval: {{ scrape_interval }}s

scrape_configs:
  - job_name: "tracker_stats"
    metrics_path: "/api/v1/stats"
    params:
      token: ["{{ api_token }}"]
      format: ["prometheus"]
    static_configs:
      - targets: ["tracker:{{ api_port }}"]
"#;

        fs::write(prometheus_dir.join("prometheus.yml.tera"), template_content)
            .expect("Failed to write template");

        // Prevent temp_dir from being dropped
        std::mem::forget(temp_dir);

        Arc::new(TemplateManager::new(templates_dir))
    }

    #[test]
    fn it_should_render_prometheus_template_successfully() {
        let template_manager = create_test_template_manager();
        let renderer = PrometheusConfigRenderer::new(template_manager);

        let context = PrometheusContext::new(15, "test_token".to_string(), 1212);

        let temp_dir = TempDir::new().expect("Failed to create temp output dir");
        let output_dir = temp_dir.path();

        renderer
            .render(&context, output_dir)
            .expect("Failed to render Prometheus template");

        let output_file = output_dir.join("prometheus.yml");
        assert!(output_file.exists(), "prometheus.yml should be created");

        let file_content = fs::read_to_string(&output_file).expect("Failed to read prometheus.yml");
        assert!(file_content.contains("scrape_interval: 15s"));
        assert!(file_content.contains(r#"token: ["test_token"]"#));
        assert!(file_content.contains(r#"targets: ["tracker:1212"]"#));
    }

    #[test]
    fn it_should_substitute_all_template_variables() {
        let template_manager = create_test_template_manager();
        let renderer = PrometheusConfigRenderer::new(template_manager);

        let context = PrometheusContext::new(30, "admin_token_123".to_string(), 8080);

        let temp_dir = TempDir::new().expect("Failed to create temp output dir");
        let output_dir = temp_dir.path();

        renderer
            .render(&context, output_dir)
            .expect("Failed to render Prometheus template");

        let file_content =
            fs::read_to_string(output_dir.join("prometheus.yml")).expect("Failed to read file");

        // Verify all variables were substituted
        assert!(file_content.contains("scrape_interval: 30s"));
        assert!(file_content.contains(r#"token: ["admin_token_123"]"#));
        assert!(file_content.contains(r#"targets: ["tracker:8080"]"#));

        // Verify no unrendered template tags remain
        assert!(!file_content.contains("{{"));
        assert!(!file_content.contains("}}"));
    }

    #[test]
    fn it_should_use_embedded_template_when_external_not_found() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let templates_dir = temp_dir.path().join("templates");
        fs::create_dir_all(&templates_dir).expect("Failed to create templates dir");

        let template_manager = Arc::new(TemplateManager::new(&templates_dir));
        let renderer = PrometheusConfigRenderer::new(template_manager);

        let context = PrometheusContext::new(15, "token".to_string(), 1212);
        let output_dir = temp_dir.path();

        let result = renderer.render(&context, output_dir);
        assert!(
            result.is_ok(),
            "Should use embedded template when external template not found"
        );
    }
}
