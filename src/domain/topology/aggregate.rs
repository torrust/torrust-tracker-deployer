//! Docker Compose Topology Aggregate
//!
//! This module defines the [`DockerComposeTopology`] aggregate that collects
//! all service topologies and derives the required networks.
//!
//! ## Design
//!
//! The topology aggregate is the single source of truth for:
//! - Which services are enabled
//! - Which networks each service uses
//! - The complete list of required networks (derived from services)
//!
//! ## Invariants
//!
//! - Every network used by a service is included in `required_networks()`
//! - No orphan networks (networks defined but not used by any service)
//! - Networks are returned in deterministic order

use std::collections::HashSet;

use super::network::Network;
use super::service::Service;

/// Topology information for a single service
///
/// Contains the service identifier and its network assignments.
#[derive(Debug, Clone)]
pub struct ServiceTopology {
    /// The service this topology describes
    pub service: Service,
    /// Networks this service is connected to
    pub networks: Vec<Network>,
}

impl ServiceTopology {
    /// Creates a new service topology
    #[must_use]
    pub fn new(service: Service, networks: Vec<Network>) -> Self {
        Self { service, networks }
    }
}

/// Docker Compose deployment topology aggregate
///
/// This aggregate ensures all invariants are maintained:
/// - Networks used by services are derived and always defined
/// - No orphan networks exist
/// - Network lists are deterministic
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::topology::{
///     DockerComposeTopology, ServiceTopology, Service, Network
/// };
///
/// let topology = DockerComposeTopology::new(vec![
///     ServiceTopology::new(Service::Tracker, vec![Network::Database, Network::Metrics]),
///     ServiceTopology::new(Service::MySQL, vec![Network::Database]),
/// ]);
///
/// let required = topology.required_networks();
/// assert!(required.contains(&Network::Database));
/// assert!(required.contains(&Network::Metrics));
/// assert!(!required.contains(&Network::Visualization)); // Not used
/// ```
#[derive(Debug, Clone)]
pub struct DockerComposeTopology {
    /// All services in the deployment with their network assignments
    services: Vec<ServiceTopology>,
}

impl DockerComposeTopology {
    /// Creates a new topology aggregate from service topologies
    #[must_use]
    pub fn new(services: Vec<ServiceTopology>) -> Self {
        Self { services }
    }

    /// Returns all networks required by enabled services
    ///
    /// This is the single source of truth - the template's `networks:` section
    /// should iterate over this, not use conditionals.
    ///
    /// The networks are returned in deterministic alphabetical order by name
    /// to ensure template stability.
    ///
    /// # Invariants
    ///
    /// - Every network used by any service is included
    /// - No orphan networks (only networks actually used are returned)
    #[must_use]
    pub fn required_networks(&self) -> Vec<Network> {
        let unique: HashSet<Network> = self
            .services
            .iter()
            .flat_map(|s| s.networks.iter().copied())
            .collect();

        // Return in deterministic order for template stability
        let mut networks: Vec<Network> = unique.into_iter().collect();
        networks.sort_by_key(Network::name);
        networks
    }

    /// Returns the services in this topology
    #[must_use]
    pub fn services(&self) -> &[ServiceTopology] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod service_topology {
        use super::*;

        #[test]
        fn it_should_create_service_topology_with_networks() {
            let topology =
                ServiceTopology::new(Service::Tracker, vec![Network::Database, Network::Metrics]);

            assert_eq!(topology.service, Service::Tracker);
            assert_eq!(topology.networks.len(), 2);
            assert!(topology.networks.contains(&Network::Database));
            assert!(topology.networks.contains(&Network::Metrics));
        }

        #[test]
        fn it_should_create_service_topology_with_empty_networks() {
            let topology = ServiceTopology::new(Service::Tracker, vec![]);

            assert_eq!(topology.service, Service::Tracker);
            assert!(topology.networks.is_empty());
        }
    }

    mod required_networks {
        use super::*;

        #[test]
        fn it_should_derive_required_networks_from_all_services() {
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::new(Service::Tracker, vec![Network::Database, Network::Metrics]),
                ServiceTopology::new(Service::MySQL, vec![Network::Database]),
                ServiceTopology::new(Service::Prometheus, vec![Network::Metrics]),
            ]);

            let required = topology.required_networks();

            assert!(required.contains(&Network::Database));
            assert!(required.contains(&Network::Metrics));
        }

        #[test]
        fn it_should_not_have_orphan_networks() {
            let topology = DockerComposeTopology::new(vec![ServiceTopology::new(
                Service::Tracker,
                vec![Network::Metrics],
            )]);

            let required = topology.required_networks();

            // Only Metrics is used, so only Metrics should be in required
            assert_eq!(required.len(), 1);
            assert!(required.contains(&Network::Metrics));
            assert!(!required.contains(&Network::Database));
            assert!(!required.contains(&Network::Visualization));
            assert!(!required.contains(&Network::Proxy));
        }

        #[test]
        fn it_should_return_networks_in_deterministic_order() {
            // Create topology with networks added in non-alphabetical order
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::new(Service::Caddy, vec![Network::Proxy]),
                ServiceTopology::new(Service::Tracker, vec![Network::Database]),
                ServiceTopology::new(Service::Prometheus, vec![Network::Metrics]),
                ServiceTopology::new(Service::Grafana, vec![Network::Visualization]),
            ]);

            let required = topology.required_networks();

            // Should be sorted alphabetically by name
            let names: Vec<&str> = required.iter().map(Network::name).collect();
            assert_eq!(
                names,
                vec![
                    "database_network",
                    "metrics_network",
                    "proxy_network",
                    "visualization_network"
                ]
            );
        }

        #[test]
        fn it_should_deduplicate_networks_used_by_multiple_services() {
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::new(Service::Tracker, vec![Network::Metrics]),
                ServiceTopology::new(Service::Prometheus, vec![Network::Metrics]),
            ]);

            let required = topology.required_networks();

            // Metrics appears twice but should only be in result once
            assert_eq!(required.len(), 1);
            assert!(required.contains(&Network::Metrics));
        }

        #[test]
        fn it_should_return_empty_when_no_services() {
            let topology = DockerComposeTopology::new(vec![]);

            let required = topology.required_networks();

            assert!(required.is_empty());
        }

        #[test]
        fn it_should_return_empty_when_services_have_no_networks() {
            let topology =
                DockerComposeTopology::new(vec![ServiceTopology::new(Service::Tracker, vec![])]);

            let required = topology.required_networks();

            assert!(required.is_empty());
        }
    }

    mod configuration_combinations {
        use super::*;

        #[test]
        fn it_should_configure_minimal_deployment() {
            // Tracker only, no optional services
            let topology =
                DockerComposeTopology::new(vec![ServiceTopology::new(Service::Tracker, vec![])]);

            let required = topology.required_networks();

            assert!(required.is_empty());
        }

        #[test]
        fn it_should_configure_deployment_with_mysql() {
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::new(Service::Tracker, vec![Network::Database]),
                ServiceTopology::new(Service::MySQL, vec![Network::Database]),
            ]);

            let required = topology.required_networks();

            assert_eq!(required.len(), 1);
            assert!(required.contains(&Network::Database));
        }

        #[test]
        fn it_should_configure_deployment_with_monitoring() {
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::new(Service::Tracker, vec![Network::Metrics]),
                ServiceTopology::new(
                    Service::Prometheus,
                    vec![Network::Metrics, Network::Visualization],
                ),
                ServiceTopology::new(Service::Grafana, vec![Network::Visualization]),
            ]);

            let required = topology.required_networks();

            assert_eq!(required.len(), 2);
            assert!(required.contains(&Network::Metrics));
            assert!(required.contains(&Network::Visualization));
        }

        #[test]
        fn it_should_configure_full_http_deployment() {
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::new(Service::Tracker, vec![Network::Database, Network::Metrics]),
                ServiceTopology::new(Service::MySQL, vec![Network::Database]),
                ServiceTopology::new(
                    Service::Prometheus,
                    vec![Network::Metrics, Network::Visualization],
                ),
                ServiceTopology::new(Service::Grafana, vec![Network::Visualization]),
            ]);

            let required = topology.required_networks();

            assert_eq!(required.len(), 3);
            assert!(required.contains(&Network::Database));
            assert!(required.contains(&Network::Metrics));
            assert!(required.contains(&Network::Visualization));
            assert!(!required.contains(&Network::Proxy)); // No Caddy
        }

        #[test]
        fn it_should_configure_full_https_deployment() {
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::new(
                    Service::Tracker,
                    vec![Network::Database, Network::Metrics, Network::Proxy],
                ),
                ServiceTopology::new(Service::MySQL, vec![Network::Database]),
                ServiceTopology::new(
                    Service::Prometheus,
                    vec![Network::Metrics, Network::Visualization],
                ),
                ServiceTopology::new(
                    Service::Grafana,
                    vec![Network::Visualization, Network::Proxy],
                ),
                ServiceTopology::new(Service::Caddy, vec![Network::Proxy]),
            ]);

            let required = topology.required_networks();

            assert_eq!(required.len(), 4);
            assert!(required.contains(&Network::Database));
            assert!(required.contains(&Network::Metrics));
            assert!(required.contains(&Network::Visualization));
            assert!(required.contains(&Network::Proxy));
        }

        #[test]
        fn it_should_configure_https_minimal_deployment() {
            // Tracker + Caddy only (HTTPS with no monitoring or database)
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::new(Service::Tracker, vec![Network::Proxy]),
                ServiceTopology::new(Service::Caddy, vec![Network::Proxy]),
            ]);

            let required = topology.required_networks();

            assert_eq!(required.len(), 1);
            assert!(required.contains(&Network::Proxy));
        }
    }
}
