//! Container port mappings for E2E testing
//!
//! Represents the actual mapped ports from Docker containers to the host.

/// Container port mappings resulting from Docker container setup
///
/// When Docker containers are created with port mappings, this struct
/// captures the actual host ports that were assigned.
#[derive(Debug, Clone)]
pub struct ContainerPorts {
    /// Mapped SSH port on the host
    pub ssh_port: u16,
    /// Mapped HTTP API port on the host
    pub http_api_port: u16,
    /// Mapped HTTP tracker port on the host
    pub http_tracker_port: u16,
    /// Mapped UDP tracker port on the host
    pub udp_tracker_port: u16,
}

impl ContainerPorts {
    /// Create container ports from a list of mapped ports
    ///
    /// # Arguments
    /// * `ssh_port` - Mapped SSH port
    /// * `additional_ports` - Mapped additional ports in order: [`http_api`, `http_tracker`, `udp_tracker`]
    ///
    /// # Panics
    /// Panics if `additional_ports` doesn't have exactly 3 additional ports
    #[must_use]
    pub fn from_mapped_ports(ssh_port: u16, additional_ports: &[u16]) -> Self {
        assert_eq!(
            additional_ports.len(),
            3,
            "Expected exactly 3 additional ports (http_api, http_tracker, udp_tracker)"
        );

        Self {
            ssh_port,
            http_api_port: additional_ports[0],
            http_tracker_port: additional_ports[1],
            udp_tracker_port: additional_ports[2],
        }
    }
}
