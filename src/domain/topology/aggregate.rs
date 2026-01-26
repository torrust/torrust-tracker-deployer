//! Docker Compose Topology Aggregate
//!
//! This module defines the [`DockerComposeTopology`] aggregate that collects
//! all service topologies and derives the required networks and ports.
//!
//! ## Design
//!
//! The topology aggregate is the single source of truth for:
//! - Which services are enabled
//! - Which networks each service uses
//! - Which ports each service exposes
//! - The complete list of required networks (derived from services)
//!
//! ## Invariants
//!
//! - Every network used by a service is included in `required_networks()`
//! - No orphan networks (networks defined but not used by any service)
//! - Networks are returned in deterministic order
//! - Port bindings include descriptions for documentation

use std::collections::{HashMap, HashSet};

use super::error::PortConflict;
use super::network::Network;
use super::port::PortBinding;
use super::service::Service;

/// Topology information for a single service
///
/// Contains the service identifier, its network assignments, and port bindings.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::topology::{
///     ServiceTopology, Service, Network, PortBinding
/// };
///
/// // Service with ports
/// let topology = ServiceTopology::new(
///     Service::Tracker,
///     vec![Network::Database, Network::Metrics],
///     vec![PortBinding::udp(6969, "BitTorrent UDP announce")],
/// );
///
/// assert_eq!(topology.service(), Service::Tracker);
/// assert_eq!(topology.networks().len(), 2);
/// assert_eq!(topology.ports().len(), 1);
///
/// // Service without ports
/// let db_topology = ServiceTopology::with_networks(
///     Service::MySQL,
///     vec![Network::Database],
/// );
///
/// assert!(!db_topology.has_ports());
/// ```
#[derive(Debug, Clone)]
pub struct ServiceTopology {
    /// The service this topology describes
    service: Service,
    /// Networks this service is connected to
    networks: Vec<Network>,
    /// Ports this service exposes to the host
    ports: Vec<PortBinding>,
}

impl ServiceTopology {
    /// Creates a new service topology with networks and ports
    #[must_use]
    pub fn new(service: Service, networks: Vec<Network>, ports: Vec<PortBinding>) -> Self {
        Self {
            service,
            networks,
            ports,
        }
    }

    /// Creates a new service topology with networks only (no ports)
    ///
    /// Convenience constructor for services that don't expose ports
    /// (e.g., `MySQL` which is internal-only).
    #[must_use]
    pub fn with_networks(service: Service, networks: Vec<Network>) -> Self {
        Self::new(service, networks, vec![])
    }

    /// Returns the service this topology describes
    #[must_use]
    pub fn service(&self) -> Service {
        self.service
    }

    /// Returns the networks this service is connected to
    #[must_use]
    pub fn networks(&self) -> &[Network] {
        &self.networks
    }

    /// Returns the ports this service exposes
    #[must_use]
    pub fn ports(&self) -> &[PortBinding] {
        &self.ports
    }

