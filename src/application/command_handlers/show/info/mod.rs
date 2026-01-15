//! Data Transfer Objects for environment information display
//!
//! These DTOs encapsulate the information extracted from environment state
//! for display purposes. They provide a clean separation between the domain
//! model and the presentation layer.
//!
//! # Module Structure
//!
//! Each service in the deployment stack has its own submodule:
//! - `tracker`: Tracker service information (UDP/HTTP trackers, API, health check)
//! - `prometheus`: Prometheus metrics service information
//! - `grafana`: Grafana visualization service information

mod grafana;
mod prometheus;
mod tracker;

use std::net::IpAddr;

use chrono::{DateTime, Utc};

pub use self::grafana::GrafanaInfo;
pub use self::prometheus::PrometheusInfo;
pub use self::tracker::{LocalhostServiceInfo, ServiceInfo, TlsDomainInfo};

/// Environment information for display purposes
///
/// This DTO contains all information about an environment that can be
/// displayed to the user. It is state-aware and contains optional fields
/// that are populated based on the environment's current state.
#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    /// Name of the environment
    pub name: String,

    /// Current state of the environment (e.g., "Created", "Provisioned", "Running")
    pub state: String,

    /// Provider name (e.g., "LXD", "Hetzner Cloud")
    pub provider: String,

    /// When the environment was created
    pub created_at: DateTime<Utc>,

    /// Infrastructure details, available after provisioning
    pub infrastructure: Option<InfrastructureInfo>,

    /// Tracker service information, available for Released/Running states
    pub services: Option<ServiceInfo>,

    /// Prometheus metrics service information, available for Released/Running states
    pub prometheus: Option<PrometheusInfo>,

    /// Grafana visualization service information, available for Released/Running states
    pub grafana: Option<GrafanaInfo>,

    /// Internal state name (e.g., "created", "provisioned") for guidance generation
    pub state_name: String,
}

impl EnvironmentInfo {
    /// Create a new `EnvironmentInfo`
    #[must_use]
    pub fn new(
        name: String,
        state: String,
        provider: String,
        created_at: DateTime<Utc>,
        state_name: String,
    ) -> Self {
        Self {
            name,
            state,
            provider,
            created_at,
            infrastructure: None,
            services: None,
            prometheus: None,
            grafana: None,
            state_name,
        }
    }

    /// Set infrastructure information
    #[must_use]
    pub fn with_infrastructure(mut self, infrastructure: InfrastructureInfo) -> Self {
        self.infrastructure = Some(infrastructure);
        self
    }

    /// Set service information
    #[must_use]
    pub fn with_services(mut self, services: ServiceInfo) -> Self {
        self.services = Some(services);
        self
    }

    /// Set Prometheus information
    #[must_use]
    pub fn with_prometheus(mut self, prometheus: PrometheusInfo) -> Self {
        self.prometheus = Some(prometheus);
        self
    }

    /// Set Grafana information
    #[must_use]
    pub fn with_grafana(mut self, grafana: GrafanaInfo) -> Self {
        self.grafana = Some(grafana);
        self
    }
}

/// Infrastructure details for an environment
///
/// This information is available after the environment has been provisioned.
#[derive(Debug, Clone)]
pub struct InfrastructureInfo {
    /// Instance IP address
    pub instance_ip: IpAddr,

    /// SSH port (typically 22)
    pub ssh_port: u16,

    /// SSH username for connecting to the instance
    pub ssh_user: String,

    /// Path to the SSH private key
    pub ssh_key_path: String,
}

impl InfrastructureInfo {
    /// Create a new `InfrastructureInfo`
    #[must_use]
    pub fn new(instance_ip: IpAddr, ssh_port: u16, ssh_user: String, ssh_key_path: String) -> Self {
        Self {
            instance_ip,
            ssh_port,
            ssh_user,
            ssh_key_path,
        }
    }

    /// Format the SSH connection command
    #[must_use]
    pub fn ssh_command(&self) -> String {
        if self.ssh_port == 22 {
            format!(
                "ssh -i {} {}@{}",
                self.ssh_key_path, self.ssh_user, self.instance_ip
            )
        } else {
            format!(
                "ssh -i {} -p {} {}@{}",
                self.ssh_key_path, self.ssh_port, self.ssh_user, self.instance_ip
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use chrono::{TimeZone, Utc};

    use super::*;

    #[test]
    fn it_should_create_environment_info() {
        let created_at = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let info = EnvironmentInfo::new(
            "test-env".to_string(),
            "Created".to_string(),
            "LXD".to_string(),
            created_at,
            "Run 'provision' to create infrastructure.".to_string(),
        );

        assert_eq!(info.name, "test-env");
        assert_eq!(info.state, "Created");
        assert_eq!(info.provider, "LXD");
        assert!(info.infrastructure.is_none());
    }

    #[test]
    fn it_should_add_infrastructure_info() {
        let created_at = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
        let info = EnvironmentInfo::new(
            "test-env".to_string(),
            "Provisioned".to_string(),
            "LXD".to_string(),
            created_at,
            "Run 'configure' to set up the system.".to_string(),
        )
        .with_infrastructure(InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(10, 140, 190, 14)),
            22,
            "ubuntu".to_string(),
            "/home/user/.ssh/key".to_string(),
        ));

        assert!(info.infrastructure.is_some());
        let infra = info.infrastructure.unwrap();
        assert_eq!(infra.ssh_port, 22);
        assert_eq!(infra.ssh_user, "ubuntu");
    }

    #[test]
    fn it_should_format_ssh_command_with_default_port() {
        let infra = InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(10, 140, 190, 14)),
            22,
            "ubuntu".to_string(),
            "/home/user/.ssh/key".to_string(),
        );

        assert_eq!(
            infra.ssh_command(),
            "ssh -i /home/user/.ssh/key ubuntu@10.140.190.14"
        );
    }

    #[test]
    fn it_should_format_ssh_command_with_custom_port() {
        let infra = InfrastructureInfo::new(
            IpAddr::V4(Ipv4Addr::new(10, 140, 190, 14)),
            2222,
            "root".to_string(),
            "/home/user/.ssh/key".to_string(),
        );

        assert_eq!(
            infra.ssh_command(),
            "ssh -i /home/user/.ssh/key -p 2222 root@10.140.190.14"
        );
    }
}
