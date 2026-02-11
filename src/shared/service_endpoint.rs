//! Service endpoint configuration for external validation
//!
//! This module provides types to represent service endpoints that can be
//! tested via HTTP or HTTPS. When TLS is enabled, the endpoint uses the
//! domain with HTTPS protocol and resolves it locally to the server IP.

use std::net::{IpAddr, SocketAddr};

use url::Url;

use crate::shared::DomainName;

/// Error when creating a `ServiceEndpoint` with an invalid URL
#[derive(Debug, Clone, PartialEq)]
pub struct InvalidServiceEndpointUrl {
    /// The URL string that failed to parse
    pub url_string: String,
    /// The parse error message
    pub reason: String,
}

impl std::fmt::Display for InvalidServiceEndpointUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid service endpoint URL '{}': {}",
            self.url_string, self.reason
        )
    }
}

impl std::error::Error for InvalidServiceEndpointUrl {}

/// Represents a service endpoint for external validation testing
///
/// Internally stores a validated URL and the server IP address.
/// For HTTPS endpoints, the IP is used to resolve the domain locally
/// (since we can't rely on DNS for `.local` domains).
///
/// # Examples
///
/// ```
/// use std::net::SocketAddr;
/// use torrust_tracker_deployer_lib::shared::ServiceEndpoint;
///
/// // HTTP endpoint
/// let socket_addr: SocketAddr = "10.0.0.1:1212".parse().unwrap();
/// let endpoint = ServiceEndpoint::http(socket_addr, "/api/health_check").unwrap();
/// assert_eq!(endpoint.url().as_str(), "http://10.0.0.1:1212/api/health_check");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct ServiceEndpoint {
    /// The validated URL for this endpoint
    url: Url,

    /// The server IP address.
    /// For HTTP: extracted from the socket address.
    /// For HTTPS: used to resolve the domain locally.
    server_ip: IpAddr,
}

impl ServiceEndpoint {
    /// Create a new HTTP endpoint (no TLS)
    ///
    /// # Arguments
    ///
    /// * `socket_addr` - The IP address and port the service listens on
    /// * `path` - The health check path (e.g., `/api/health_check`)
    ///
    /// # Errors
    ///
    /// Returns an error if the socket address and path don't form a valid URL.
    pub fn http(
        socket_addr: SocketAddr,
        path: impl Into<String>,
    ) -> Result<Self, InvalidServiceEndpointUrl> {
        let path = path.into();
        let url_string = format!("http://{}:{}{}", socket_addr.ip(), socket_addr.port(), path); // DevSkim: ignore DS137138

        let url = Url::parse(&url_string).map_err(|e| InvalidServiceEndpointUrl {
            url_string,
            reason: e.to_string(),
        })?;

        Ok(Self {
            url,
            server_ip: socket_addr.ip(),
        })
    }

    /// Create a new HTTPS endpoint with TLS
    ///
    /// # Arguments
    ///
    /// * `domain` - The domain name for HTTPS access (required for certificate issuance)
    /// * `path` - The health check path (e.g., `/api/health_check`)
    /// * `server_ip` - The IP address to resolve the domain to
    ///
    /// # Errors
    ///
    /// Returns an error if the domain and path don't form a valid URL.
    pub fn https(
        domain: &DomainName,
        path: impl Into<String>,
        server_ip: IpAddr,
    ) -> Result<Self, InvalidServiceEndpointUrl> {
        let path = path.into();
        let url_string = format!("https://{}{}", domain.as_str(), path);

        let url = Url::parse(&url_string).map_err(|e| InvalidServiceEndpointUrl {
            url_string,
            reason: e.to_string(),
        })?;

        Ok(Self { url, server_ip })
    }

    /// Returns the URL for this endpoint
    #[must_use]
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns true if this endpoint uses TLS (HTTPS)
    #[must_use]
    pub fn uses_tls(&self) -> bool {
        self.url.scheme() == "https"
    }

    /// Returns the server IP address
    #[must_use]
    pub fn server_ip(&self) -> IpAddr {
        self.server_ip
    }

    /// Returns the port for this endpoint
    ///
    /// For HTTP: the configured port from the URL.
    /// For HTTPS: 443 (the default HTTPS port).
    #[must_use]
    pub fn port(&self) -> u16 {
        self.url.port_or_known_default().unwrap_or(80)
    }

    /// Get the domain if this is an HTTPS endpoint
    #[must_use]
    pub fn domain(&self) -> Option<&str> {
        if self.uses_tls() {
            self.url.host_str()
        } else {
            None
        }
    }

