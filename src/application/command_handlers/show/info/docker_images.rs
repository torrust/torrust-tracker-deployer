use serde::Serialize;

/// Docker image information for the deployment stack
///
/// Contains the Docker image references for all services in the deployment.
/// Optional services (`MySQL`, Prometheus, Grafana) are `None` if not configured.
#[derive(Debug, Clone, Serialize)]
pub struct DockerImagesInfo {
    /// Tracker Docker image reference (e.g. `torrust/tracker:develop`)
    pub tracker: String,

    /// `MySQL` Docker image reference (e.g. `mysql:8.4`), present when `MySQL` is configured
    pub mysql: Option<String>,

    /// Prometheus Docker image reference (e.g. `prom/prometheus:v3.11.2`), present when configured
    pub prometheus: Option<String>,

    /// Grafana Docker image reference (e.g. `grafana/grafana:12.4.2`), present when configured
    pub grafana: Option<String>,
}

impl DockerImagesInfo {
    /// Create a new `DockerImagesInfo` with the tracker image and optional service images
    #[must_use]
    pub fn new(
        tracker: String,
        mysql: Option<String>,
        prometheus: Option<String>,
        grafana: Option<String>,
    ) -> Self {
        Self {
            tracker,
            mysql,
            prometheus,
            grafana,
        }
    }
}
