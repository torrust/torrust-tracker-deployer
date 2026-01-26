//! Traits for service topology participation
//!
//! This module defines traits that allow domain objects to participate
//! in Docker Compose topology construction.
//!
//! ## Design Principles
//!
//! - **Open/Closed Principle**: Port derivation is local to each service config.
//!   Adding a new service doesn't require modifying existing services.
//!
//! - **DDD Layer Separation**: Business rules for port exposure live in the domain,
//!   not in infrastructure template rendering code.
//!
//! - **Service-Local Logic**: Port derivation can be computed from a service's own
//!   configuration without knowledge of other services.

use super::enabled_services::EnabledServices;
use super::Network;
use super::PortBinding;

/// Trait for services that can derive their port bindings
///
/// This trait enables the Open/Closed principle: each service
/// encapsulates its own port derivation logic without requiring
/// changes to other services or the topology builder.
///
/// # Implementors
///
/// - [`TrackerConfig`](crate::domain::tracker::TrackerConfig) - derives ports based on TLS settings
/// - [`GrafanaConfig`](crate::domain::grafana::GrafanaConfig) - port 3000 only without TLS
/// - [`PrometheusConfig`](crate::domain::prometheus::PrometheusConfig) - port 9090 localhost-only
///
/// # Port Rules Reference
///
/// Each implementation applies the relevant PORT-* rules from the refactoring plan:
///
/// | Rule    | Service    | Description                                    |
/// |---------|------------|------------------------------------------------|
/// | PORT-02 | Tracker    | UDP ports always exposed (no TLS for UDP)      |
/// | PORT-03 | Tracker    | HTTP ports WITHOUT TLS exposed directly        |
/// | PORT-04 | Tracker    | HTTP ports WITH TLS NOT exposed (Caddy)        |
/// | PORT-05 | Tracker    | API exposed only when no TLS                   |
/// | PORT-06 | Tracker    | API NOT exposed when TLS                       |
/// | PORT-07 | Grafana    | Port 3000 exposed only without TLS             |
/// | PORT-08 | Grafana    | Port 3000 NOT exposed with TLS                 |
/// | PORT-10 | Prometheus | Port 9090 on localhost only                    |
pub trait PortDerivation {
    /// Derives port bindings based on service configuration
    ///
    /// The implementation should apply all PORT-* rules relevant
    /// to this service (e.g., hiding ports when TLS is enabled).
    ///
    /// # Returns
    ///
    /// A vector of [`PortBinding`] that should be exposed in Docker Compose.
    /// An empty vector means the service has no exposed ports.
    fn derive_ports(&self) -> Vec<PortBinding>;
}

/// Trait for services that can derive their network assignments
///
/// This trait enables domain-driven network computation: each service
/// determines which networks it needs based on its configuration and
/// the topology context (which other services are enabled).
///
/// # Network Rules Reference
///
/// Each implementation applies the relevant NET-* rules:
///
/// | Rule   | Service    | Description                                    |
/// |--------|------------|------------------------------------------------|
/// | NET-01 | Tracker    | Metrics network if Prometheus enabled          |
/// | NET-02 | Tracker    | Database network if MySQL enabled              |
/// | NET-03 | Tracker    | Proxy network if Caddy enabled                 |
/// | NET-04 | Prometheus | Metrics network always                         |
/// | NET-05 | Prometheus | Visualization network if Grafana enabled       |
/// | NET-06 | Grafana    | Visualization network always                   |
/// | NET-07 | Grafana    | Proxy network if Caddy enabled                 |
/// | NET-08 | MySQL      | Database network always                        |
/// | NET-09 | Caddy      | Proxy network always                           |
pub trait NetworkDerivation {
    /// Derives network assignments based on service configuration and topology
    ///
    /// # Arguments
    ///
    /// * `enabled_services` - Information about which other services are enabled
    ///
    /// # Returns
    ///
    /// A vector of [`Network`] that this service should be connected to.
    fn derive_networks(&self, enabled_services: &EnabledServices) -> Vec<Network>;
}
