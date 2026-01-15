//! Tracker service information for display purposes
//!
//! This module contains DTOs for the Torrust Tracker service endpoints.

use std::net::IpAddr;

use crate::domain::environment::runtime_outputs::ServiceEndpoints;
use crate::domain::grafana::GrafanaConfig;
use crate::domain::tracker::TrackerConfig;

/// Tracker service information for display purposes
///
/// This information is available for Released and Running states and shows
/// the tracker services configured for the environment.
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// UDP tracker URLs (e.g., `udp://10.0.0.1:6969/announce`)
    pub udp_trackers: Vec<String>,

    /// HTTP tracker URLs with HTTPS via Caddy (e.g., `https://http1.tracker.local/announce`)
    pub https_http_trackers: Vec<String>,

    /// HTTP tracker URLs with direct access (e.g., `http://10.0.0.1:7072/announce`)
    pub direct_http_trackers: Vec<String>,

    /// HTTP API endpoint URL (e.g., `http://10.0.0.1:1212/api` or `https://api.tracker.local/api`)
    pub api_endpoint: String,

    /// Whether the API endpoint uses HTTPS via Caddy
    pub api_uses_https: bool,

    /// Health check API URL (e.g., `http://10.0.0.1:1313/health_check`)
    pub health_check_url: String,

    /// Domains configured for TLS services (for /etc/hosts hint)
    pub tls_domains: Vec<TlsDomainInfo>,
}

/// Information about a TLS-enabled domain for /etc/hosts hint
#[derive(Debug, Clone)]
pub struct TlsDomainInfo {
    /// The domain name
    pub domain: String,
    /// Internal port that is NOT exposed (for informational purposes)
    pub internal_port: u16,
}

impl TlsDomainInfo {
    /// Create a new `TlsDomainInfo`
    #[must_use]
    pub fn new(domain: String, internal_port: u16) -> Self {
        Self {
            domain,
            internal_port,
        }
    }
}

impl ServiceInfo {
    /// Create a new `ServiceInfo`
    #[must_use]
    pub fn new(
        udp_trackers: Vec<String>,
        https_http_trackers: Vec<String>,
        direct_http_trackers: Vec<String>,
        api_endpoint: String,
        api_uses_https: bool,
        health_check_url: String,
        tls_domains: Vec<TlsDomainInfo>,
    ) -> Self {
        Self {
            udp_trackers,
            https_http_trackers,
            direct_http_trackers,
            api_endpoint,
            api_uses_https,
            health_check_url,
            tls_domains,
        }
    }

    /// Build `ServiceInfo` from tracker configuration and instance IP
    ///
    /// This method constructs service URLs by combining the configured bind
    /// addresses with the actual instance IP address. It separates HTTP trackers
    /// into HTTPS-enabled (via Caddy) and direct HTTP access groups.
    ///
    /// # Arguments
    ///
    /// * `tracker_config` - The tracker configuration containing service settings
    /// * `instance_ip` - The IP address of the deployed instance
    /// * `grafana_config` - Optional Grafana configuration (for TLS domain info)
    #[must_use]
    pub fn from_tracker_config(
        tracker_config: &TrackerConfig,
        instance_ip: IpAddr,
        grafana_config: Option<&GrafanaConfig>,
    ) -> Self {
        let udp_trackers = tracker_config
            .udp_trackers
            .iter()
            .map(|udp| format!("udp://{}:{}/announce", instance_ip, udp.bind_address.port()))
            .collect();

        // Separate HTTP trackers by TLS configuration
        let mut https_http_trackers = Vec::new();
        let mut direct_http_trackers = Vec::new();
        let mut tls_domains = Vec::new();

        for http in &tracker_config.http_trackers {
            if let Some(tls) = &http.tls {
                // TLS-enabled tracker - use HTTPS domain URL
                https_http_trackers.push(format!("https://{}/announce", tls.domain()));
                tls_domains.push(TlsDomainInfo {
                    domain: tls.domain().to_string(),
                    internal_port: http.bind_address.port(),
                });
            } else {
                // Non-TLS tracker - use direct IP URL
                direct_http_trackers.push(format!(
                    "http://{}:{}/announce", // DevSkim: ignore DS137138
                    instance_ip,
                    http.bind_address.port()
                ));
            }
        }

        // Build API endpoint based on TLS configuration
        let (api_endpoint, api_uses_https) = if let Some(tls) = &tracker_config.http_api.tls {
            tls_domains.push(TlsDomainInfo {
                domain: tls.domain().to_string(),
                internal_port: tracker_config.http_api.bind_address.port(),
            });
            (format!("https://{}/api", tls.domain()), true)
        } else {
            (
                format!(
                    "http://{}:{}/api", // DevSkim: ignore DS137138
                    instance_ip,
                    tracker_config.http_api.bind_address.port()
                ),
                false,
            )
        };

        // Add Grafana TLS domain if configured
        if let Some(grafana) = grafana_config {
            if let Some(domain) = grafana.tls_domain() {
                tls_domains.push(TlsDomainInfo {
                    domain: domain.to_string(),
                    internal_port: 3000, // Grafana internal port
                });
            }
        }

        let health_check_url = format!(
            "http://{}:{}/health_check", // DevSkim: ignore DS137138
            instance_ip,
            tracker_config.health_check_api.bind_address.port()
        );

        Self::new(
            udp_trackers,
            https_http_trackers,
            direct_http_trackers,
            api_endpoint,
            api_uses_https,
            health_check_url,
            tls_domains,
        )
    }

