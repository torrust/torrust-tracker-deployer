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

use crate::domain::tracker::config::{HttpApiConfig, HttpTrackerConfig};
use crate::infrastructure::external_validators::ServiceEndpoint;

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
