//! Caddy template context
//!
//! Defines the variables needed for Caddyfile.tera template rendering.
//!
//! ## Context Data Preparation Pattern
//!
//! This context follows the Context Data Preparation Pattern (see
//! `docs/contributing/templates/template-system-architecture.md`):
//! - Ports are pre-extracted in Rust from `SocketAddr` when building the context
//! - Templates receive ready-to-use data without complex Tera filters
//! - Each service includes both domain and port as simple types

use serde::Serialize;

use crate::infrastructure::templating::TemplateMetadata;

/// Represents a service that can be proxied through Caddy
///
/// Contains the domain name for TLS certificate acquisition and the
/// backend port for reverse proxying.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::caddy::CaddyService;
///
/// let service = CaddyService {
///     domain: "api.torrust-tracker.com".to_string(),
///     port: 1212,
/// };
/// ```
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CaddyService {
    /// Domain name for this service (used for TLS certificate)
    ///
    /// Must be a valid domain name that points to the deployment server.
    /// Let's Encrypt will validate domain ownership via HTTP-01 challenge.
    pub domain: String,

    /// Backend port where the service is listening
    ///
    /// This is the internal Docker network port, not the public-facing port.
    /// Caddy will reverse proxy HTTPS traffic to this port.
    pub port: u16,
}

impl CaddyService {
    /// Creates a new `CaddyService`
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain name for TLS certificate
    /// * `port` - The backend port for reverse proxying
    #[must_use]
    pub fn new(domain: impl Into<String>, port: u16) -> Self {
        Self {
            domain: domain.into(),
            port,
        }
    }
}

