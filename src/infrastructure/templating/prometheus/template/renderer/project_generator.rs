//! Prometheus Project Generator
//!
//! Orchestrates the rendering of all Prometheus configuration templates following
//! the Project Generator pattern.
//!
//! ## Architecture
//!
//! This follows the three-layer Project Generator pattern:
//! - **Context** (`PrometheusContext`) - Defines variables needed by templates
//! - **Template** (`PrometheusTemplate`) - Wraps template file with context
//! - **Renderer** (`PrometheusConfigRenderer`) - Renders specific .tera templates
//! - **`ProjectGenerator`** (this file) - Orchestrates all renderers
//!
//! ## Data Flow
//!
//! Environment Config → `PrometheusConfig` → `PrometheusContext` → Template Rendering

use std::path::{Path, PathBuf};
use std::sync::Arc;

use thiserror::Error;
use tracing::instrument;

use crate::domain::prometheus::PrometheusConfig;
use crate::domain::template::TemplateManager;
use crate::domain::tracker::TrackerConfig;
use crate::infrastructure::templating::prometheus::template::{
    renderer::{PrometheusConfigRenderer, PrometheusConfigRendererError},
    PrometheusContext,
};

/// Errors that can occur during Prometheus project generation
#[derive(Error, Debug)]
pub enum PrometheusProjectGeneratorError {
    /// Failed to create the build directory
    #[error("Failed to create build directory '{directory}': {source}")]
    DirectoryCreationFailed {
        directory: String,
        #[source]
        source: std::io::Error,
    },

    /// Failed to render Prometheus configuration
    #[error("Failed to render Prometheus configuration: {0}")]
    RendererFailed(#[from] PrometheusConfigRendererError),

    /// Missing required tracker configuration
    #[error("Tracker configuration is required to extract API token and port for Prometheus")]
    MissingTrackerConfig,
}

/// Orchestrates Prometheus configuration template rendering
///
/// This is the Project Generator that coordinates all Prometheus template rendering.
/// It follows the standard pattern:
/// 1. Create build directory structure
/// 2. Extract data from tracker and Prometheus configs
/// 3. Build `PrometheusContext`
/// 4. Call `PrometheusConfigRenderer` to render prometheus.yml.tera
pub struct PrometheusProjectGenerator {
    build_dir: PathBuf,
    prometheus_renderer: PrometheusConfigRenderer,
}

impl PrometheusProjectGenerator {
    /// Default relative path for Prometheus configuration files
    const PROMETHEUS_BUILD_PATH: &'static str = "prometheus";

    /// Creates a new Prometheus project generator
    ///
    /// # Arguments
    ///
    /// * `build_dir` - The destination directory where templates will be rendered
    /// * `template_manager` - The template manager to source templates from
    #[must_use]
    pub fn new<P: AsRef<Path>>(build_dir: P, template_manager: Arc<TemplateManager>) -> Self {
        let prometheus_renderer = PrometheusConfigRenderer::new(template_manager);

        Self {
            build_dir: build_dir.as_ref().to_path_buf(),
            prometheus_renderer,
        }
    }

