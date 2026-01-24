//! Timeout configurations for container operations
//!
//! This module provides timeout configurations for various container operations
//! to make the system more flexible and adaptable to different environments.

use std::time::Duration;

/// Container timeout configurations for different operations
///
/// This struct provides configurable timeouts for various container operations
/// to make the system more flexible and adaptable to different environments.
#[derive(Debug, Clone)]
pub struct ContainerTimeouts {
    /// Timeout for Docker image build operations
    pub docker_build: Duration,
    /// Timeout for container startup operations
    pub container_start: Duration,
    /// Timeout for SSH connectivity to become available
    pub ssh_ready: Duration,
    /// Timeout for SSH key setup operations
    pub ssh_setup: Duration,
}

impl Default for ContainerTimeouts {
    fn default() -> Self {
        Self {
            docker_build: Duration::from_mins(5),    // 5 minutes
            container_start: Duration::from_mins(1), // 1 minute
            ssh_ready: Duration::from_secs(30),      // 30 seconds
            ssh_setup: Duration::from_secs(15),      // 15 seconds
        }
    }
}