/// Context for rendering Caddyfile.tera template
///
/// Contains all variables needed for Caddy reverse proxy configuration.
/// Only services with TLS configuration will be included in this context.
///
/// # Design Decisions
///
/// - `tracker_api`, `grafana`: `Option<CaddyService>` - only present if TLS configured
/// - `http_trackers`: `Vec<CaddyService>` - only TLS-enabled trackers included
/// - Ports are pre-extracted in Rust (not in templates) per Context Data Preparation Pattern
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::caddy::{
///     CaddyContext, CaddyService,
/// };
/// use torrust_tracker_deployer_lib::infrastructure::templating::TemplateMetadata;
/// use torrust_tracker_deployer_lib::shared::clock::{Clock, SystemClock};
///
/// let clock = SystemClock;
/// let metadata = TemplateMetadata::new(clock.now());
///
/// // All services with HTTPS
/// let context = CaddyContext::new(
///     metadata,
///     "admin@example.com",
///     false,
/// )
/// .with_tracker_api(CaddyService::new("api.example.com", 1212))
/// .with_http_tracker(CaddyService::new("http1.example.com", 7070))
/// .with_http_tracker(CaddyService::new("http2.example.com", 7071))
/// .with_grafana(CaddyService::new("grafana.example.com", 3000));
/// ```
///
/// # Data Flow
///
/// Environment Config (tracker, grafana sections with tls) → Application Layer
/// → `CaddyContext` with pre-extracted ports
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CaddyContext {
    /// Template metadata (timestamp, etc.)
    ///
    /// Contains information about when the template was generated, useful for
    /// tracking template versions and ensuring reproducibility.
    #[serde(flatten)]
    pub metadata: TemplateMetadata,

    /// Email for Let's Encrypt certificate notifications
    ///
    /// Required when any service has TLS configured.
    /// Let's Encrypt sends expiration warnings to this email.
    pub admin_email: String,

    /// Whether to use Let's Encrypt staging environment
    ///
    /// - `true`: Use staging CA (for testing, avoids rate limits)
    /// - `false`: Use production CA (default, trusted certificates)
    ///
    /// Staging certificates show browser warnings (not trusted by browsers).
    pub use_staging: bool,

    /// Tracker REST API service (if TLS configured)
    ///
    /// Present only if `tracker.http_api.tls` is configured.
    pub tracker_api: Option<CaddyService>,

    /// HTTP Tracker services with TLS proxy configured
    ///
    /// Contains only trackers that have `use_tls_proxy: true` and a domain.
    /// Trackers without TLS proxy are served directly over HTTP, not through Caddy.
    pub http_trackers: Vec<CaddyService>,

    /// Health Check API service (if TLS proxy configured)
    ///
    /// Present only if `tracker.health_check_api.use_tls_proxy` is enabled.
    /// The health check API provides a simple /health endpoint for monitoring.
    pub health_check_api: Option<CaddyService>,

    /// Grafana UI service (if TLS configured)
    ///
    /// Present only if `grafana.tls` is configured.
    /// Caddy provides WebSocket support for Grafana Live features.
    pub grafana: Option<CaddyService>,
}

impl CaddyContext {
    /// Creates a new `CaddyContext`
    ///
    /// # Arguments
    ///
    /// * `metadata` - Template metadata (timestamp, etc.)
    /// * `admin_email` - Email for Let's Encrypt notifications
    /// * `use_staging` - Whether to use Let's Encrypt staging environment
    #[must_use]
    pub fn new(
        metadata: TemplateMetadata,
        admin_email: impl Into<String>,
        use_staging: bool,
    ) -> Self {
        Self {
            metadata,
            admin_email: admin_email.into(),
            use_staging,
            tracker_api: None,
            http_trackers: Vec::new(),
            health_check_api: None,
            grafana: None,
        }
    }

    /// Sets the Tracker API service
    #[must_use]
    pub fn with_tracker_api(mut self, service: CaddyService) -> Self {
        self.tracker_api = Some(service);
        self
    }

    /// Adds an HTTP Tracker service
    #[must_use]
    pub fn with_http_tracker(mut self, service: CaddyService) -> Self {
        self.http_trackers.push(service);
        self
    }

    /// Sets the Health Check API service
    #[must_use]
    pub fn with_health_check_api(mut self, service: CaddyService) -> Self {
        self.health_check_api = Some(service);
        self
    }

    /// Sets the Grafana service
    #[must_use]
    pub fn with_grafana(mut self, service: CaddyService) -> Self {
        self.grafana = Some(service);
        self
    }

    /// Returns true if any service has TLS configured
    ///
    /// Used to determine whether Caddy should be deployed at all.
    #[must_use]
    pub fn has_any_tls(&self) -> bool {
        self.tracker_api.is_some()
            || !self.http_trackers.is_empty()
            || self.health_check_api.is_some()
            || self.grafana.is_some()
    }
}

impl Default for CaddyContext {
    fn default() -> Self {
        use chrono::{TimeZone, Utc};
        Self {
            metadata: TemplateMetadata::new(Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()),
            admin_email: String::new(),
            use_staging: false,
            tracker_api: None,
            http_trackers: Vec::new(),
            health_check_api: None,
            grafana: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    fn create_test_metadata() -> TemplateMetadata {
        TemplateMetadata::new(Utc.with_ymd_and_hms(2026, 1, 27, 13, 41, 56).unwrap())
    }

    #[test]
    fn it_should_create_caddy_service() {
        let service = CaddyService::new("api.example.com", 1212);

        assert_eq!(service.domain, "api.example.com");
        assert_eq!(service.port, 1212);
    }

    #[test]
    fn it_should_create_caddy_context_with_builder_pattern() {
        let context = CaddyContext::new(create_test_metadata(), "admin@example.com", false)
            .with_tracker_api(CaddyService::new("api.example.com", 1212))
            .with_http_tracker(CaddyService::new("http1.example.com", 7070))
            .with_http_tracker(CaddyService::new("http2.example.com", 7071))
            .with_grafana(CaddyService::new("grafana.example.com", 3000));

        assert_eq!(context.admin_email, "admin@example.com");
        assert!(!context.use_staging);
        assert!(context.tracker_api.is_some());
        assert_eq!(context.http_trackers.len(), 2);
        assert!(context.grafana.is_some());
    }

    #[test]
    fn it_should_detect_when_tls_is_configured() {
        let empty_context = CaddyContext::default();
        assert!(!empty_context.has_any_tls());

        let api_only = CaddyContext::new(create_test_metadata(), "admin@example.com", false)
            .with_tracker_api(CaddyService::new("api.example.com", 1212));
        assert!(api_only.has_any_tls());

        let http_tracker_only =
            CaddyContext::new(create_test_metadata(), "admin@example.com", false)
                .with_http_tracker(CaddyService::new("http.example.com", 7070));
        assert!(http_tracker_only.has_any_tls());

        let health_check_only =
            CaddyContext::new(create_test_metadata(), "admin@example.com", false)
                .with_health_check_api(CaddyService::new("health.example.com", 1313));
        assert!(health_check_only.has_any_tls());

        let grafana_only = CaddyContext::new(create_test_metadata(), "admin@example.com", false)
            .with_grafana(CaddyService::new("grafana.example.com", 3000));
        assert!(grafana_only.has_any_tls());
    }

    #[test]
    fn it_should_create_default_context() {
        let context = CaddyContext::default();

        assert_eq!(context.admin_email, "");
        assert!(!context.use_staging);
        assert!(context.tracker_api.is_none());
        assert!(context.http_trackers.is_empty());
        assert!(context.health_check_api.is_none());
        assert!(context.grafana.is_none());
    }

    #[test]
    fn it_should_serialize_to_json() {
        let context = CaddyContext::new(create_test_metadata(), "admin@example.com", true)
            .with_tracker_api(CaddyService::new("api.example.com", 1212))
            .with_http_tracker(CaddyService::new("http.example.com", 7070));

        let json = serde_json::to_value(&context).expect("serialization should succeed");

        assert_eq!(json["generated_at"], "2026-01-27T13:41:56Z");
        assert_eq!(json["admin_email"], "admin@example.com");
        assert_eq!(json["use_staging"], true);
        assert_eq!(json["tracker_api"]["domain"], "api.example.com");
        assert_eq!(json["tracker_api"]["port"], 1212);
        assert_eq!(json["http_trackers"][0]["domain"], "http.example.com");
        assert_eq!(json["http_trackers"][0]["port"], 7070);
        assert!(json["grafana"].is_null());
    }

    #[test]
    fn it_should_use_staging_for_testing() {
        let production = CaddyContext::new(create_test_metadata(), "admin@example.com", false);
        let staging = CaddyContext::new(create_test_metadata(), "admin@example.com", true);

        assert!(!production.use_staging);
        assert!(staging.use_staging);
    }
}
