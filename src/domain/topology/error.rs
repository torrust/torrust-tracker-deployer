//! Topology validation errors
//!
//! This module defines errors that can occur when validating the Docker
//! Compose topology, particularly around port conflicts.

use std::fmt;

use super::port::PortBinding;
use super::service::Service;

/// Error indicating a port conflict between services
///
/// Occurs when two services expose the same host port, which would cause
/// a bind error when starting the Docker Compose stack.
#[derive(Debug, Clone)]
pub struct PortConflict {
    /// The host port that is bound multiple times
    pub host_port: u16,
    /// The first service binding this port
    pub first_service: Service,
    /// The port binding from the first service
    pub first_binding: PortBinding,
    /// The second service (conflicting) binding this port
    pub second_service: Service,
    /// The port binding from the second service
    pub second_binding: PortBinding,
}

impl fmt::Display for PortConflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Port {} is bound by both {} ({}) and {} ({})",
            self.host_port,
            self.first_service.name(),
            self.first_binding.docker_compose_binding(),
            self.second_service.name(),
            self.second_binding.docker_compose_binding(),
        )
    }
}

impl std::error::Error for PortConflict {}

/// Errors that can occur when building or validating a topology
#[derive(Debug)]
pub enum TopologyError {
    /// A port conflict was detected between two services
    PortConflict(PortConflict),
}

impl fmt::Display for TopologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TopologyError::PortConflict(conflict) => write!(f, "{conflict}"),
        }
    }
}

impl std::error::Error for TopologyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TopologyError::PortConflict(conflict) => Some(conflict),
        }
    }
}

impl From<PortConflict> for TopologyError {
    fn from(conflict: PortConflict) -> Self {
        TopologyError::PortConflict(conflict)
    }
}

impl TopologyError {
    /// Returns guidance on how to resolve this error
    ///
    /// Following the DDD practices for expressive error types, this method
    /// explains what went wrong and suggests how to fix it.
    #[must_use]
    pub fn help(&self) -> String {
        match self {
            TopologyError::PortConflict(conflict) => {
                format!(
                    "Two services are trying to bind to the same host port {}. \
                     Either change the host port for '{}' or '{}', \
                     or modify the configuration to avoid the conflict. \
                     Docker cannot start if two services bind the same port.",
                    conflict.host_port,
                    conflict.first_service.name(),
                    conflict.second_service.name(),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::tracker::Protocol;

    use super::*;

    #[test]
    fn it_should_format_port_conflict_with_both_services() {
        let conflict = PortConflict {
            host_port: 9090,
            first_service: Service::Tracker,
            first_binding: PortBinding::tcp(9090, "Health check"),
            second_service: Service::Prometheus,
            second_binding: PortBinding::tcp(9090, "Web UI"),
        };

        let message = conflict.to_string();

        assert!(message.contains("9090"));
        assert!(message.contains("tracker"));
        assert!(message.contains("prometheus"));
    }

    #[test]
    fn it_should_include_protocol_in_conflict_message_for_udp() {
        let conflict = PortConflict {
            host_port: 6969,
            first_service: Service::Tracker,
            first_binding: PortBinding::new(6969, 6969, Protocol::Udp, None, "UDP announce"),
            second_service: Service::Tracker,
            second_binding: PortBinding::new(6969, 6970, Protocol::Udp, None, "Another UDP"),
        };

        let message = conflict.to_string();

        assert!(message.contains("6969:6969/udp"));
        assert!(message.contains("6969:6970/udp"));
    }

    #[test]
    fn it_should_provide_help_for_port_conflict_resolution() {
        let conflict = PortConflict {
            host_port: 9090,
            first_service: Service::Tracker,
            first_binding: PortBinding::tcp(9090, "Health check"),
            second_service: Service::Prometheus,
            second_binding: PortBinding::tcp(9090, "Web UI"),
        };

        let error = TopologyError::from(conflict);
        let help = error.help();

        assert!(
            help.contains("9090"),
            "Help should mention the conflicting port"
        );
        assert!(
            help.contains("tracker"),
            "Help should mention the first service"
        );
        assert!(
            help.contains("prometheus"),
            "Help should mention the second service"
        );
        assert!(
            help.contains("change") || help.contains("modify"),
            "Help should suggest a fix"
        );
    }
}