    /// Returns whether this service exposes any ports
    #[must_use]
    pub fn has_ports(&self) -> bool {
        !self.ports.is_empty()
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
///     ServiceTopology::with_networks(Service::Tracker, vec![Network::Database, Network::Metrics]),
///     ServiceTopology::with_networks(Service::MySQL, vec![Network::Database]),
/// ]).expect("no port conflicts");
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
    ///
    /// Validates that no two services bind to the same host port.
    /// This enforces the "always valid" invariant - a `DockerComposeTopology`
    /// instance is guaranteed to have no port conflicts.
    ///
    /// # Errors
    ///
    /// Returns [`PortConflict`] if two services try to bind the same host port.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::topology::{
    ///     DockerComposeTopology, ServiceTopology, Service, PortBinding
    /// };
    ///
    /// // Valid: different ports
    /// let valid = DockerComposeTopology::new(vec![
    ///     ServiceTopology::new(Service::Tracker, vec![], vec![PortBinding::tcp(6969, "Tracker")]),
    ///     ServiceTopology::new(Service::Prometheus, vec![], vec![PortBinding::tcp(9090, "Prometheus")]),
    /// ]);
    /// assert!(valid.is_ok());
    ///
    /// // Invalid: same port
    /// let invalid = DockerComposeTopology::new(vec![
    ///     ServiceTopology::new(Service::Tracker, vec![], vec![PortBinding::tcp(9090, "A")]),
    ///     ServiceTopology::new(Service::Prometheus, vec![], vec![PortBinding::tcp(9090, "B")]),
    /// ]);
    /// assert!(invalid.is_err());
    /// ```
    pub fn new(services: Vec<ServiceTopology>) -> Result<Self, PortConflict> {
        let topology = Self { services };
        topology.validate_port_uniqueness_internal()?;
        Ok(topology)
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
            .flat_map(|s| s.networks().iter().copied())
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

    /// Validates that no two services bind to the same host port
    ///
    /// Docker Compose will fail at startup if two services try to bind
    /// to the same host port. This method detects such conflicts early,
    /// providing a clear error message.
    ///
    /// Note: Binding to different IPs (e.g., 127.0.0.1:8080 and 0.0.0.0:8080)
    /// is still considered a conflict since 0.0.0.0 includes all interfaces.
    ///
    /// Note: This method is primarily used internally by the constructor.
    /// Since `DockerComposeTopology` is "always valid" (validated at construction),
    /// a valid instance will always return `Ok(())`. This method remains available
    /// for cases where you need to re-validate or for testing purposes.
    ///
    /// # Errors
    ///
    /// Returns [`PortConflict`] when two services expose the same host port.
    /// The error includes details about both conflicting services and their
    /// port bindings, enabling actionable error messages.
    fn validate_port_uniqueness_internal(&self) -> Result<(), PortConflict> {
        // Track which service has bound each host port
        let mut port_bindings: HashMap<u16, (Service, PortBinding)> = HashMap::new();

        for service_topology in &self.services {
            for binding in service_topology.ports() {
                let host_port = binding.host_port();

                if let Some((first_service, first_binding)) = port_bindings.get(&host_port) {
                    return Err(PortConflict {
                        host_port,
                        first_service: *first_service,
                        first_binding: first_binding.clone(),
                        second_service: service_topology.service(),
                        second_binding: binding.clone(),
                    });
                }

                port_bindings.insert(host_port, (service_topology.service(), binding.clone()));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod service_topology {
        use super::*;

        #[test]
        fn it_should_create_service_topology_with_networks() {
            let topology = ServiceTopology::with_networks(
                Service::Tracker,
                vec![Network::Database, Network::Metrics],
            );

            assert_eq!(topology.service(), Service::Tracker);
            assert_eq!(topology.networks().len(), 2);
            assert!(topology.networks().contains(&Network::Database));
            assert!(topology.networks().contains(&Network::Metrics));
        }

        #[test]
        fn it_should_create_service_topology_with_empty_networks() {
            let topology = ServiceTopology::with_networks(Service::Tracker, vec![]);

            assert_eq!(topology.service(), Service::Tracker);
            assert!(topology.networks().is_empty());
        }

        #[test]
        fn it_should_create_service_topology_with_ports() {
            let topology = ServiceTopology::new(
                Service::Tracker,
                vec![Network::Metrics],
                vec![
                    PortBinding::udp(6969, "BitTorrent UDP announce"),
                    PortBinding::tcp(7070, "HTTP tracker announce"),
                ],
            );

            assert_eq!(topology.service(), Service::Tracker);
            assert_eq!(topology.ports().len(), 2);
            assert!(topology.has_ports());
        }

        #[test]
        fn it_should_have_no_ports_when_using_with_networks() {
            let topology = ServiceTopology::with_networks(Service::MySQL, vec![Network::Database]);

            assert!(topology.ports().is_empty());
            assert!(!topology.has_ports());
        }
    }

    mod required_networks {
        use super::*;

        #[test]
        fn it_should_derive_required_networks_from_all_services() {
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::with_networks(
                    Service::Tracker,
                    vec![Network::Database, Network::Metrics],
                ),
                ServiceTopology::with_networks(Service::MySQL, vec![Network::Database]),
                ServiceTopology::with_networks(Service::Prometheus, vec![Network::Metrics]),
            ])
            .unwrap();

            let required = topology.required_networks();

            assert!(required.contains(&Network::Database));
            assert!(required.contains(&Network::Metrics));
        }

        #[test]
        fn it_should_not_have_orphan_networks() {
            let topology = DockerComposeTopology::new(vec![ServiceTopology::with_networks(
                Service::Tracker,
                vec![Network::Metrics],
            )])
            .unwrap();

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
                ServiceTopology::with_networks(Service::Caddy, vec![Network::Proxy]),
                ServiceTopology::with_networks(Service::Tracker, vec![Network::Database]),
                ServiceTopology::with_networks(Service::Prometheus, vec![Network::Metrics]),
                ServiceTopology::with_networks(Service::Grafana, vec![Network::Visualization]),
            ])
            .unwrap();

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
                ServiceTopology::with_networks(Service::Tracker, vec![Network::Metrics]),
                ServiceTopology::with_networks(Service::Prometheus, vec![Network::Metrics]),
            ])
            .unwrap();

            let required = topology.required_networks();

            // Metrics appears twice but should only be in result once
            assert_eq!(required.len(), 1);
            assert!(required.contains(&Network::Metrics));
        }

