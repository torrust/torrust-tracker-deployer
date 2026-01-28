//! Prometheus template wrapper
//!
//! Wraps the prometheus.yml.tera template file with its context for rendering.

use std::path::Path;

use tera::Tera;
use thiserror::Error;

use super::context::PrometheusContext;

/// Errors that can occur during Prometheus template operations
#[derive(Error, Debug)]
pub enum PrometheusTemplateError {
    /// Failed to create Tera instance
    #[error("Failed to create Tera template engine: {0}")]
    TeraCreationFailed(#[from] tera::Error),

    /// Failed to render template
    #[error("Failed to render Prometheus template: {0}")]
    RenderingFailed(String),

    /// Failed to write rendered content to file
    #[error("Failed to write Prometheus configuration to '{path}': {source}")]
    WriteFileFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Wrapper for prometheus.yml template with rendering context
///
/// This type encapsulates the Prometheus configuration template and provides
/// methods to render it with the given context.
///
/// The context contains:
/// - `scrape_interval`: How often Prometheus scrapes metrics
/// - `api_token`: Tracker HTTP API admin token
/// - `api_port`: Tracker HTTP API port
pub struct PrometheusTemplate {
    /// The template content
    content: String,
    /// The rendering context
    context: PrometheusContext,
}

impl PrometheusTemplate {
    /// Creates a new Prometheus template with the given content and context
    ///
    /// # Arguments
    ///
    /// * `content` - The raw template content (prometheus.yml.tera)
    /// * `context` - The rendering context with `scrape_interval`, `api_token`, `api_port`
    ///
    /// # Errors
    ///
    /// Returns an error if the template content is invalid Tera syntax
    pub fn new(
        template_content: String,
        context: PrometheusContext,
    ) -> Result<Self, PrometheusTemplateError> {
        // Validate template syntax by attempting to create a Tera instance
        let mut tera = Tera::default();
        tera.add_raw_template("prometheus.yml", &template_content)?;

        Ok(Self {
            content: template_content,
            context,
        })
    }

    /// Renders the template with the context
    ///
    /// # Returns
    ///
    /// The rendered template content as a String
    ///
    /// # Errors
    ///
    /// Returns an error if template rendering fails
    pub fn render(&self) -> Result<String, PrometheusTemplateError> {
        let mut tera = Tera::default();
        tera.add_raw_template("prometheus.yml", &self.content)
            .map_err(|e| PrometheusTemplateError::RenderingFailed(e.to_string()))?;

        let context = tera::Context::from_serialize(&self.context)
            .map_err(|e| PrometheusTemplateError::RenderingFailed(e.to_string()))?;

        tera.render("prometheus.yml", &context)
            .map_err(|e| PrometheusTemplateError::RenderingFailed(e.to_string()))
    }

    /// Renders the template and writes it to a file
    ///
    /// # Arguments
    ///
    /// * `output_path` - Path where the rendered prometheus.yml should be written
    ///
    /// # Errors
    ///
    /// Returns an error if rendering fails or if writing to the file fails
    pub fn render_to_file(&self, output_path: &Path) -> Result<(), PrometheusTemplateError> {
        let rendered = self.render()?;

        std::fs::write(output_path, rendered).map_err(|source| {
            PrometheusTemplateError::WriteFileFailed {
                path: output_path.display().to_string(),
                source,
            }
        })?;

        Ok(())
    }

    /// Returns the raw template content
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Returns a reference to the rendering context
    #[must_use]
    pub fn context(&self) -> &PrometheusContext {
        &self.context
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use tempfile::TempDir;

    use super::*;

    /// Helper to create test metadata with a fixed timestamp
    fn create_test_metadata() -> crate::infrastructure::templating::metadata::TemplateMetadata {
        let fixed_time = Utc.with_ymd_and_hms(2026, 1, 27, 13, 41, 56).unwrap();
        crate::infrastructure::templating::metadata::TemplateMetadata::new(fixed_time)
    }

    fn sample_template_content() -> String {
        r#"global:
  scrape_interval: {{ scrape_interval }}s

scrape_configs:
  - job_name: "tracker_stats"
    metrics_path: "/api/v1/stats"
    params:
      token: ["{{ api_token }}"]
      format: ["prometheus"]
    static_configs:
      - targets: ["tracker:{{ api_port }}"]
"#
        .to_string()
    }

    #[test]
    fn it_should_create_prometheus_template_successfully() {
        let template_content = sample_template_content();
        let metadata = create_test_metadata();
        let ctx =
            PrometheusContext::new(metadata, "15s".to_string(), "test_token".to_string(), 1212);

        let template = PrometheusTemplate::new(template_content, ctx);
        assert!(template.is_ok());
    }

    #[test]
    fn it_should_fail_with_invalid_template_syntax() {
        let invalid_content = "{{ unclosed".to_string();
        let metadata = create_test_metadata();
        let context =
            PrometheusContext::new(metadata, "15s".to_string(), "token".to_string(), 1212);

        let result = PrometheusTemplate::new(invalid_content, context);
        assert!(result.is_err());
    }

    #[test]
    fn it_should_render_template_with_context() {
        let template_content = sample_template_content();
        let metadata = create_test_metadata();
        let ctx =
            PrometheusContext::new(metadata, "30s".to_string(), "admin_token".to_string(), 8080);

        let template =
            PrometheusTemplate::new(template_content, ctx).expect("Failed to create template");

        let rendered = template.render().expect("Failed to render template");

        assert!(rendered.contains("scrape_interval: 30s"));
        assert!(rendered.contains(r#"token: ["admin_token"]"#));
        assert!(rendered.contains(r#"targets: ["tracker:8080"]"#));
    }

    #[test]
    fn it_should_not_contain_template_syntax_after_rendering() {
        let template_content = sample_template_content();
        let metadata = create_test_metadata();
        let ctx = PrometheusContext::new(metadata, "15s".to_string(), "token".to_string(), 1212);

        let template =
            PrometheusTemplate::new(template_content, ctx).expect("Failed to create template");

        let rendered = template.render().expect("Failed to render template");

        // Verify no unrendered template tags remain
        assert!(!rendered.contains("{{"));
        assert!(!rendered.contains("}}"));
    }

    #[test]
    fn it_should_render_to_file_successfully() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let output_path = temp_dir.path().join("prometheus.yml");

        let template_content = sample_template_content();
        let metadata = create_test_metadata();
        let ctx =
            PrometheusContext::new(metadata, "20s".to_string(), "file_token".to_string(), 9090);

        let template =
            PrometheusTemplate::new(template_content, ctx).expect("Failed to create template");

        template
            .render_to_file(&output_path)
            .expect("Failed to render to file");

        assert!(output_path.exists());

        let file_content =
            std::fs::read_to_string(&output_path).expect("Failed to read output file");

        assert!(file_content.contains("scrape_interval: 20s"));
        assert!(file_content.contains(r#"token: ["file_token"]"#));
        assert!(file_content.contains(r#"targets: ["tracker:9090"]"#));
    }

    #[test]
    fn it_should_provide_access_to_content() {
        let template_content = sample_template_content();
        let metadata = create_test_metadata();
        let ctx = PrometheusContext::new(metadata, "15s".to_string(), "token".to_string(), 1212);

        let template = PrometheusTemplate::new(template_content.clone(), ctx)
            .expect("Failed to create template");

        assert_eq!(template.content(), template_content);
    }

    #[test]
    fn it_should_provide_access_to_context() {
        let template_content = sample_template_content();
        let metadata = create_test_metadata();
        let ctx = PrometheusContext::new(
            metadata,
            "25s".to_string(),
            "context_token".to_string(),
            7070,
        );

        let template = PrometheusTemplate::new(template_content, ctx.clone())
            .expect("Failed to create template");

        assert_eq!(template.context(), &ctx);
    }
}
