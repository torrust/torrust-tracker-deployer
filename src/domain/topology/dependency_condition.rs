//! Dependency condition type for docker-compose service dependencies

/// Condition for service dependency (maps to docker-compose `depends_on` conditions)
///
/// Defines when a dependent service is considered ready for the waiting service to start.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyCondition {
    /// Service is healthy (passed health check)
    ///
    /// Maps to `condition: service_healthy` in docker-compose.
    /// The service must have a healthcheck defined and pass it.
    ServiceHealthy,

    /// Service has started (but may not be ready)
    ///
    /// Maps to `condition: service_started` in docker-compose.
    /// The service has been started but no guarantee it's ready.
    ServiceStarted,

    /// Service completed successfully
    ///
    /// Maps to `condition: service_completed_successfully` in docker-compose.
    /// Used for one-shot services that need to complete before dependents start.
    ServiceCompletedSuccessfully,
}

impl DependencyCondition {
    /// Returns the docker-compose string value for this condition
    ///
    /// # Example
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::topology::DependencyCondition;
    ///
    /// assert_eq!(DependencyCondition::ServiceHealthy.as_docker_compose_value(), "service_healthy");
    /// assert_eq!(DependencyCondition::ServiceStarted.as_docker_compose_value(), "service_started");
    /// ```
    #[must_use]
    pub const fn as_docker_compose_value(&self) -> &'static str {
        match self {
            Self::ServiceHealthy => "service_healthy",
            Self::ServiceStarted => "service_started",
            Self::ServiceCompletedSuccessfully => "service_completed_successfully",
        }
    }
}
