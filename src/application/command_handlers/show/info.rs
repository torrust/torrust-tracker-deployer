//! Data Transfer Objects for environment information display
//!
//! These DTOs encapsulate the information extracted from environment state
//! for display purposes. They provide a clean separation between the domain
//! model and the presentation layer.

use std::net::IpAddr;

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

    /// Infrastructure details, available after provisioning
    pub infrastructure: Option<InfrastructureInfo>,

    /// Tracker service information, available for Released/Running states
    pub services: Option<ServiceInfo>,

    /// Guidance for the next step based on current state
    pub next_step: String,
}

impl EnvironmentInfo {
    /// Create a new `EnvironmentInfo`
    #[must_use]
    pub fn new(name: String, state: String, provider: String, next_step: String) -> Self {
        Self {
            name,
            state,
            provider,
            infrastructure: None,
            services: None,
            next_step,
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

/// Tracker service information for display purposes
///
/// This information is available for Released and Running states and shows
/// the tracker services configured for the environment.
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// UDP tracker URLs (e.g., `udp://10.0.0.1:6969/announce`)
    pub udp_trackers: Vec<String>,

    /// HTTP tracker URLs (e.g., `http://10.0.0.1:7070/announce`)
    pub http_trackers: Vec<String>,

    /// HTTP API endpoint URL (e.g., `http://10.0.0.1:1212/api`)
    pub api_endpoint: String,

    /// Health check API URL (e.g., `http://10.0.0.1:1313/health_check`)
    pub health_check_url: String,
}

impl ServiceInfo {
    /// Create a new `ServiceInfo`
    #[must_use]
    pub fn new(
        udp_trackers: Vec<String>,
        http_trackers: Vec<String>,
        api_endpoint: String,
        health_check_url: String,
    ) -> Self {
        Self {
            udp_trackers,
            http_trackers,
            api_endpoint,
            health_check_url,
        }
    }

    /// Build `ServiceInfo` from tracker configuration and instance IP
    ///
    /// This method constructs service URLs by combining the configured bind
    /// addresses with the actual instance IP address.
    #[must_use]
    pub fn from_tracker_config(
        tracker_config: &crate::domain::tracker::TrackerConfig,
        instance_ip: IpAddr,
    ) -> Self {
        let udp_trackers = tracker_config
            .udp_trackers
            .iter()
            .map(|udp| format!("udp://{}:{}/announce", instance_ip, udp.bind_address.port()))
            .collect();

        let http_trackers = tracker_config
            .http_trackers
            .iter()
            .map(|http| {
                format!(
                    "http://{}:{}/announce", // DevSkim: ignore DS137138
                    instance_ip,
                    http.bind_address.port()
                )
            })
            .collect();

        let api_endpoint = format!(
            "http://{}:{}/api", // DevSkim: ignore DS137138
            instance_ip,
            tracker_config.http_api.bind_address.port()
        );

        let health_check_url = format!(
            "http://{}:{}/health_check", // DevSkim: ignore DS137138
            instance_ip,
            tracker_config.health_check_api.bind_address.port()
        );

        Self::new(udp_trackers, http_trackers, api_endpoint, health_check_url)
    }
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn it_should_create_environment_info() {
        let info = EnvironmentInfo::new(
            "test-env".to_string(),
            "Created".to_string(),
            "LXD".to_string(),
            "Run 'provision' to create infrastructure.".to_string(),
        );

        assert_eq!(info.name, "test-env");
        assert_eq!(info.state, "Created");
        assert_eq!(info.provider, "LXD");
        assert!(info.infrastructure.is_none());
    }

    #[test]
    fn it_should_add_infrastructure_info() {
        let info = EnvironmentInfo::new(
            "test-env".to_string(),
            "Provisioned".to_string(),
            "LXD".to_string(),
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

    #[test]
    fn it_should_create_service_info() {
        let services = ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            "http://10.0.0.1:1212/api".to_string(),            // DevSkim: ignore DS137138
            "http://10.0.0.1:1313/health_check".to_string(),   // DevSkim: ignore DS137138
        );

        assert_eq!(services.udp_trackers.len(), 1);
        assert_eq!(services.http_trackers.len(), 1);
        assert!(services.api_endpoint.contains("1212"));
        assert!(services.health_check_url.contains("1313"));
    }

    #[test]
    fn it_should_add_services_to_environment_info() {
        let info = EnvironmentInfo::new(
            "test-env".to_string(),
            "Running".to_string(),
            "LXD".to_string(),
            "Services are running.".to_string(),
        )
        .with_services(ServiceInfo::new(
            vec!["udp://10.0.0.1:6969/announce".to_string()],
            vec!["http://10.0.0.1:7070/announce".to_string()], // DevSkim: ignore DS137138
            "http://10.0.0.1:1212/api".to_string(),            // DevSkim: ignore DS137138
            "http://10.0.0.1:1313/health_check".to_string(),   // DevSkim: ignore DS137138
        ));

        assert!(info.services.is_some());
        let services = info.services.unwrap();
        assert_eq!(services.udp_trackers.len(), 1);
    }
}
