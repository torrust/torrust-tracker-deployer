//! Service dependency type for docker-compose

use super::dependency_condition::DependencyCondition;
use super::service::Service;

/// A service dependency with its condition
///
/// Represents that a service depends on another service being in a certain state
/// before it can start.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::topology::{ServiceDependency, DependencyCondition, Service};
///
/// let dep = ServiceDependency {
///     service: Service::MySQL,
///     condition: DependencyCondition::ServiceHealthy,
/// };
///
/// assert_eq!(dep.service, Service::MySQL);
/// assert_eq!(dep.condition, DependencyCondition::ServiceHealthy);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceDependency {
    /// The service being depended upon
    pub service: Service,

    /// The condition that must be met for the dependency
    pub condition: DependencyCondition,
}
