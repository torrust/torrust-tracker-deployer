//! Enabled Services
//!
//! Provides a set of enabled services in a Docker Compose topology.
//! This is used by [`NetworkDerivation`](super::NetworkDerivation) implementations
//! to determine network assignments based on inter-service relationships.

use std::collections::HashSet;

use super::service::Service;

/// Set of enabled services in a topology
///
/// Provides information about which services are enabled in the topology.
/// This is needed because network assignments depend on inter-service
/// relationships (e.g., Tracker needs Metrics network only if Prometheus exists).
///
/// # Design
///
/// Uses a `HashSet<Service>` internally, following the Open/Closed principle:
/// adding new services doesn't require adding new fields or methods.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::topology::{EnabledServices, Service};
///
/// // Create with specific services enabled
/// let enabled = EnabledServices::from(&[Service::Prometheus, Service::Grafana]);
///
/// assert!(enabled.has(Service::Prometheus));
/// assert!(enabled.has(Service::Grafana));
/// assert!(!enabled.has(Service::Caddy));
/// assert!(!enabled.has(Service::MySQL));
/// ```
#[derive(Debug, Clone, Default)]
pub struct EnabledServices {
    /// Set of enabled services in this topology
    services: HashSet<Service>,
}

impl EnabledServices {
    /// Creates a new set of enabled services from a slice
    ///
    /// This is the primary constructor. The services slice represents
    /// which optional services are present in the deployment.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::topology::{EnabledServices, Service};
    ///
    /// let enabled = EnabledServices::from(&[Service::Prometheus, Service::MySQL]);
    ///
    /// assert!(enabled.has(Service::Prometheus));
    /// assert!(enabled.has(Service::MySQL));
    /// ```
    #[must_use]
    pub fn from(services: &[Service]) -> Self {
        Self {
            services: services.iter().copied().collect(),
        }
    }

    /// Checks if a specific service is enabled
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::topology::{EnabledServices, Service};
    ///
    /// let enabled = EnabledServices::from(&[Service::Caddy]);
    ///
    /// assert!(enabled.has(Service::Caddy));
    /// assert!(!enabled.has(Service::MySQL));
    /// ```
    #[must_use]
    pub fn has(&self, service: Service) -> bool {
        self.services.contains(&service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod from_constructor {
        use super::*;

        #[test]
        fn it_should_create_empty_set_when_given_empty_slice() {
            let enabled = EnabledServices::from(&[]);

            assert!(!enabled.has(Service::Prometheus));
            assert!(!enabled.has(Service::Grafana));
            assert!(!enabled.has(Service::Caddy));
            assert!(!enabled.has(Service::MySQL));
            assert!(!enabled.has(Service::Tracker));
        }

        #[test]
        fn it_should_create_set_with_single_service_when_given_one_service() {
            let enabled = EnabledServices::from(&[Service::Prometheus]);

            assert!(enabled.has(Service::Prometheus));
            assert!(!enabled.has(Service::Grafana));
        }

        #[test]
        fn it_should_create_set_with_multiple_services_when_given_multiple_services() {
            let enabled =
                EnabledServices::from(&[Service::Prometheus, Service::Grafana, Service::MySQL]);

            assert!(enabled.has(Service::Prometheus));
            assert!(enabled.has(Service::Grafana));
            assert!(enabled.has(Service::MySQL));
            assert!(!enabled.has(Service::Caddy));
            assert!(!enabled.has(Service::Tracker));
        }

        #[test]
        fn it_should_deduplicate_when_given_duplicate_services() {
            let enabled = EnabledServices::from(&[
                Service::Prometheus,
                Service::Prometheus,
                Service::Grafana,
            ]);

            assert!(enabled.has(Service::Prometheus));
            assert!(enabled.has(Service::Grafana));
        }

        #[test]
        fn it_should_include_all_service_types_when_given_all_services() {
            let enabled = EnabledServices::from(Service::all());

            for service in Service::all() {
                assert!(enabled.has(*service), "Expected {service:?} to be present");
            }
        }
    }

    mod has_method {
        use super::*;

        #[test]
        fn it_should_return_true_when_service_is_present() {
            let enabled = EnabledServices::from(&[Service::Caddy]);

            assert!(enabled.has(Service::Caddy));
        }

        #[test]
        fn it_should_return_false_when_service_is_absent() {
            let enabled = EnabledServices::from(&[Service::Prometheus]);

            assert!(!enabled.has(Service::Caddy));
        }

        #[test]
        fn it_should_return_false_for_all_services_when_empty() {
            let enabled = EnabledServices::from(&[]);

            for service in Service::all() {
                assert!(!enabled.has(*service), "Expected {service:?} to be absent");
            }
        }
    }

    mod default_trait {
        use super::*;

        #[test]
        fn it_should_create_empty_set_when_using_default() {
            let enabled = EnabledServices::default();

            for service in Service::all() {
                assert!(!enabled.has(*service), "Expected {service:?} to be absent");
            }
        }
    }

    mod clone_trait {
        use super::*;

        #[test]
        fn it_should_create_independent_copy_when_cloned() {
            let original = EnabledServices::from(&[Service::Prometheus, Service::Grafana]);
            let cloned = original.clone();

            assert!(cloned.has(Service::Prometheus));
            assert!(cloned.has(Service::Grafana));
            assert!(!cloned.has(Service::MySQL));
        }
    }
}
