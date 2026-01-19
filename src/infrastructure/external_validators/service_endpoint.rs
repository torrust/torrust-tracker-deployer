//! Service endpoint configuration for external validation
//!
//! This module provides types to represent service endpoints that can be
//! tested via HTTP or HTTPS. When TLS is enabled, the endpoint uses the
//! domain with HTTPS protocol and resolves it locally to the server IP.

use std::net::IpAddr;

use crate::shared::DomainName;

/// Represents a service endpoint for external validation testing
///
/// When TLS is enabled, the endpoint uses HTTPS with the configured domain.
/// The domain is resolved locally to the server IP using reqwest's resolve
/// feature (equivalent to curl's `--resolve` flag), allowing tests to work
/// without DNS configuration while still being realistic.
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceEndpoint {
    /// The port the service listens on internally
    pub port: u16,

    /// The health check path (e.g., `/api/health_check` or `/health_check`)
    pub path: String,

    /// TLS configuration if HTTPS is enabled
    pub tls: Option<TlsEndpointConfig>,
}

/// TLS configuration for an endpoint
#[derive(Debug, Clone, PartialEq)]
pub struct TlsEndpointConfig {
    /// Domain name for HTTPS access
    pub domain: DomainName,
}

impl ServiceEndpoint {
    /// Create a new HTTP endpoint (no TLS)
    #[must_use]
    pub fn http(port: u16, path: impl Into<String>) -> Self {
        Self {
            port,
            path: path.into(),
            tls: None,
        }
    }

    /// Create a new HTTPS endpoint with TLS
    #[must_use]
    pub fn https(port: u16, path: impl Into<String>, domain: DomainName) -> Self {
        Self {
            port,
            path: path.into(),
            tls: Some(TlsEndpointConfig { domain }),
        }
    }

    /// Returns true if this endpoint uses TLS
    #[must_use]
    pub fn uses_tls(&self) -> bool {
        self.tls.is_some()
    }

    /// Get the domain if TLS is enabled
    #[must_use]
    pub fn domain(&self) -> Option<&DomainName> {
        self.tls.as_ref().map(|t| &t.domain)
    }

    /// Build the URL for this endpoint
    ///
    /// - For HTTP: `http://{server_ip}:{port}{path}`
    /// - For HTTPS: `https://{domain}{path}` (port 443 implied)
    #[must_use]
    pub fn url(&self, server_ip: &IpAddr) -> String {
        if let Some(tls) = &self.tls {
            // HTTPS uses domain, port 443 is implied
            format!("https://{}{}", tls.domain.as_str(), self.path)
        } else {
            // HTTP uses IP and port directly
            format!("http://{}:{}{}", server_ip, self.port, self.path) // DevSkim: ignore DS137138
        }
    }

    /// Check if the domain ends with .local (for self-signed cert handling)
    #[must_use]
    pub fn is_local_domain(&self) -> bool {
        self.tls.as_ref().is_some_and(|t| {
            std::path::Path::new(t.domain.as_str())
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("local"))
        })
    }

    /// Returns the HTTPS port (443 for TLS, or the configured port for HTTP)
    #[must_use]
    pub fn effective_port(&self) -> u16 {
        if self.uses_tls() {
            443
        } else {
            self.port
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_http_endpoint() {
        let endpoint = ServiceEndpoint::http(1212, "/api/health_check");

        assert_eq!(endpoint.port, 1212);
        assert_eq!(endpoint.path, "/api/health_check");
        assert!(!endpoint.uses_tls());
        assert!(endpoint.domain().is_none());
    }

    #[test]
    fn it_should_create_https_endpoint() {
        let domain = DomainName::new("api.tracker.local").unwrap();
        let endpoint = ServiceEndpoint::https(1212, "/api/health_check", domain);

        assert_eq!(endpoint.port, 1212);
        assert_eq!(endpoint.path, "/api/health_check");
        assert!(endpoint.uses_tls());
        assert_eq!(endpoint.domain().unwrap().as_str(), "api.tracker.local");
    }

    #[test]
    fn it_should_build_http_url() {
        let endpoint = ServiceEndpoint::http(1212, "/api/health_check");
        let ip: IpAddr = "10.0.0.1".parse().unwrap();

        assert_eq!(endpoint.url(&ip), "http://10.0.0.1:1212/api/health_check");
    }

    #[test]
    fn it_should_build_https_url() {
        let domain = DomainName::new("api.tracker.local").unwrap();
        let endpoint = ServiceEndpoint::https(1212, "/api/health_check", domain);
        let ip: IpAddr = "10.0.0.1".parse().unwrap();

        // HTTPS uses domain, not IP
        assert_eq!(
            endpoint.url(&ip),
            "https://api.tracker.local/api/health_check"
        );
    }

    #[test]
    fn it_should_detect_local_domain() {
        let domain = DomainName::new("api.tracker.local").unwrap();
        let endpoint = ServiceEndpoint::https(1212, "/api/health_check", domain);

        assert!(endpoint.is_local_domain());
    }

    #[test]
    fn it_should_not_detect_non_local_domain_as_local() {
        let domain = DomainName::new("api.tracker.example.com").unwrap();
        let endpoint = ServiceEndpoint::https(1212, "/api/health_check", domain);

        assert!(!endpoint.is_local_domain());
    }

    #[test]
    fn it_should_return_effective_port_443_for_tls() {
        let domain = DomainName::new("api.tracker.local").unwrap();
        let endpoint = ServiceEndpoint::https(1212, "/api/health_check", domain);

        assert_eq!(endpoint.effective_port(), 443);
    }

    #[test]
    fn it_should_return_configured_port_for_http() {
        let endpoint = ServiceEndpoint::http(1212, "/api/health_check");

        assert_eq!(endpoint.effective_port(), 1212);
    }
}
