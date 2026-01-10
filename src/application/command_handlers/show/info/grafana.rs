//! Grafana visualization service information for display purposes
//!
//! This module contains DTOs for the Grafana service.

use std::net::IpAddr;

use url::Url;

/// Grafana visualization service information for display purposes
///
/// This information shows the status of the Grafana service when configured.
/// Grafana provides dashboards for visualizing tracker metrics.
/// Note: Grafana requires Prometheus to be configured.
#[derive(Debug, Clone)]
pub struct GrafanaInfo {
    /// Grafana dashboard URL
    pub url: Url,
}

impl GrafanaInfo {
    /// Create a new `GrafanaInfo`
    #[must_use]
    pub fn new(url: Url) -> Self {
        Self { url }
    }

    /// Build `GrafanaInfo` from instance IP
    ///
    /// Grafana is exposed on port 3100 (mapped from internal port 3000).
    ///
    /// # Panics
    ///
    /// This function will panic if the URL cannot be parsed, which should
    /// never happen since we construct a valid URL from a valid IP address.
    #[must_use]
    pub fn from_instance_ip(instance_ip: IpAddr) -> Self {
        let url = Url::parse(&format!("http://{instance_ip}:3100")) // DevSkim: ignore DS137138
            .expect("Valid IP address should produce valid URL");
        Self::new(url)
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn it_should_create_grafana_info_from_url() {
        let url = Url::parse("http://10.0.0.1:3100").unwrap(); // DevSkim: ignore DS137138
        let info = GrafanaInfo::new(url.clone());
        assert_eq!(info.url, url);
    }

    #[test]
    fn it_should_create_grafana_info_from_instance_ip() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let info = GrafanaInfo::from_instance_ip(ip);
        assert_eq!(info.url.host_str(), Some("192.168.1.100"));
        assert_eq!(info.url.port(), Some(3100));
    }
}
