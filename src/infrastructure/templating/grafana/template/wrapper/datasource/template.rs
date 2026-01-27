//! Datasource template
//!
//! Wraps the prometheus.yml.tera template with rendering capabilities.

use std::path::Path;

use thiserror::Error;

use super::DatasourceContext;

/// Errors that can occur during datasource template processing
#[derive(Error, Debug)]
pub enum DatasourceTemplateError {
    /// Failed to initialize Tera template engine
    #[error("Failed to initialize Tera engine: {0}")]
    TeraInitializationFailed(#[from] tera::Error),

    /// Failed to write rendered template to file
    #[error("Failed to write datasource file to '{path}': {source}")]
    FileWriteFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Datasource template wrapper
///
/// This wraps the prometheus.yml.tera template content and provides rendering capability.
///
/// # Workflow
///
/// 1. Load template content from file
/// 2. Create `DatasourceTemplate` with content and context
/// 3. Call `render_to_file()` to write rendered output
pub struct DatasourceTemplate {
    content: String,
    context: DatasourceContext,
}

impl DatasourceTemplate {
    /// Template name for Tera engine
    const TEMPLATE_NAME: &'static str = "prometheus.yml.tera";

    /// Creates a new datasource template
    ///
    /// # Arguments
    ///
    /// * `template_content` - The raw .tera template content
    /// * `context` - The context with `prometheus_scrape_interval_in_secs`
    #[must_use]
    pub fn new(template_content: String, context: DatasourceContext) -> Self {
        Self {
            content: template_content,
            context,
        }
    }

    /// Renders the template to an output file
    ///
    /// # Arguments
    ///
    /// * `output_path` - Where to write the rendered prometheus.yml file
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Tera initialization fails
    /// - Template rendering fails
    /// - File write fails
    pub fn render_to_file(&self, output_path: &Path) -> Result<(), DatasourceTemplateError> {
        // Initialize Tera engine
        let mut tera = tera::Tera::default();
        tera.add_raw_template(Self::TEMPLATE_NAME, &self.content)?;

        // Render template with context
        let rendered_content = tera.render(
            Self::TEMPLATE_NAME,
            &tera::Context::from_serialize(&self.context)?,
        )?;

        // Write to file
        std::fs::write(output_path, rendered_content).map_err(|source| {
            DatasourceTemplateError::FileWriteFailed {
                path: output_path.display().to_string(),
                source,
            }
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use tempfile::NamedTempFile;

    use super::*;
    use crate::infrastructure::templating::TemplateMetadata;

    fn create_test_metadata() -> TemplateMetadata {
        TemplateMetadata::new(Utc.with_ymd_and_hms(2026, 1, 27, 13, 41, 56).unwrap())
    }

    #[test]
    fn it_should_render_datasource_template() {
        let template_content = r#"apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
    jsonData:
      timeInterval: "{{ prometheus_scrape_interval_in_secs }}s"
      httpMethod: POST
"#;

        let context = DatasourceContext::new(create_test_metadata(), 15);
        let template = DatasourceTemplate::new(template_content.to_string(), context);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        template
            .render_to_file(temp_file.path())
            .expect("Failed to render");

        let rendered_content =
            std::fs::read_to_string(temp_file.path()).expect("Failed to read rendered file");

        assert!(rendered_content.contains(r#"timeInterval: "15s""#));
        assert!(rendered_content.contains("name: Prometheus"));
    }

    #[test]
    fn it_should_handle_different_scrape_intervals() {
        let template_content = r#"timeInterval: "{{ prometheus_scrape_interval_in_secs }}s""#;

        let context = DatasourceContext::new(create_test_metadata(), 30);
        let template = DatasourceTemplate::new(template_content.to_string(), context);

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        template
            .render_to_file(temp_file.path())
            .expect("Failed to render");

        let rendered_content =
            std::fs::read_to_string(temp_file.path()).expect("Failed to read rendered file");

        assert_eq!(rendered_content.trim(), r#"timeInterval: "30s""#);
    }
}
