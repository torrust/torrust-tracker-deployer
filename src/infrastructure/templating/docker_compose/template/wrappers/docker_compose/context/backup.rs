//! Backup service configuration for Docker Compose
//!
//! This module defines the backup service configuration for the docker-compose.yml template.
//!
//! ## Note on Configuration Separation
//!
//! There are multiple backup-related types:
//!
//! - `BackupConfig` (domain): Contains backup settings like schedule, retention, database type
//!
//! - `BackupServiceContext` (this module): Contains service definition settings like networks
//!   and dependencies, following the same pattern as other service contexts
//!
//! - `BackupContext` (in backup template infrastructure): Context for rendering backup.conf.tera
//!
//! This separation keeps the pattern consistent across all services - each service
//! has its own context type for Docker Compose service topology.

use serde::Serialize;

use crate::domain::backup::BackupConfig as DomainBackupConfig;
use crate::domain::topology::{
    DependencyDerivation, EnabledServices, NetworkDerivation, PortDerivation,
};

use super::port_definition::PortDefinition;
use super::service_dependency::ServiceDependency;
use super::service_topology::ServiceTopology;

/// Backup service configuration for Docker Compose
///
/// Contains configuration for the backup service definition in docker-compose.yml.
/// Uses `ServiceTopology` to share the common topology structure with other services.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::docker_compose::context::BackupServiceContext;
/// use torrust_tracker_deployer_lib::domain::backup::{BackupConfig, RetentionDays, CronSchedule};
/// use torrust_tracker_deployer_lib::domain::topology::{EnabledServices, Service};
///
/// // SQLite backup (no dependencies, no networks)
/// let backup_config = BackupConfig::new(
///     CronSchedule::default(),
///     RetentionDays::default(),
/// );
/// let enabled_services = EnabledServices::from(&[]);
/// let backup = BackupServiceContext::from_domain_config(&backup_config, &enabled_services);
/// assert!(backup.networks().is_empty());
/// assert!(backup.dependencies().is_empty());
///
/// // MySQL backup (depends on MySQL, uses Database network)
/// let mysql_backup_config = BackupConfig::new(
///     CronSchedule::default(),
///     RetentionDays::default(),
/// );
/// let mysql_enabled = EnabledServices::from(&[Service::MySQL]);
/// let mysql_backup = BackupServiceContext::from_domain_config(&mysql_backup_config, &mysql_enabled);
/// assert!(!mysql_backup.networks().is_empty());
/// assert!(!mysql_backup.dependencies().is_empty());
/// ```
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BackupServiceContext {
    /// Service topology (ports and networks)
    ///
    /// Flattened for template compatibility - serializes ports/networks at top level.
    #[serde(flatten)]
    pub topology: ServiceTopology,

    /// Service dependencies (e.g., `MySQL` must be healthy before backup)
    ///
    /// For `MySQL` backups, the backup service depends on the `MySQL` service being healthy.
    /// For `SQLite` backups, there are no dependencies (file-based).
    pub dependencies: Vec<ServiceDependency>,
}

impl BackupServiceContext {
    /// Creates a new `BackupServiceContext` from domain configuration
    ///
    /// Uses the domain `PortDerivation`, `NetworkDerivation`, and `DependencyDerivation` traits,
    /// ensuring business rules live in the domain layer.
    ///
    /// # Arguments
    ///
    /// * `config` - The domain backup configuration
    /// * `enabled_services` - Topology context with information about enabled services
    #[must_use]
    pub fn from_domain_config(
        config: &DomainBackupConfig,
        enabled_services: &EnabledServices,
    ) -> Self {
        let port_bindings = config.derive_ports();
        let ports = port_bindings.iter().map(PortDefinition::from).collect();
        let networks = config.derive_networks(enabled_services);
        let dependencies = config
            .derive_dependencies(enabled_services)
            .into_iter()
            .map(ServiceDependency::from)
            .collect();

        Self {
            topology: ServiceTopology::new(ports, networks),
            dependencies,
        }
    }

    /// Returns the networks for this service
    #[must_use]
    pub fn networks(&self) -> &[crate::domain::topology::Network] {
        &self.topology.networks
    }

    /// Returns the ports for this service
    #[must_use]
    pub fn ports(&self) -> &[PortDefinition] {
        &self.topology.ports
    }

    /// Returns the dependencies for this service
    #[must_use]
    pub fn dependencies(&self) -> &[ServiceDependency] {
        &self.dependencies
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::backup::{BackupConfig, CronSchedule, RetentionDays};
    use crate::domain::topology::{DependencyCondition, Network, Service};

    #[test]
    fn it_should_have_no_exposed_ports() {
        let config = BackupConfig::new(CronSchedule::default(), RetentionDays::default());
        let enabled_services = EnabledServices::from(&[]);

        let backup = BackupServiceContext::from_domain_config(&config, &enabled_services);

        assert!(backup.ports().is_empty());
    }

    #[test]
    fn it_should_not_use_networks_for_sqlite_backup() {
        let config = BackupConfig::new(CronSchedule::default(), RetentionDays::default());
        let enabled_services = EnabledServices::from(&[]);

        let backup = BackupServiceContext::from_domain_config(&config, &enabled_services);

        assert!(backup.networks().is_empty());
    }

    #[test]
    fn it_should_use_database_network_for_mysql_backup() {
        let config = BackupConfig::new(CronSchedule::default(), RetentionDays::default());
        let enabled_services = EnabledServices::from(&[Service::MySQL]);

        let backup = BackupServiceContext::from_domain_config(&config, &enabled_services);

        assert_eq!(backup.networks(), &[Network::Database]);
    }

    #[test]
    fn it_should_have_no_dependencies_for_sqlite_backup() {
        let config = BackupConfig::new(CronSchedule::default(), RetentionDays::default());
        let enabled_services = EnabledServices::from(&[]);

        let backup = BackupServiceContext::from_domain_config(&config, &enabled_services);

        assert!(backup.dependencies().is_empty());
    }

    #[test]
    fn it_should_depend_on_mysql_service_for_mysql_backup() {
        let config = BackupConfig::new(CronSchedule::default(), RetentionDays::default());
        let enabled_services = EnabledServices::from(&[Service::MySQL]);

        let backup = BackupServiceContext::from_domain_config(&config, &enabled_services);

        assert_eq!(backup.dependencies().len(), 1);
        let dep = &backup.dependencies()[0];
        assert_eq!(dep.service, Service::MySQL);
        assert_eq!(dep.condition, DependencyCondition::ServiceHealthy);
    }

    #[test]
    fn it_should_serialize_with_flattened_topology() {
        let config = BackupConfig::new(CronSchedule::default(), RetentionDays::default());
        let enabled_services = EnabledServices::from(&[Service::MySQL]);

        let backup = BackupServiceContext::from_domain_config(&config, &enabled_services);
        let json = serde_json::to_value(&backup).unwrap();

        // Check that topology fields are at top level (not nested under "topology")
        assert!(
            json.get("topology").is_none(),
            "topology should be flattened"
        );
        assert!(
            json.get("networks").is_some(),
            "networks should be at top level"
        );
        assert!(json.get("ports").is_some(), "ports should be at top level");
        assert!(
            json.get("dependencies").is_some(),
            "dependencies should be at top level"
        );
    }
}
