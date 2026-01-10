//! Tracker service information for display purposes
//!
//! This module contains DTOs for the Torrust Tracker service endpoints.

use std::net::IpAddr;

use crate::domain::environment::runtime_outputs::ServiceEndpoints;
use crate::domain::tracker::TrackerConfig;

/// Tracker service information for display purposes
///
/// This information is available for Released and Running states and shows
/// the tracker services configured for the environment.
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// UDP tracker URLs (e.g., `udp://10.0.0.1:6969/announce`)
    pub udp_trackers: Vec<String>,

    /// HTTP tracker URLs (e.g., `http://10.0.0.1:7070/announce`)
    pub http_trackers: Vec<String>,

    /// HTTP API endpoint URL (e.g., `http://10.0.0.1:1212/api`)
    pub api_endpoint: String,

    /// Health check API URL (e.g., `http://10.0.0.1:1313/health_check`)
    pub health_check_url: String,
}

impl ServiceInfo {
    /// Create a new `ServiceInfo`
    #[must_use]
    pub fn new(
        udp_trackers: Vec<String>,
        http_trackers: Vec<String>,
        api_endpoint: String,
        health_check_url: String,
    ) -> Self {
        Self {
            udp_trackers,
            http_trackers,
            api_endpoint,
            health_check_url,
        }
    }

    /// Build `ServiceInfo` from tracker configuration and instance IP
    ///
    /// This method constructs service URLs by combining the configured bind
    /// addresses with the actual instance IP address.
    #[must_use]
    pub fn from_tracker_config(tracker_config: &TrackerConfig, instance_ip: IpAddr) -> Self {
        let udp_trackers = tracker_config
            .udp_trackers
            .iter()
            .map(|udp| format!("udp://{}:{}/announce", instance_ip, udp.bind_address.port()))
            .collect();

        let http_trackers = tracker_config
            .http_trackers
            .iter()
            .map(|http| {
                format!(
                    "http://{}:{}/announce", // DevSkim: ignore DS137138
                    instance_ip,
                    http.bind_address.port()
                )
            })
            .collect();

        let api_endpoint = format!(
            "http://{}:{}/api", // DevSkim: ignore DS137138
            instance_ip,
            tracker_config.http_api.bind_address.port()
        );

        let health_check_url = format!(
            "http://{}:{}/health_check", // DevSkim: ignore DS137138
            instance_ip,
            tracker_config.health_check_api.bind_address.port()
        );

        Self::new(udp_trackers, http_trackers, api_endpoint, health_check_url)
    }

    /// Build `ServiceInfo` from stored `ServiceEndpoints`
    ///
    /// This method extracts service URLs from the runtime outputs
    /// that were stored when services were started.
    #[must_use]
    pub fn from_service_endpoints(endpoints: &ServiceEndpoints) -> Self {
        let udp_trackers = endpoints
            .udp_trackers
            .iter()
            .map(ToString::to_string)
            .collect();

        let http_trackers = endpoints
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

        Self::new(udp_trackers, http_trackers, api_endpoint, health_check_url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_service_info() {
        let services = ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            "http://10.0.0.1:1212/api".to_string(),            // DevSkim: ignore DS137138
            "http://10.0.0.1:1313/health_check".to_string(),   // DevSkim: ignore DS137138
        );

        assert_eq!(services.udp_trackers.len(), 1);
        assert_eq!(services.http_trackers.len(), 1);
        assert!(services.api_endpoint.contains("1212"));
        assert!(services.health_check_url.contains("1313"));
    }
}
