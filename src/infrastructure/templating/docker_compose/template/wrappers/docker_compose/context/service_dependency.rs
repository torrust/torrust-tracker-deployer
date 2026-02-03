//! Service dependency representation for Docker Compose templates
//!
//! This module provides the `ServiceDependency` type for expressing service dependencies
//! in the docker-compose.yml template.

use serde::Serialize;

use crate::domain::topology::{
    DependencyCondition, Service, ServiceDependency as DomainServiceDependency,
};

/// Represents a service dependency for Docker Compose
///
/// Serializes to the format expected by docker-compose.yml `depends_on` long syntax.
/// Uses domain types (`Service`, `DependencyCondition`) which serialize to lowercase
/// strings for template compatibility.
///
/// ```yaml
/// depends_on:
///   mysql:
///     condition: service_healthy
/// ```
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::context::ServiceDependency;
/// use torrust_tracker_deployer_lib::domain::topology::{ServiceDependency as DomainServiceDependency, DependencyCondition, Service};
///
/// let domain_dep = DomainServiceDependency {
///     service: Service::MySQL,
///     condition: DependencyCondition::ServiceHealthy,
/// };
///
/// let dep = ServiceDependency::from(domain_dep);
/// assert_eq!(dep.service, Service::MySQL);
/// assert_eq!(dep.condition, DependencyCondition::ServiceHealthy);
/// ```
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ServiceDependency {
    /// Service to depend on (serializes to lowercase name)
    pub service: Service,
    /// Dependency condition (serializes to docker-compose format)
    pub condition: DependencyCondition,
}

impl From<DomainServiceDependency> for ServiceDependency {
    fn from(domain_dep: DomainServiceDependency) -> Self {
        Self {
            service: domain_dep.service,
            condition: domain_dep.condition,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::topology::Service;

    #[test]
    fn it_should_convert_from_domain_service_dependency_with_healthy_condition() {
        let domain_dep = DomainServiceDependency {
            service: Service::MySQL,
            condition: DependencyCondition::ServiceHealthy,
        };

        let dep = ServiceDependency::from(domain_dep);

        assert_eq!(dep.service, Service::MySQL);
        assert_eq!(dep.condition, DependencyCondition::ServiceHealthy);
    }

    #[test]
    fn it_should_convert_from_domain_service_dependency_with_started_condition() {
        let domain_dep = DomainServiceDependency {
            service: Service::Tracker,
            condition: DependencyCondition::ServiceStarted,
        };

        let dep = ServiceDependency::from(domain_dep);

        assert_eq!(dep.service, Service::Tracker);
        assert_eq!(dep.condition, DependencyCondition::ServiceStarted);
    }

    #[test]
    fn it_should_convert_from_domain_service_dependency_with_completed_condition() {
        let domain_dep = DomainServiceDependency {
            service: Service::Prometheus,
            condition: DependencyCondition::ServiceCompletedSuccessfully,
        };

        let dep = ServiceDependency::from(domain_dep);

        assert_eq!(dep.service, Service::Prometheus);
        assert_eq!(
            dep.condition,
            DependencyCondition::ServiceCompletedSuccessfully
        );
    }

    #[test]
    fn it_should_serialize_to_docker_compose_format() {
        let dep = ServiceDependency {
            service: Service::MySQL,
            condition: DependencyCondition::ServiceHealthy,
        };

        let json = serde_json::to_value(&dep).unwrap();

        assert_eq!(json["service"], "mysql");
        assert_eq!(json["condition"], "service_healthy");
    }
}
