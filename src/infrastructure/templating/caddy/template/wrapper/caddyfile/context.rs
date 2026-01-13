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
///
/// // All services with HTTPS
/// let context = CaddyContext {
///     admin_email: "admin@example.com".to_string(),
///     use_staging: false,
///     tracker_api: Some(CaddyService::new("api.example.com", 1212)),
///     http_trackers: vec![
///         CaddyService::new("http1.example.com", 7070),
///         CaddyService::new("http2.example.com", 7071),
///     ],
///     grafana: Some(CaddyService::new("grafana.example.com", 3000)),
/// };
/// ```
///
/// # Data Flow
///
/// Environment Config (tracker, grafana sections with tls) → Application Layer
/// → `CaddyContext` with pre-extracted ports
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct CaddyContext {
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

    /// HTTP Tracker services with TLS configured
    ///
    /// Contains only trackers that have `tls` configuration.
    /// Trackers without TLS are served directly over HTTP, not through Caddy.
    pub http_trackers: Vec<CaddyService>,

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
    /// * `admin_email` - Email for Let's Encrypt notifications
    /// * `use_staging` - Whether to use Let's Encrypt staging environment
    #[must_use]
    pub fn new(admin_email: impl Into<String>, use_staging: bool) -> Self {
        Self {
            admin_email: admin_email.into(),
            use_staging,
            tracker_api: None,
            http_trackers: Vec::new(),
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
        self.tracker_api.is_some() || !self.http_trackers.is_empty() || self.grafana.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_caddy_service() {
        let service = CaddyService::new("api.example.com", 1212);

        assert_eq!(service.domain, "api.example.com");
        assert_eq!(service.port, 1212);
    }

    #[test]
    fn it_should_create_caddy_context_with_builder_pattern() {
        let context = CaddyContext::new("admin@example.com", false)
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

        let api_only = CaddyContext::new("admin@example.com", false)
            .with_tracker_api(CaddyService::new("api.example.com", 1212));
        assert!(api_only.has_any_tls());

        let http_tracker_only = CaddyContext::new("admin@example.com", false)
            .with_http_tracker(CaddyService::new("http.example.com", 7070));
        assert!(http_tracker_only.has_any_tls());

        let grafana_only = CaddyContext::new("admin@example.com", false)
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
        assert!(context.grafana.is_none());
    }

    #[test]
    fn it_should_serialize_to_json() {
        let context = CaddyContext::new("admin@example.com", true)
            .with_tracker_api(CaddyService::new("api.example.com", 1212))
            .with_http_tracker(CaddyService::new("http.example.com", 7070));

        let json = serde_json::to_value(&context).expect("serialization should succeed");

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
        let production = CaddyContext::new("admin@example.com", false);
        let staging = CaddyContext::new("admin@example.com", true);

        assert!(!production.use_staging);
        assert!(staging.use_staging);
    }
}
