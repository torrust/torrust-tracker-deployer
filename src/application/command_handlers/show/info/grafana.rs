//! Grafana visualization service information for display purposes
//!
//! This module contains DTOs for the Grafana service.

use std::net::IpAddr;

use serde::Serialize;
use url::Url;

use crate::domain::grafana::GrafanaConfig;

/// Grafana visualization service information for display purposes
///
/// This information shows the status of the Grafana service when configured.
/// Grafana provides dashboards for visualizing tracker metrics.
/// Note: Grafana requires Prometheus to be configured.
#[derive(Debug, Clone, Serialize)]
pub struct GrafanaInfo {
    /// Grafana dashboard URL
    pub url: Url,

    /// Whether Grafana is accessed via HTTPS through Caddy
    pub uses_https: bool,
}

impl GrafanaInfo {
    /// Create a new `GrafanaInfo`
    #[must_use]
    pub fn new(url: Url, uses_https: bool) -> Self {
        Self { url, uses_https }
    }

    /// Build `GrafanaInfo` from instance IP (HTTP direct access)
    ///
    /// Grafana is exposed on port 3000 (same as internal port).
    ///
    /// # Panics
    ///
    /// This function will panic if the URL cannot be parsed, which should
    /// never happen since we construct a valid URL from a valid IP address.
    #[must_use]
    pub fn from_instance_ip(instance_ip: IpAddr) -> Self {
        let url = Url::parse(&format!("http://{instance_ip}:3000")) // DevSkim: ignore DS137138
            .expect("Valid IP address should produce valid URL");
        Self::new(url, false)
    }

    /// Build `GrafanaInfo` from Grafana configuration
    ///
    /// If TLS is configured, returns HTTPS URL with domain.
    /// Otherwise, returns HTTP URL with IP address.
    ///
    /// # Panics
    ///
    /// This function will panic if the URL cannot be parsed, which should
    /// never happen since we construct valid URLs.
    #[must_use]
    pub fn from_config(config: &GrafanaConfig, instance_ip: IpAddr) -> Self {
        if let Some(domain) = config.tls_domain() {
            let url = Url::parse(&format!("https://{domain}"))
                .expect("Valid domain should produce valid URL");
            Self::new(url, true)
        } else {
            Self::from_instance_ip(instance_ip)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn it_should_create_grafana_info_from_url() {
        let url = Url::parse("http://10.0.0.1:3000").unwrap(); // DevSkim: ignore DS137138
        let info = GrafanaInfo::new(url.clone(), false);
        assert_eq!(info.url, url);
        assert!(!info.uses_https);
    }

    #[test]
    fn it_should_create_grafana_info_from_instance_ip() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let info = GrafanaInfo::from_instance_ip(ip);
        assert_eq!(info.url.host_str(), Some("192.168.1.100"));
        assert_eq!(info.url.port(), Some(3000));
        assert!(!info.uses_https);
    }

    #[test]
    fn it_should_create_grafana_info_with_https_from_config() {
        use crate::domain::grafana::GrafanaConfig;
        use crate::shared::domain_name::DomainName;

        let domain = DomainName::new("grafana.tracker.local").unwrap();
        let config =
            GrafanaConfig::new("admin".to_string(), "pass".to_string(), Some(domain), true);
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        let info = GrafanaInfo::from_config(&config, ip);

        assert_eq!(info.url.scheme(), "https");
        assert_eq!(info.url.host_str(), Some("grafana.tracker.local"));
        assert!(info.uses_https);
    }

    #[test]
    fn it_should_create_grafana_info_with_http_from_config_without_tls() {
        let config = GrafanaConfig::new("admin".to_string(), "pass".to_string(), None, false);
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

        let info = GrafanaInfo::from_config(&config, ip);

        assert_eq!(info.url.scheme(), "http");
        assert_eq!(info.url.host_str(), Some("10.0.0.1"));
        assert_eq!(info.url.port(), Some(3000));
        assert!(!info.uses_https);
    }
}
