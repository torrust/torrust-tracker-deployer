use std::net::SocketAddr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::tracker::HealthCheckApiConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct HealthCheckApiSection {
    pub bind_address: String,
}

impl HealthCheckApiSection {
    /// Converts this DTO to a domain `HealthCheckApiConfig`
    ///
    /// # Errors
    ///
    /// Returns `CreateConfigError::InvalidBindAddress` if the bind address cannot be parsed as a valid IP:PORT combination.
    /// Returns `CreateConfigError::DynamicPortNotSupported` if port 0 (dynamic port assignment) is specified.
    pub fn to_health_check_api_config(&self) -> Result<HealthCheckApiConfig, CreateConfigError> {
        // Validate that the bind address can be parsed as SocketAddr
        let bind_address = self.bind_address.parse::<SocketAddr>().map_err(|e| {
            CreateConfigError::InvalidBindAddress {
                address: self.bind_address.clone(),
                source: e,
            }
        })?;

        // Reject port 0 (dynamic port assignment)
        if bind_address.port() == 0 {
            return Err(CreateConfigError::DynamicPortNotSupported {
                bind_address: self.bind_address.clone(),
            });
        }

        Ok(HealthCheckApiConfig { bind_address })
    }
}

impl Default for HealthCheckApiSection {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:1313".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_convert_to_domain_config_when_bind_address_is_valid() {
        let section = HealthCheckApiSection {
            bind_address: "127.0.0.1:1313".to_string(),
        };

        let config = section.to_health_check_api_config().unwrap();

        assert_eq!(
            config.bind_address,
            "127.0.0.1:1313".parse::<SocketAddr>().unwrap()
        );
    }

    #[test]
    fn it_should_fail_when_bind_address_is_invalid() {
        let section = HealthCheckApiSection {
            bind_address: "invalid".to_string(),
        };

        let result = section.to_health_check_api_config();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::InvalidBindAddress { .. }
        ));
    }

    #[test]
    fn it_should_reject_dynamic_port_assignment() {
        let section = HealthCheckApiSection {
            bind_address: "0.0.0.0:0".to_string(),
        };

        let result = section.to_health_check_api_config();

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CreateConfigError::DynamicPortNotSupported { .. }
        ));
    }

    #[test]
    fn it_should_allow_ipv6_addresses() {
        let section = HealthCheckApiSection {
            bind_address: "[::1]:1313".to_string(),
        };

        let result = section.to_health_check_api_config();

        assert!(result.is_ok());
    }

    #[test]
    fn it_should_allow_any_port_except_zero() {
        let section = HealthCheckApiSection {
            bind_address: "127.0.0.1:8080".to_string(),
        };

        let result = section.to_health_check_api_config();

        assert!(result.is_ok());
    }

    #[test]
    fn it_should_provide_default_localhost_1313() {
        let section = HealthCheckApiSection::default();

        assert_eq!(section.bind_address, "127.0.0.1:1313");
    }
}