    /// Build `ServiceInfo` from stored `ServiceEndpoints`
    ///
    /// This method extracts service URLs from the runtime outputs
    /// that were stored when services were started.
    ///
    /// Note: This method is for backward compatibility with stored endpoints.
    /// New deployments should use `from_tracker_config` which has full TLS awareness.
    #[must_use]
    pub fn from_service_endpoints(endpoints: &ServiceEndpoints) -> Self {
        let udp_trackers = endpoints
            .udp_trackers
            .iter()
            .map(ToString::to_string)
            .collect();

        // For backward compatibility, all HTTP trackers go to direct access
        // (stored endpoints don't have TLS information)
        let direct_http_trackers = endpoints
            .http_trackers
            .iter()
            .map(ToString::to_string)
            .collect();

        let api_endpoint = endpoints
            .api_endpoint
            .as_ref()
            .map_or_else(String::new, ToString::to_string);

        let health_check_url = endpoints
            .health_check_url
            .as_ref()
            .map_or_else(String::new, ToString::to_string);

        Self::new(
            udp_trackers,
            Vec::new(), // No HTTPS trackers from legacy endpoints
            direct_http_trackers,
            api_endpoint,
            false, // Legacy endpoints don't have TLS info
            health_check_url,
            Vec::new(), // No TLS domains from legacy endpoints
        )
    }

    /// Returns true if any service has TLS enabled
    #[must_use]
    pub fn has_any_tls(&self) -> bool {
        !self.tls_domains.is_empty()
    }

    /// Returns all TLS domain names (for /etc/hosts hint)
    #[must_use]
    pub fn tls_domain_names(&self) -> Vec<&str> {
        self.tls_domains.iter().map(|d| d.domain.as_str()).collect()
    }

    /// Returns all internal ports that are not exposed due to TLS
    #[must_use]
    pub fn unexposed_ports(&self) -> Vec<u16> {
        self.tls_domains.iter().map(|d| d.internal_port).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_service_info() {
        let services = ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec!["https://http1.tracker.local/announce".to_string()],
            vec!["http://10.0.0.1:7072/announce".to_string()], // DevSkim: ignore DS137138
            "http://10.0.0.1:1212/api".to_string(),            // DevSkim: ignore DS137138
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            vec![TlsDomainInfo {
                domain: "http1.tracker.local".to_string(),
                internal_port: 7070,
            }],
        );

        assert_eq!(services.udp_trackers.len(), 1);
        assert_eq!(services.https_http_trackers.len(), 1);
        assert_eq!(services.direct_http_trackers.len(), 1);
        assert!(services.api_endpoint.contains("1212"));
        assert!(!services.api_uses_https);
        assert!(services.health_check_url.contains("1313"));
        assert!(services.has_any_tls());
    }

    #[test]
    fn it_should_return_tls_domain_names() {
        let services = ServiceInfo::new(
            vec![],
            vec!["https://api.tracker.local/announce".to_string()],
            vec![],
            "https://api.tracker.local/api".to_string(),
            true,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            vec![
                TlsDomainInfo {
                    domain: "api.tracker.local".to_string(),
                    internal_port: 1212,
                },
                TlsDomainInfo {
                    domain: "grafana.tracker.local".to_string(),
                    internal_port: 3000,
                },
            ],
        );

        let domains = services.tls_domain_names();
        assert_eq!(domains.len(), 2);
        assert!(domains.contains(&"api.tracker.local"));
        assert!(domains.contains(&"grafana.tracker.local"));
    }

    #[test]
    fn it_should_return_unexposed_ports() {
        let services = ServiceInfo::new(
            vec![],
            vec![],
            vec![],
            "https://api.tracker.local/api".to_string(),
            true,
            String::new(),
            vec![
                TlsDomainInfo {
                    domain: "api.tracker.local".to_string(),
                    internal_port: 1212,
                },
                TlsDomainInfo {
                    domain: "http1.tracker.local".to_string(),
                    internal_port: 7070,
                },
            ],
        );

        let ports = services.unexposed_ports();
        assert_eq!(ports.len(), 2);
        assert!(ports.contains(&1212));
        assert!(ports.contains(&7070));
    }

    #[test]
    fn it_should_detect_no_tls_when_empty() {
        let services = ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec![],
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            "http://10.0.0.1:1212/api".to_string(),            // DevSkim: ignore DS137138
            false,
            "http://10.0.0.1:1313/health_check".to_string(), // DevSkim: ignore DS137138
            vec![],
        );

        assert!(!services.has_any_tls());
        assert!(services.tls_domain_names().is_empty());
        assert!(services.unexposed_ports().is_empty());
    }
}
