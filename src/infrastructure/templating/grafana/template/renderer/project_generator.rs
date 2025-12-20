//! Grafana Project Generator
//!
//! Orchestrates the rendering of Grafana provisioning configuration templates following
//! the Project Generator pattern.
//!
//! ## Architecture
//!
//! This follows the three-layer Project Generator pattern:
//! - **Context** (`DatasourceContext`) - Defines variables needed by templates
//! - **Renderer** (`DatasourceRenderer`) - Renders specific .tera templates
//! - **`ProjectGenerator`** (this file) - Orchestrates all renderers and static file copying
//!
//! ## Data Flow
//!
//! Prometheus Config → `DatasourceContext` → Template Rendering → Provisioning Files

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use crate::domain::prometheus::PrometheusConfig;
use crate::domain::template::{TemplateManager, TemplateManagerError};
use crate::infrastructure::templating::grafana::template::{
    renderer::{DatasourceRenderer, DatasourceRendererError},
    DatasourceContext,
};

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

    /// Failed to get template path
    #[error("Failed to get template path: {0}")]
    TemplatePathFailed(#[from] TemplateManagerError),

    /// Failed to render datasource configuration
    #[error("Failed to render datasource configuration: {0}")]
    DatasourceRendererFailed(#[from] DatasourceRendererError),

    /// Failed to copy static files
    #[error("Failed to copy static file from '{from}' to '{to}': {source}")]
    FileCopyFailed {
        from: String,
        to: String,
        #[source]
        source: std::io::Error,
    },
}

/// Orchestrates Grafana provisioning configuration template rendering
///
/// This is the Project Generator that coordinates all Grafana template rendering.
/// It follows the standard pattern:
/// 1. Create build directory structure
/// 2. Render datasource template using `DatasourceRenderer`
/// 3. Copy static dashboard provider configuration
/// 4. Copy static dashboard JSON files
pub struct GrafanaProjectGenerator {
    build_dir: PathBuf,
    template_manager: Arc<TemplateManager>,
    datasource_renderer: DatasourceRenderer,
}

impl GrafanaProjectGenerator {
    /// Relative path for Grafana provisioning files within build directory
    const GRAFANA_BUILD_PATH: &'static str = "grafana/provisioning";

    /// Static dashboard provider configuration file
    const DASHBOARD_PROVIDER_FILE: &'static str = "grafana/provisioning/dashboards/torrust.yml";

    /// Static dashboard JSON directory
    const DASHBOARD_JSON_DIR: &'static str = "grafana/provisioning/dashboards/torrust";

    /// Output directory for dashboard provider configuration
    const DASHBOARD_PROVIDER_OUTPUT_DIR: &'static str = "dashboards";

    /// Output directory for dashboard JSON files
    const DASHBOARD_JSON_OUTPUT_DIR: &'static str = "dashboards/torrust";

    /// Creates a new Grafana project generator
    ///
    /// # Arguments
    ///
    /// * `build_dir` - The destination directory where templates will be rendered
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new<P: AsRef<Path>>(build_dir: P, template_manager: Arc<TemplateManager>) -> Self {
        let datasource_renderer = DatasourceRenderer::new(Arc::clone(&template_manager));

        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
            template_manager,
            datasource_renderer,
        }
    }

    /// Renders Grafana provisioning configuration templates to the build directory
    /// Renders Grafana provisioning configuration templates to the build directory
    ///
    /// This method:
    /// 1. Creates the build directory structure for Grafana provisioning
    /// 2. Renders prometheus.yml.tera datasource template with the provided context
    /// 3. Writes the rendered content to datasources/prometheus.yml
    /// 4. Copies static dashboard provider configuration (dashboards/torrust.yml)
    /// 5. Copies static dashboard JSON files (dashboards/torrust/*.json)
    ///
    /// # Arguments
    ///
    /// * `prometheus_config` - Prometheus configuration containing `scrape_interval`
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Build directory creation fails
    /// - Template loading fails
    /// - Template rendering fails
    /// - Writing output file fails
    /// - Copying static files fails
    #[instrument(
        name = "grafana_project_generator_render",
        skip(self, prometheus_config),
        fields(
            build_dir = %self.build_dir.display()
        )
    )]
    pub fn render(
        &self,
        prometheus_config: &PrometheusConfig,
    ) -> Result<(), GrafanaProjectGeneratorError> {
        // Create build directory structure
        let grafana_build_dir = self.build_dir.join(Self::GRAFANA_BUILD_PATH);
        self.create_directory_structure(&grafana_build_dir)?;

        // Build context from Prometheus config
        let context = Self::build_context(prometheus_config);

        // Render datasource template
        self.render_datasource_template(&context, &grafana_build_dir)?;

        // Copy static dashboard files
        self.copy_dashboard_provider(&grafana_build_dir)?;
        self.copy_dashboard_json_files(&grafana_build_dir)?;

        Ok(())
    }

    /// Creates the directory structure for Grafana provisioning files
    #[allow(clippy::unused_self)]
    fn create_directory_structure(
        &self,
        grafana_build_dir: &Path,
    ) -> Result<(), GrafanaProjectGeneratorError> {
        let datasources_dir = grafana_build_dir.join("datasources");
        let dashboards_dir = grafana_build_dir.join(Self::DASHBOARD_PROVIDER_OUTPUT_DIR);
        let dashboards_torrust_dir = grafana_build_dir.join(Self::DASHBOARD_JSON_OUTPUT_DIR);

        // Create all necessary directories
        for dir in [&datasources_dir, &dashboards_dir, &dashboards_torrust_dir] {
            fs::create_dir_all(dir).map_err(|source| {
                GrafanaProjectGeneratorError::DirectoryCreationFailed {
                    directory: dir.display().to_string(),
                    source,
                }
            })?;
        }

        Ok(())
    }

    /// Builds `DatasourceContext` from Prometheus configuration
    fn build_context(prometheus_config: &PrometheusConfig) -> DatasourceContext {
        DatasourceContext::new(prometheus_config.scrape_interval_in_secs())
    }

    /// Renders the datasource template using `DatasourceRenderer`
    fn render_datasource_template(
        &self,
        context: &DatasourceContext,
        grafana_build_dir: &Path,
    ) -> Result<(), GrafanaProjectGeneratorError> {
        let datasources_dir = grafana_build_dir.join("datasources");
        self.datasource_renderer.render(context, &datasources_dir)?;
        Ok(())
    }

    /// Copies the static dashboard provider configuration file
    fn copy_dashboard_provider(
        &self,
        grafana_build_dir: &Path,
    ) -> Result<(), GrafanaProjectGeneratorError> {
        let provider_source_path = self
            .template_manager
            .get_template_path(Self::DASHBOARD_PROVIDER_FILE)?;
        let provider_dest_path = grafana_build_dir.join(format!(
            "{}/torrust.yml",
            Self::DASHBOARD_PROVIDER_OUTPUT_DIR
        ));

        fs::copy(&provider_source_path, &provider_dest_path).map_err(|source| {
            GrafanaProjectGeneratorError::FileCopyFailed {
                from: provider_source_path.display().to_string(),
                to: provider_dest_path.display().to_string(),
                source,
            }
        })?;

        Ok(())
    }

    /// Copies all static dashboard JSON files
    fn copy_dashboard_json_files(
        &self,
        grafana_build_dir: &Path,
    ) -> Result<(), GrafanaProjectGeneratorError> {
        let dashboards_torrust_dir = grafana_build_dir.join(Self::DASHBOARD_JSON_OUTPUT_DIR);

        // List of dashboard JSON files to copy
        let dashboard_files = ["stats.json", "metrics.json"];

        for file_name in &dashboard_files {
            // Build the relative path for the dashboard JSON file
            let relative_path = format!("{}/{}", Self::DASHBOARD_JSON_DIR, file_name);

            // Get the template path (this will extract from embedded resources if needed)
            let source_path = self
                .template_manager
                .get_template_path(&relative_path)
                .map_err(GrafanaProjectGeneratorError::TemplatePathFailed)?;

            let dest_path = dashboards_torrust_dir.join(file_name);

            fs::copy(&source_path, &dest_path).map_err(|source| {
                GrafanaProjectGeneratorError::FileCopyFailed {
                    from: source_path.display().to_string(),
                    to: dest_path.display().to_string(),
                    source,
                }
            })?;
        }

        Ok(())
    }
}
