//! TLS configuration domain types
//!
//! This module provides domain-level TLS configuration used in tracker
//! and Grafana services for HTTPS termination via Caddy.

use serde::{Deserialize, Serialize};

use crate::shared::DomainName;

/// Service-specific TLS configuration (domain level)
///
/// Contains the domain name for Let's Encrypt certificate acquisition.
/// Present on services that should be accessible via HTTPS.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TlsConfig {
    /// Domain name for this service (used for TLS certificate)
    ///
    /// Must be a valid domain name that points to the deployment server.
    /// Let's Encrypt will validate domain ownership via HTTP-01 challenge.
    domain: DomainName,
}

impl TlsConfig {
    /// Creates a new TLS configuration
    ///
    /// # Arguments
    ///
    /// * `domain` - The validated domain name for TLS certificate
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::tls::TlsConfig;
    /// use torrust_tracker_deployer_lib::shared::DomainName;
    ///
    /// let domain = DomainName::new("api.example.com").unwrap();
    /// let tls = TlsConfig::new(domain);
    /// assert_eq!(tls.domain(), "api.example.com");
    /// ```
    #[must_use]
    pub fn new(domain: DomainName) -> Self {
        Self { domain }
    }

    /// Returns the domain name as a string slice
    #[must_use]
    pub fn domain(&self) -> &str {
        self.domain.as_str()
    }

    /// Returns the domain name type
    #[must_use]
    pub fn domain_name(&self) -> &DomainName {
        &self.domain
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_tls_config() {
        let domain = DomainName::new("api.tracker.example.com").unwrap();
        let tls = TlsConfig::new(domain);

        assert_eq!(tls.domain(), "api.tracker.example.com");
    }

    #[test]
    fn it_should_serialize_to_json() {
        let domain = DomainName::new("api.example.com").unwrap();
        let tls = TlsConfig::new(domain);

        let json = serde_json::to_string(&tls).expect("serialization should succeed");

        assert!(json.contains("\"domain\":\"api.example.com\""));
    }

    #[test]
    fn it_should_deserialize_from_json() {
        let json = r#"{"domain":"api.example.com"}"#;

        let tls: TlsConfig = serde_json::from_str(json).expect("deserialization should succeed");

        assert_eq!(tls.domain(), "api.example.com");
    }

    #[test]
    fn it_should_be_cloneable() {
        let domain = DomainName::new("api.example.com").unwrap();
        let tls = TlsConfig::new(domain);
        let cloned = tls.clone();

        assert_eq!(tls, cloned);
    }
}