    /// Check if the domain ends with `.local` (for self-signed cert handling)
    #[must_use]
    pub fn is_local_domain(&self) -> bool {
        self.domain().is_some_and(|d| {
            std::path::Path::new(d)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("local"))
        })
    }

    /// Returns the socket address for connecting to this endpoint
    ///
    /// Combines the server IP with the port from the URL.
    #[must_use]
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.server_ip(), self.port())
    }
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::*;

    fn test_ip() -> IpAddr {
        "10.0.0.1".parse().unwrap()
    }

    fn test_socket_addr(port: u16) -> SocketAddr {
        SocketAddr::new(test_ip(), port)
    }

    #[test]
    fn it_should_create_http_endpoint() {
        let endpoint = ServiceEndpoint::http(test_socket_addr(1212), "/api/health_check").unwrap();

        assert_eq!(endpoint.server_ip(), test_ip());
        assert_eq!(endpoint.port(), 1212);
        assert!(!endpoint.uses_tls());
        assert!(endpoint.domain().is_none());
    }

    #[test]
    fn it_should_create_https_endpoint() {
        let domain = DomainName::new("api.tracker.local").unwrap();
        let endpoint = ServiceEndpoint::https(&domain, "/api/health_check", test_ip()).unwrap();

        assert_eq!(endpoint.server_ip(), test_ip());
        assert_eq!(endpoint.port(), 443);
        assert!(endpoint.uses_tls());
        assert_eq!(endpoint.domain().unwrap(), "api.tracker.local");
    }

    #[test]
    fn it_should_build_http_url() {
        let endpoint = ServiceEndpoint::http(test_socket_addr(1212), "/api/health_check").unwrap();

        assert_eq!(
            endpoint.url().as_str(),
            "http://10.0.0.1:1212/api/health_check" // DevSkim: ignore DS137138
        );
    }

    #[test]
    fn it_should_build_https_url() {
        let domain = DomainName::new("api.tracker.local").unwrap();
        let endpoint = ServiceEndpoint::https(&domain, "/api/health_check", test_ip()).unwrap();

        // HTTPS uses domain, not IP
        assert_eq!(
            endpoint.url().as_str(),
            "https://api.tracker.local/api/health_check"
        );
    }

    #[test]
    fn it_should_detect_local_domain() {
        let domain = DomainName::new("api.tracker.local").unwrap();
        let endpoint = ServiceEndpoint::https(&domain, "/api/health_check", test_ip()).unwrap();

        assert!(endpoint.is_local_domain());
    }

    #[test]
    fn it_should_not_detect_non_local_domain_as_local() {
        let domain = DomainName::new("api.tracker.example.com").unwrap();
        let endpoint = ServiceEndpoint::https(&domain, "/api/health_check", test_ip()).unwrap();

        assert!(!endpoint.is_local_domain());
    }

    #[test]
    fn it_should_return_port_443_for_https() {
        let domain = DomainName::new("api.tracker.local").unwrap();
        let endpoint = ServiceEndpoint::https(&domain, "/api/health_check", test_ip()).unwrap();

        assert_eq!(endpoint.port(), 443);
    }

    #[test]
    fn it_should_return_configured_port_for_http() {
        let endpoint = ServiceEndpoint::http(test_socket_addr(1212), "/api/health_check").unwrap();

        assert_eq!(endpoint.port(), 1212);
    }

    #[test]
    fn it_should_return_socket_addr_for_http() {
        let endpoint = ServiceEndpoint::http(test_socket_addr(1212), "/api/health_check").unwrap();

        assert_eq!(endpoint.socket_addr(), test_socket_addr(1212));
    }

    #[test]
    fn it_should_return_socket_addr_with_port_443_for_https() {
        let domain = DomainName::new("api.tracker.local").unwrap();
        let endpoint = ServiceEndpoint::https(&domain, "/api/health_check", test_ip()).unwrap();

        assert_eq!(endpoint.socket_addr(), test_socket_addr(443));
    }

    #[test]
    fn it_should_return_error_for_invalid_http_path() {
        // A path with invalid characters that would break URL parsing
        let result = ServiceEndpoint::http(test_socket_addr(1212), "not a valid path\x00");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.url_string.contains("not a valid path"));
    }

    #[test]
    fn it_should_return_url_reference() {
        let endpoint = ServiceEndpoint::http(test_socket_addr(1212), "/api/health_check").unwrap();

        // url() returns a reference, not a clone
        let url_ref: &Url = endpoint.url();
        assert_eq!(url_ref.path(), "/api/health_check");
    }
}