        #[test]
        fn it_should_return_empty_when_no_services() {
            let topology = DockerComposeTopology::new(vec![]).unwrap();

            let required = topology.required_networks();

            assert!(required.is_empty());
        }

        #[test]
        fn it_should_return_empty_when_services_have_no_networks() {
            let topology = DockerComposeTopology::new(vec![ServiceTopology::with_networks(
                Service::Tracker,
                vec![],
            )])
            .unwrap();

            let required = topology.required_networks();

            assert!(required.is_empty());
        }
    }

    mod configuration_combinations {
        use super::*;

        #[test]
        fn it_should_configure_minimal_deployment() {
            // Tracker only, no optional services
            let topology = DockerComposeTopology::new(vec![ServiceTopology::with_networks(
                Service::Tracker,
                vec![],
            )])
            .unwrap();

            let required = topology.required_networks();

            assert!(required.is_empty());
        }

        #[test]
        fn it_should_configure_deployment_with_mysql() {
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::with_networks(Service::Tracker, vec![Network::Database]),
                ServiceTopology::with_networks(Service::MySQL, vec![Network::Database]),
            ])
            .unwrap();

            let required = topology.required_networks();

            assert_eq!(required.len(), 1);
            assert!(required.contains(&Network::Database));
        }

        #[test]
        fn it_should_configure_deployment_with_monitoring() {
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::with_networks(Service::Tracker, vec![Network::Metrics]),
                ServiceTopology::with_networks(
                    Service::Prometheus,
                    vec![Network::Metrics, Network::Visualization],
                ),
                ServiceTopology::with_networks(Service::Grafana, vec![Network::Visualization]),
            ])
            .unwrap();

            let required = topology.required_networks();

            assert_eq!(required.len(), 2);
            assert!(required.contains(&Network::Metrics));
            assert!(required.contains(&Network::Visualization));
        }

        #[test]
        fn it_should_configure_full_http_deployment() {
            let topology = DockerComposeTopology::new(vec![
                ServiceTopology::with_networks(
                    Service::Tracker,
                    vec![Network::Database, Network::Metrics],
                ),
                ServiceTopology::with_networks(Service::MySQL, vec![Network::Database]),
                ServiceTopology::with_networks(
                    Service::Prometheus,
                    vec![Network::Metrics, Network::Visualization],
                ),
                ServiceTopology::with_networks(Service::Grafana, vec![Network::Visualization]),
            ])
            .unwrap();

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
                ServiceTopology::with_networks(
                    Service::Tracker,
                    vec![Network::Database, Network::Metrics, Network::Proxy],
                ),
                ServiceTopology::with_networks(Service::MySQL, vec![Network::Database]),
                ServiceTopology::with_networks(
                    Service::Prometheus,
                    vec![Network::Metrics, Network::Visualization],
                ),
                ServiceTopology::with_networks(
                    Service::Grafana,
                    vec![Network::Visualization, Network::Proxy],
                ),
                ServiceTopology::with_networks(Service::Caddy, vec![Network::Proxy]),
            ])
            .unwrap();

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
                ServiceTopology::with_networks(Service::Tracker, vec![Network::Proxy]),
                ServiceTopology::with_networks(Service::Caddy, vec![Network::Proxy]),
            ])
            .unwrap();

            let required = topology.required_networks();

            assert_eq!(required.len(), 1);
            assert!(required.contains(&Network::Proxy));
        }
    }

    mod port_validation {
        use super::*;

        #[test]
        fn it_should_succeed_when_no_port_conflicts() {
            let result = DockerComposeTopology::new(vec![
                ServiceTopology::new(
                    Service::Tracker,
                    vec![],
                    vec![
                        PortBinding::udp(6969, "UDP announce"),
                        PortBinding::tcp(7070, "HTTP announce"),
                    ],
                ),
                ServiceTopology::new(
                    Service::Prometheus,
                    vec![],
                    vec![PortBinding::tcp(9090, "Web UI")],
                ),
            ]);

            assert!(result.is_ok());
        }

        #[test]
        fn it_should_fail_construction_when_same_host_port_bound_by_two_services() {
            let result = DockerComposeTopology::new(vec![
                ServiceTopology::new(
                    Service::Tracker,
                    vec![],
                    vec![PortBinding::tcp(9090, "Health check")],
                ),
                ServiceTopology::new(
                    Service::Prometheus,
                    vec![],
                    vec![PortBinding::tcp(9090, "Web UI")],
                ),
            ]);

            assert!(result.is_err());
            let conflict = result.unwrap_err();
            assert_eq!(conflict.host_port, 9090);
        }

        #[test]
        fn it_should_succeed_when_services_have_no_ports() {
            let result = DockerComposeTopology::new(vec![
                ServiceTopology::with_networks(Service::MySQL, vec![Network::Database]),
                ServiceTopology::with_networks(Service::Tracker, vec![Network::Database]),
            ]);

            assert!(result.is_ok());
        }

        #[test]
        fn it_should_succeed_when_empty_topology() {
            let result = DockerComposeTopology::new(vec![]);

            assert!(result.is_ok());
        }

        #[test]
        fn it_should_allow_same_container_port_on_different_host_ports() {
            // Both services use container port 80 but mapped to different host ports
            let result = DockerComposeTopology::new(vec![
                ServiceTopology::new(
                    Service::Tracker,
                    vec![],
                    vec![PortBinding::new(
                        8080,
                        80,
                        crate::domain::tracker::Protocol::Tcp,
                        None,
                        "HTTP",
                    )],
                ),
                ServiceTopology::new(
                    Service::Prometheus,
                    vec![],
                    vec![PortBinding::new(
                        9090,
                        80,
                        crate::domain::tracker::Protocol::Tcp,
                        None,
                        "HTTP",
                    )],
                ),
            ]);

            assert!(result.is_ok());
        }

        #[test]
        fn it_should_include_conflict_details_in_error() {
            let result = DockerComposeTopology::new(vec![
                ServiceTopology::new(
                    Service::Tracker,
                    vec![],
                    vec![PortBinding::tcp(9090, "Health check")],
                ),
                ServiceTopology::new(
                    Service::Prometheus,
                    vec![],
                    vec![PortBinding::tcp(9090, "Prometheus UI")],
                ),
            ]);

            let conflict = result.unwrap_err();

            assert_eq!(conflict.first_service, Service::Tracker);
            assert_eq!(conflict.second_service, Service::Prometheus);
            assert_eq!(conflict.first_binding.description(), "Health check");
            assert_eq!(conflict.second_binding.description(), "Prometheus UI");
        }
    }
}
