//! Datasource configuration renderer
//!
//! Renders prometheus.yml.tera template using `DatasourceContext` and `DatasourceTemplate` wrappers.

use std::path::Path;
use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use crate::domain::template::{TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::grafana::template::wrapper::datasource::{
    template::DatasourceTemplateError, DatasourceContext, DatasourceTemplate,
};

/// Errors that can occur during datasource configuration rendering
#[derive(Error, Debug)]
pub enum DatasourceRendererError {
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
    #[error("Failed to process datasource template: {0}")]
    TemplateProcessingFailed(#[from] DatasourceTemplateError),
}

/// Renders prometheus.yml.tera template to prometheus.yml datasource configuration file
///
/// This renderer follows the Project Generator pattern:
/// 1. Loads prometheus.yml.tera from the template manager
/// 2. Creates a `DatasourceTemplate` with `DatasourceContext`
/// 3. Renders the template to an output file
///
/// The `DatasourceContext` contains:
/// - `prometheus_scrape_interval_in_secs`: Matches Prometheus scrape interval
pub struct DatasourceRenderer {
    template_manager: Arc<TemplateManager>,
}

impl DatasourceRenderer {
    /// Template filename for the datasource Tera template
    const DATASOURCE_TEMPLATE_FILE: &'static str = "prometheus.yml.tera";

    /// Output filename for the rendered datasource config file
    const DATASOURCE_OUTPUT_FILE: &'static str = "prometheus.yml";

    /// Directory path for Grafana datasource templates
    const DATASOURCE_TEMPLATE_DIR: &'static str = "grafana/provisioning/datasources";

    /// Creates a new datasource renderer
    ///
    /// # Arguments
    ///
    /// * `template_manager` - The template manager to load templates from
    #[must_use]
    pub fn new(template_manager: Arc<TemplateManager>) -> Self {
        Self { template_manager }
    }

    /// Renders the datasource configuration to a file
    ///
    /// # Arguments
    ///
    /// * `context` - The rendering context with `prometheus_scrape_interval_in_secs`
    /// * `output_dir` - Directory where prometheus.yml will be written (e.g., build/grafana/provisioning/datasources)
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
        context: &DatasourceContext,
        output_dir: &Path,
    ) -> Result<(), DatasourceRendererError> {
        // 1. Load template from template manager
        let template_path = self.template_manager.get_template_path(&format!(
            "{}/{}",
            Self::DATASOURCE_TEMPLATE_DIR,
            Self::DATASOURCE_TEMPLATE_FILE
        ))?;

        // 2. Read template content
        let template_content = std::fs::read_to_string(&template_path).map_err(|source| {
            DatasourceRendererError::TemplateReadFailed {
                path: template_path.display().to_string(),
                source,
            }
        })?;

        // 3. Create template wrapper
        let template = DatasourceTemplate::new(template_content, context.clone());

        // 4. Render to output file
        let output_path = output_dir.join(Self::DATASOURCE_OUTPUT_FILE);
        template.render_to_file(&output_path)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn create_test_template_manager() -> Arc<TemplateManager> {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let templates_dir = temp_dir.path().join("templates");
        let datasources_dir = templates_dir.join("grafana/provisioning/datasources");

        fs::create_dir_all(&datasources_dir).expect("Failed to create datasources dir");

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

        fs::write(
            datasources_dir.join("prometheus.yml.tera"),
            template_content,
        )
        .expect("Failed to write template");

        Arc::new(TemplateManager::new(templates_dir))
    }

    #[test]
    fn it_should_render_datasource_config() {
        let template_manager = create_test_template_manager();
        let renderer = DatasourceRenderer::new(template_manager);

        let context = DatasourceContext::new(15);
        let output_dir = TempDir::new().expect("Failed to create output dir");

        renderer
            .render(&context, output_dir.path())
            .expect("Failed to render");

        let output_file = output_dir.path().join("prometheus.yml");
        assert!(output_file.exists());

        let rendered_content = fs::read_to_string(output_file).expect("Failed to read output");
        assert!(rendered_content.contains("timeInterval: \"15s\""));
        assert!(rendered_content.contains("name: Prometheus"));
    }
}
