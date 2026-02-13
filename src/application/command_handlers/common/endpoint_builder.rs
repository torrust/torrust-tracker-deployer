//! Service endpoint builder utilities
//!
//! This module provides utilities for constructing `ServiceEndpoint` objects
//! from domain configuration types and runtime deployment data (like instance IPs).
//!
//! These builders are used by command handlers that need to create endpoints for
//! external validation, testing, or other operations that require service URLs.
//!
//! ## Design Rationale
//!
//! These functions exist in the application layer (not domain) because:
//! - They combine domain configuration with runtime deployment state
//! - Domain configuration types shouldn't know about deployment IPs
//! - Multiple command handlers may need to build endpoints
//! - They translate domain types → infrastructure types (`ServiceEndpoint`)

use std::net::{IpAddr, SocketAddr};

use crate::domain::tracker::config::{HttpApiConfig, HttpTrackerConfig, TrackerConfig};
use crate::shared::ServiceEndpoint;

/// Build a `ServiceEndpoint` for the HTTP API from configuration and instance IP
///
/// Creates either an HTTP or HTTPS endpoint depending on whether TLS is enabled
/// in the configuration. For TLS endpoints, the domain is used with the instance
/// IP for local resolution (no DNS dependency).
///
/// # Arguments
///
/// * `instance_ip` - The IP address of the deployed instance
/// * `config` - The HTTP API configuration containing port and TLS settings
///
/// # Returns
///
/// A `ServiceEndpoint` configured for the HTTP API health check endpoint.
///
/// # Panics
///
/// Panics if the configuration produces an invalid URL (this should never happen
/// with valid configuration types from the domain layer).
#[must_use]
pub fn build_api_endpoint(instance_ip: IpAddr, config: &HttpApiConfig) -> ServiceEndpoint {
    let port = config.bind_address().port();
    let path = "/api/health_check";
    let socket_addr = SocketAddr::new(instance_ip, port);

    if let Some(domain) = config.tls_domain() {
        ServiceEndpoint::https(domain, path, instance_ip)
            .expect("Valid TLS domain should produce valid HTTPS URL")
    } else {
        ServiceEndpoint::http(socket_addr, path)
            .expect("Valid socket address should produce valid HTTP URL")
    }
}

/// Build a `ServiceEndpoint` for an HTTP Tracker from configuration and instance IP
///
/// Creates either an HTTP or HTTPS endpoint depending on whether TLS is enabled
/// in the configuration. For TLS endpoints, the domain is used with the instance
/// IP for local resolution (no DNS dependency).
///
/// # Arguments
///
/// * `instance_ip` - The IP address of the deployed instance
/// * `config` - The HTTP Tracker configuration containing port and TLS settings
///
/// # Returns
///
/// A `ServiceEndpoint` configured for the HTTP Tracker health check endpoint.
///
/// # Panics
///
/// Panics if the configuration produces an invalid URL (this should never happen
/// with valid configuration types from the domain layer).
#[must_use]
pub fn build_http_tracker_endpoint(
    instance_ip: IpAddr,
    config: &HttpTrackerConfig,
) -> ServiceEndpoint {
    let port = config.bind_address().port();
    let path = "/health_check";
    let socket_addr = SocketAddr::new(instance_ip, port);

    if let Some(domain) = config.tls_domain() {
        ServiceEndpoint::https(domain, path, instance_ip)
            .expect("Valid TLS domain should produce valid HTTPS URL")
    } else {
        ServiceEndpoint::http(socket_addr, path)
            .expect("Valid socket address should produce valid HTTP URL")
    }
}

