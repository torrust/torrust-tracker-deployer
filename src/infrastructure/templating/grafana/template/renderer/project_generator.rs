//! Grafana Project Generator
//!
//! Orchestrates the rendering of Grafana provisioning configuration templates following
//! the Project Generator pattern.
//!
//! ## Architecture
//!
//! This follows the three-layer Project Generator pattern:
//! - **Context** (`GrafanaContext`) - Defines variables needed by templates
//! - **Renderer** - Renders .tera templates with context
//! - **`ProjectGenerator`** (this file) - Orchestrates all renderers
//!
//! ## Data Flow
//!
//! Prometheus Config → `GrafanaContext` → Template Rendering → Provisioning Files

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use crate::domain::template::{TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::grafana::template::GrafanaContext;

/// Errors that can occur during Grafana project generation
#[derive(Error, Debug)]
pub enum GrafanaProjectGeneratorError {
    /// Failed to create the build directory
    #[error("Failed to create build directory '{directory}': {source}")]
    DirectoryCreationFailed {
        directory: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to load template
    #[error("Failed to load Grafana template: {0}")]
    TemplateLoadFailed(#[from] TemplateManagerError),

    /// Failed to render Grafana provisioning template
    #[error("Failed to render Grafana datasource template: {0}")]
    TemplateRenderFailed(#[from] tera::Error),

    /// Failed to write rendered template to file
    #[error("Failed to write datasource file '{path}': {source}")]
    FileWriteFailed {
        path: String,
        #[source]
        source: std::io::Error,
    },
}

/// Orchestrates Grafana provisioning configuration template rendering
///
/// This is the Project Generator that coordinates all Grafana template rendering.
/// It follows the standard pattern:
/// 1. Create build directory structure
/// 2. Build `GrafanaContext` from configuration
/// 3. Render datasource template (prometheus.yml.tera)
/// 4. Write rendered content to build directory
pub struct GrafanaProjectGenerator {
    build_dir: PathBuf,
    template_manager: Arc<TemplateManager>,
}

impl GrafanaProjectGenerator {
    /// Relative path for Grafana provisioning files within build directory
    const GRAFANA_BUILD_PATH: &'static str = "grafana/provisioning";

    /// Template file name for Prometheus datasource configuration
    const DATASOURCE_TEMPLATE_NAME: &'static str =
        "grafana/provisioning/datasources/prometheus.yml.tera";

    /// Output file name for rendered datasource configuration
    const DATASOURCE_OUTPUT_NAME: &'static str = "datasources/prometheus.yml";

    /// Creates a new Grafana project generator
    ///
    /// # Arguments
    ///
    /// * `build_dir` - The destination directory where templates will be rendered
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new<P: AsRef<Path>>(build_dir: P, template_manager: Arc<TemplateManager>) -> Self {
        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
            template_manager,
        }
    }

    /// Renders Grafana provisioning configuration templates to the build directory
    ///
    /// This method:
    /// 1. Creates the build directory structure for Grafana provisioning
    /// 2. Renders prometheus.yml.tera datasource template with the provided context
    /// 3. Writes the rendered content to datasources/prometheus.yml
    ///
    /// # Arguments
    ///
    /// * `context` - Context containing Prometheus scrape interval
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Build directory creation fails
    /// - Template loading fails
    /// - Template rendering fails
    /// - Writing output file fails
    #[instrument(
        name = "grafana_project_generator_render",
        skip(self, context),
        fields(
            build_dir = %self.build_dir.display()
        )
    )]
    pub fn render(&self, context: &GrafanaContext) -> Result<(), GrafanaProjectGeneratorError> {
        // Create build directory for Grafana provisioning
        let grafana_build_dir = self.build_dir.join(Self::GRAFANA_BUILD_PATH);
        let datasources_dir = grafana_build_dir.join("datasources");

        fs::create_dir_all(&datasources_dir).map_err(|source| {
            GrafanaProjectGeneratorError::DirectoryCreationFailed {
                directory: datasources_dir.display().to_string(),
                source,
            }
        })?;

        // Render datasource template
        // 1. Load template from template manager
        let template_path = self
            .template_manager
            .get_template_path(Self::DATASOURCE_TEMPLATE_NAME)?;

        // 2. Read template content
        let template_content = fs::read_to_string(&template_path).map_err(|source| {
            GrafanaProjectGeneratorError::FileWriteFailed {
                path: template_path.display().to_string(),
                source,
            }
        })?;

        // 3. Render template with context
        let mut tera = tera::Tera::default();
        tera.add_raw_template(Self::DATASOURCE_TEMPLATE_NAME, &template_content)?;
        let rendered_content = tera.render(
            Self::DATASOURCE_TEMPLATE_NAME,
            &tera::Context::from_serialize(context)?,
        )?;

        // Write rendered datasource configuration
        let output_path = grafana_build_dir.join(Self::DATASOURCE_OUTPUT_NAME);
        fs::write(&output_path, rendered_content).map_err(|source| {
            GrafanaProjectGeneratorError::FileWriteFailed {
                path: output_path.display().to_string(),
                source,
            }
        })?;

        Ok(())
    }
}
