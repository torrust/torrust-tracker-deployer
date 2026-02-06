//! E2E runtime environment
//!
//! Represents the actual runtime state after container starts.

use super::config::E2eConfigEnvironment;
use super::container_ports::ContainerPorts;

/// E2E runtime environment - represents actual runtime state after container starts
///
/// This type contains the actual mapped ports returned by Docker when using
/// bridge networking mode. These may differ from the requested ports in the config.
#[derive(Debug, Clone)]
pub struct E2eRuntimeEnvironment {
    /// Configuration environment (what we requested)
    pub config: E2eConfigEnvironment,
    /// Actual mapped ports from Docker (what we got)
    pub container_ports: ContainerPorts,
}

impl E2eRuntimeEnvironment {
    /// Create a new runtime environment from config and container ports
    #[must_use]
    pub fn new(config: E2eConfigEnvironment, container_ports: ContainerPorts) -> Self {
        Self {
            config,
            container_ports,
        }
    }

    /// Get the SSH socket address using the mapped SSH port
    #[must_use]
    pub fn ssh_socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from(([127, 0, 0, 1], self.container_ports.ssh_port))
    }

    /// Get the tracker API URL for external access
    #[must_use]
    pub fn tracker_api_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.container_ports.http_api_port)
    }

    /// Get the HTTP tracker URL for external access
    #[must_use]
    pub fn http_tracker_url(&self) -> String {
        format!(
            "http://127.0.0.1:{}",
            self.container_ports.http_tracker_port
        )
    }
}