/// Build all tracker service endpoints from configuration and instance IP
///
/// This is a convenience function that builds both the HTTP API endpoint and
/// all HTTP Tracker endpoints in a single call. It's the recommended way to
/// construct endpoints when you need all tracker services.
///
/// # Arguments
///
/// * `instance_ip` - The IP address of the deployed instance
/// * `tracker_config` - The complete tracker configuration
///
/// # Returns
///
/// A tuple containing:
/// - The HTTP API `ServiceEndpoint`
/// - A vector of HTTP Tracker `ServiceEndpoint`s (one per configured tracker)
///
/// # Panics
///
/// Panics if any configuration produces an invalid URL (this should never happen
/// with valid configuration types from the domain layer).
#[must_use]
pub fn build_all_tracker_endpoints(
    instance_ip: IpAddr,
    tracker_config: &TrackerConfig,
) -> (ServiceEndpoint, Vec<ServiceEndpoint>) {
    let api_endpoint = build_api_endpoint(instance_ip, tracker_config.http_api());

    let http_tracker_endpoints = tracker_config
        .http_trackers()
        .iter()
        .map(|config| build_http_tracker_endpoint(instance_ip, config))
        .collect();

    (api_endpoint, http_tracker_endpoints)
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::*;
    use crate::domain::tracker::config::{
        DatabaseConfig, HealthCheckApiConfig, HttpApiConfig, HttpTrackerConfig, SqliteConfig,
        TrackerConfig, TrackerCoreConfig, UdpTrackerConfig,
    };
    use crate::shared::{ApiToken, DomainName};

    // Test fixtures

    fn test_ip() -> IpAddr {
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))
    }

    fn http_api_config_without_tls() -> HttpApiConfig {
        HttpApiConfig::new(
            "0.0.0.0:1212".parse().unwrap(),
            ApiToken::from("test_token".to_string()),
            None,
            false,
        )
        .expect("valid config")
    }

    fn http_api_config_with_tls() -> HttpApiConfig {
        HttpApiConfig::new(
            "0.0.0.0:1212".parse().unwrap(),
            ApiToken::from("test_token".to_string()),
            Some(DomainName::new("api.tracker.local").unwrap()),
            true,
        )
        .expect("valid config")
    }

    fn http_tracker_config_without_tls() -> HttpTrackerConfig {
        HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), None, false).expect("valid config")
    }

    fn http_tracker_config_with_tls() -> HttpTrackerConfig {
        HttpTrackerConfig::new(
            "0.0.0.0:7070".parse().unwrap(),
            Some(DomainName::new("tracker.example.com").unwrap()),
            true,
        )
        .expect("valid config")
    }

    fn tracker_config_with_one_http_tracker() -> TrackerConfig {
        TrackerConfig::new(
            TrackerCoreConfig::new(
                DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), None).unwrap()],
            vec![http_tracker_config_without_tls()],
            http_api_config_without_tls(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .expect("valid config")
    }

    fn tracker_config_with_multiple_http_trackers() -> TrackerConfig {
        TrackerConfig::new(
            TrackerCoreConfig::new(
                DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![],
            vec![
                HttpTrackerConfig::new("0.0.0.0:7070".parse().unwrap(), None, false)
                    .expect("valid config"),
                HttpTrackerConfig::new("0.0.0.0:8080".parse().unwrap(), None, false)
                    .expect("valid config"),
                HttpTrackerConfig::new(
                    "0.0.0.0:9090".parse().unwrap(),
                    Some(DomainName::new("tracker.example.com").unwrap()),
                    true,
                )
                .expect("valid config"),
            ],
            http_api_config_with_tls(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .expect("valid config")
    }

    // Tests for build_api_endpoint

    #[test]
    fn it_should_build_http_api_endpoint_when_tls_is_disabled() {
        let config = http_api_config_without_tls();
        let endpoint = build_api_endpoint(test_ip(), &config);

        assert!(!endpoint.uses_tls());
        assert_eq!(endpoint.server_ip(), test_ip());
        assert_eq!(endpoint.port(), 1212);
        assert_eq!(
            endpoint.url().as_str(),
            "http://10.0.0.1:1212/api/health_check" // DevSkim: ignore DS137138
        );
    }

    #[test]
    fn it_should_build_https_api_endpoint_when_tls_is_enabled() {
        let config = http_api_config_with_tls();
        let endpoint = build_api_endpoint(test_ip(), &config);

        assert!(endpoint.uses_tls());
        assert_eq!(endpoint.server_ip(), test_ip());
        assert_eq!(endpoint.port(), 443);
        assert_eq!(endpoint.domain(), Some("api.tracker.local"));
        assert_eq!(
            endpoint.url().as_str(),
            "https://api.tracker.local/api/health_check"
        );
    }

    #[test]
    fn it_should_use_correct_path_when_building_api_endpoint() {
        let config = http_api_config_without_tls();
        let endpoint = build_api_endpoint(test_ip(), &config);

        assert_eq!(endpoint.url().path(), "/api/health_check");
    }

    #[test]
    fn it_should_extract_port_from_config_when_building_api_endpoint() {
        let config = HttpApiConfig::new(
            "0.0.0.0:9999".parse().unwrap(),
            ApiToken::from("test".to_string()),
            None,
            false,
        )
        .expect("valid config");

        let endpoint = build_api_endpoint(test_ip(), &config);

        assert_eq!(endpoint.port(), 9999);
    }

    #[test]
    fn it_should_use_instance_ip_when_building_api_endpoint() {
        let different_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let config = http_api_config_without_tls();

        let endpoint = build_api_endpoint(different_ip, &config);

        assert_eq!(endpoint.server_ip(), different_ip);
    }

    // Tests for build_http_tracker_endpoint

    #[test]
    fn it_should_build_http_tracker_endpoint_when_tls_is_disabled() {
        let config = http_tracker_config_without_tls();
        let endpoint = build_http_tracker_endpoint(test_ip(), &config);

        assert!(!endpoint.uses_tls());
        assert_eq!(endpoint.server_ip(), test_ip());
        assert_eq!(endpoint.port(), 7070);
        assert_eq!(
            endpoint.url().as_str(),
            "http://10.0.0.1:7070/health_check" // DevSkim: ignore DS137138
        );
    }

    #[test]
    fn it_should_build_https_tracker_endpoint_when_tls_is_enabled() {
        let config = http_tracker_config_with_tls();
        let endpoint = build_http_tracker_endpoint(test_ip(), &config);

        assert!(endpoint.uses_tls());
        assert_eq!(endpoint.server_ip(), test_ip());
        assert_eq!(endpoint.port(), 443);
        assert_eq!(endpoint.domain(), Some("tracker.example.com"));
        assert_eq!(
            endpoint.url().as_str(),
            "https://tracker.example.com/health_check"
        );
    }

    #[test]
    fn it_should_use_correct_path_when_building_tracker_endpoint() {
        let config = http_tracker_config_without_tls();
        let endpoint = build_http_tracker_endpoint(test_ip(), &config);

        assert_eq!(endpoint.url().path(), "/health_check");
    }

    #[test]
    fn it_should_extract_port_from_config_when_building_tracker_endpoint() {
        let config = HttpTrackerConfig::new("0.0.0.0:8888".parse().unwrap(), None, false)
            .expect("valid config");

        let endpoint = build_http_tracker_endpoint(test_ip(), &config);

        assert_eq!(endpoint.port(), 8888);
    }

    #[test]
    fn it_should_use_instance_ip_when_building_tracker_endpoint() {
        let different_ip = IpAddr::V4(Ipv4Addr::new(172, 16, 0, 50));
        let config = http_tracker_config_without_tls();

        let endpoint = build_http_tracker_endpoint(different_ip, &config);

        assert_eq!(endpoint.server_ip(), different_ip);
    }

    // Tests for build_all_tracker_endpoints

    #[test]
    fn it_should_build_api_and_tracker_endpoints_when_given_tracker_config() {
        let config = tracker_config_with_one_http_tracker();

        let (api_endpoint, tracker_endpoints) = build_all_tracker_endpoints(test_ip(), &config);

        // Verify API endpoint
        assert!(!api_endpoint.uses_tls());
        assert_eq!(api_endpoint.port(), 1212);
        assert_eq!(api_endpoint.url().path(), "/api/health_check");

        // Verify tracker endpoints
        assert_eq!(tracker_endpoints.len(), 1);
        assert!(!tracker_endpoints[0].uses_tls());
        assert_eq!(tracker_endpoints[0].port(), 7070);
        assert_eq!(tracker_endpoints[0].url().path(), "/health_check");
    }

    #[test]
    fn it_should_build_multiple_tracker_endpoints_when_multiple_trackers_configured() {
        let config = tracker_config_with_multiple_http_trackers();

        let (_api_endpoint, tracker_endpoints) = build_all_tracker_endpoints(test_ip(), &config);

        assert_eq!(tracker_endpoints.len(), 3);

        // First tracker (HTTP on 7070)
        assert!(!tracker_endpoints[0].uses_tls());
        assert_eq!(tracker_endpoints[0].port(), 7070);

        // Second tracker (HTTP on 8080)
        assert!(!tracker_endpoints[1].uses_tls());
        assert_eq!(tracker_endpoints[1].port(), 8080);

        // Third tracker (HTTPS on 9090 -> 443)
        assert!(tracker_endpoints[2].uses_tls());
        assert_eq!(tracker_endpoints[2].port(), 443);
        assert_eq!(tracker_endpoints[2].domain(), Some("tracker.example.com"));
    }

    #[test]
    fn it_should_build_tls_api_endpoint_when_tracker_config_has_tls_enabled() {
        let config = tracker_config_with_multiple_http_trackers();

        let (api_endpoint, _tracker_endpoints) = build_all_tracker_endpoints(test_ip(), &config);

        assert!(api_endpoint.uses_tls());
        assert_eq!(api_endpoint.domain(), Some("api.tracker.local"));
        assert_eq!(
            api_endpoint.url().as_str(),
            "https://api.tracker.local/api/health_check"
        );
    }

    #[test]
    fn it_should_return_empty_tracker_list_when_no_http_trackers_configured() {
        let config = TrackerConfig::new(
            TrackerCoreConfig::new(
                DatabaseConfig::Sqlite(SqliteConfig::new("tracker.db").unwrap()),
                false,
            ),
            vec![UdpTrackerConfig::new("0.0.0.0:6969".parse().unwrap(), None).unwrap()],
            vec![], // No HTTP trackers
            http_api_config_without_tls(),
            HealthCheckApiConfig::new("127.0.0.1:1313".parse().unwrap(), None, false).unwrap(),
        )
        .expect("valid config");

        let (_api_endpoint, tracker_endpoints) = build_all_tracker_endpoints(test_ip(), &config);

        assert!(tracker_endpoints.is_empty());
    }

    #[test]
    fn it_should_use_same_instance_ip_for_all_endpoints_when_building_all() {
        let config = tracker_config_with_multiple_http_trackers();
        let specific_ip = IpAddr::V4(Ipv4Addr::new(203, 0, 113, 42));

        let (api_endpoint, tracker_endpoints) = build_all_tracker_endpoints(specific_ip, &config);

        // All endpoints should use the same instance IP
        assert_eq!(api_endpoint.server_ip(), specific_ip);
        for endpoint in &tracker_endpoints {
            assert_eq!(endpoint.server_ip(), specific_ip);
        }
    }

    #[test]
    fn it_should_preserve_individual_port_configurations_when_building_all() {
        let config = tracker_config_with_multiple_http_trackers();

        let (_api_endpoint, tracker_endpoints) = build_all_tracker_endpoints(test_ip(), &config);

        // Each tracker should preserve its configured port (or use 443 for HTTPS)
        let ports: Vec<u16> = tracker_endpoints
            .iter()
            .map(ServiceEndpoint::port)
            .collect();

        // First two are HTTP (use configured ports), third is HTTPS (uses 443)
        assert_eq!(ports[0], 7070);
        assert_eq!(ports[1], 8080);
        assert_eq!(ports[2], 443); // HTTPS default port
    }
}