    /// Renders Prometheus configuration templates to the build directory
    ///
    /// This method:
    /// 1. Creates the build directory structure for Prometheus config
    /// 2. Extracts API token and port from tracker configuration
    /// 3. Builds `PrometheusContext` with `scrape_interval`, `api_token`, `api_port`
    /// 4. Renders prometheus.yml.tera template
    /// 5. Writes the rendered content to prometheus.yml
    ///
    /// # Arguments
    ///
    /// * `prometheus_config` - Prometheus configuration (`scrape_interval`)
    /// * `tracker_config` - Tracker configuration (needed for API token and port)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Tracker configuration is not provided
    /// - Build directory creation fails
    /// - Template loading fails
    /// - Template rendering fails
    /// - Writing output file fails
    #[instrument(
        name = "prometheus_project_generator_render",
        skip(self, prometheus_config, tracker_config),
        fields(
            build_dir = %self.build_dir.display()
        )
    )]
    pub fn render(
        &self,
        prometheus_config: &PrometheusConfig,
        tracker_config: &TrackerConfig,
    ) -> Result<(), PrometheusProjectGeneratorError> {
        // Create build directory for Prometheus templates
        let prometheus_build_dir = self.build_dir.join(Self::PROMETHEUS_BUILD_PATH);
        std::fs::create_dir_all(&prometheus_build_dir).map_err(|source| {
            PrometheusProjectGeneratorError::DirectoryCreationFailed {
                directory: prometheus_build_dir.display().to_string(),
                source,
            }
        })?;

        // Build PrometheusContext from configurations
        let context = Self::build_context(prometheus_config, tracker_config);

        // Render prometheus.yml using PrometheusConfigRenderer
        self.prometheus_renderer
            .render(&context, &prometheus_build_dir)?;

        Ok(())
    }

    /// Builds `PrometheusContext` from Prometheus and Tracker configurations
    ///
    /// # Arguments
    ///
    /// * `prometheus_config` - Contains `scrape_interval`
    /// * `tracker_config` - Contains HTTP API `admin_token` and `bind_address`
    ///
    /// # Returns
    ///
    /// A `PrometheusContext` with:
    /// - `scrape_interval`: From `prometheus_config.scrape_interval_in_secs`
    /// - `api_token`: From `tracker_config.http_api.admin_token`
    /// - `api_port`: Parsed from `tracker_config.http_api.bind_address`
    fn build_context(
        prometheus_config: &PrometheusConfig,
        tracker_config: &TrackerConfig,
    ) -> PrometheusContext {
        let scrape_interval = prometheus_config.scrape_interval_in_secs().to_string();
        let api_token = tracker_config
            .http_api
            .admin_token
            .expose_secret()
            .to_string();

        // Extract port from SocketAddr
        let api_port = tracker_config.http_api.bind_address.port();

        PrometheusContext::new(scrape_interval, api_token, api_port)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::domain::tracker::HttpApiConfig;

    fn create_test_template_manager() -> Arc<TemplateManager> {
        use tempfile::TempDir;

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

    fn create_test_tracker_config() -> TrackerConfig {
        TrackerConfig {
            http_api: HttpApiConfig {
                bind_address: "0.0.0.0:1212".parse().expect("valid address"),
                admin_token: "test_admin_token".to_string().into(),
                domain: None,
                use_tls_proxy: false,
            },
            ..Default::default()
        }
    }

    #[test]
    fn it_should_create_prometheus_build_directory() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let build_dir = temp_dir.path().join("build");

        let template_manager = create_test_template_manager();
        let generator = PrometheusProjectGenerator::new(&build_dir, template_manager);

        let prometheus_config = PrometheusConfig::default();
        let tracker_config = create_test_tracker_config();

        generator
            .render(&prometheus_config, &tracker_config)
            .expect("Failed to render templates");

        let prometheus_dir = build_dir.join("prometheus");
        assert!(
            prometheus_dir.exists(),
            "Prometheus directory should be created"
        );
        assert!(
            prometheus_dir.is_dir(),
            "Prometheus build path should be a directory"
        );
    }

    #[test]
    fn it_should_render_prometheus_yml_with_default_config() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let build_dir = temp_dir.path().join("build");

        let template_manager = create_test_template_manager();
        let generator = PrometheusProjectGenerator::new(&build_dir, template_manager);

        let prometheus_config = PrometheusConfig::default(); // scrape_interval: 15
        let tracker_config = create_test_tracker_config();

        generator
            .render(&prometheus_config, &tracker_config)
            .expect("Failed to render templates");

        let prometheus_yml_path = build_dir.join("prometheus/prometheus.yml");
        assert!(
            prometheus_yml_path.exists(),
            "prometheus.yml should be created"
        );

        let content =
            fs::read_to_string(&prometheus_yml_path).expect("Failed to read prometheus.yml");

        // Verify default values
        assert!(content.contains("scrape_interval: 15s"));
        assert!(content.contains(r#"token: ["test_admin_token"]"#));
        assert!(content.contains("targets: [\"tracker:1212\"]"));
    }

    #[test]
    fn it_should_render_prometheus_yml_with_custom_scrape_interval() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let build_dir = temp_dir.path().join("build");

        let template_manager = create_test_template_manager();
        let generator = PrometheusProjectGenerator::new(&build_dir, template_manager);

        let prometheus_config =
            PrometheusConfig::new(std::num::NonZeroU32::new(30).expect("30 is non-zero"));
        let tracker_config = create_test_tracker_config();

        generator
            .render(&prometheus_config, &tracker_config)
            .expect("Failed to render templates");

        let content = fs::read_to_string(build_dir.join("prometheus/prometheus.yml"))
            .expect("Failed to read file");

        assert!(content.contains("scrape_interval: 30s"));
    }

    #[test]
    fn it_should_extract_api_port_from_tracker_config() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let build_dir = temp_dir.path().join("build");

        let template_manager = create_test_template_manager();
        let generator = PrometheusProjectGenerator::new(&build_dir, template_manager);

        let prometheus_config = PrometheusConfig::default();
        let mut tracker_config = create_test_tracker_config();
        tracker_config.http_api.bind_address = "0.0.0.0:8080".parse().expect("valid address");

        generator
            .render(&prometheus_config, &tracker_config)
            .expect("Failed to render templates");

        let content = fs::read_to_string(build_dir.join("prometheus/prometheus.yml"))
            .expect("Failed to read file");

        assert!(content.contains("targets: [\"tracker:8080\"]"));
    }

    #[test]
    fn it_should_use_tracker_api_token() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let build_dir = temp_dir.path().join("build");

        let template_manager = create_test_template_manager();
        let generator = PrometheusProjectGenerator::new(&build_dir, template_manager);

        let prometheus_config = PrometheusConfig::default();
        let mut tracker_config = create_test_tracker_config();
        tracker_config.http_api.admin_token = "custom_admin_token_123".to_string().into();

        generator
            .render(&prometheus_config, &tracker_config)
            .expect("Failed to render templates");

        let content = fs::read_to_string(build_dir.join("prometheus/prometheus.yml"))
            .expect("Failed to read file");

        assert!(content.contains(r#"token: ["custom_admin_token_123"]"#));
    }
}
