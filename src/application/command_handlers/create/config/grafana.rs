//! Grafana Configuration DTO (Application Layer)
//!
//! This module contains the DTO type for Grafana configuration used in
//! environment creation. This type uses raw primitives (String) for JSON
//! deserialization and converts to the rich domain type (`GrafanaConfig`).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::application::command_handlers::create::config::errors::CreateConfigError;
use crate::domain::grafana::GrafanaConfig;
use crate::shared::secrets::PlainPassword;

/// Grafana configuration section (DTO)
///
/// This is a DTO that deserializes from JSON strings and validates
/// when converting to the domain `GrafanaConfig`.
///
/// # Security
///
/// The `admin_password` field uses `PlainPassword` type alias for string at
/// DTO boundaries. It will be converted to `Password` (secrecy-wrapped) in
/// the domain layer.
///
/// # Examples
///
/// ```json
/// {
///     "admin_user": "admin",
///     "admin_password": "admin"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct GrafanaSection {
    /// Grafana admin username
    pub admin_user: String,

    /// Grafana admin password (plain string at DTO boundary)
    ///
    /// This will be converted to `Password` type in the domain layer
    /// to prevent accidental exposure in logs or debug output.
    pub admin_password: PlainPassword,
}

impl Default for GrafanaSection {
    fn default() -> Self {
        let default_config = GrafanaConfig::default();
        Self {
            admin_user: default_config.admin_user().to_string(),
            admin_password: default_config.admin_password().expose_secret().to_string(),
        }
    }
}

impl GrafanaSection {
    /// Converts this DTO to a domain `GrafanaConfig`
    ///
    /// This method performs validation and type conversion from the
    /// string-based DTO to the strongly-typed domain model with secrecy
    /// protection for the password.
    ///
    /// # Errors
    ///
    /// Currently returns `Ok` for all valid inputs. Future versions may
    /// add validation for `admin_user` format or password strength requirements.
    pub fn to_grafana_config(&self) -> Result<GrafanaConfig, CreateConfigError> {
        Ok(GrafanaConfig::new(
            self.admin_user.clone(),
            self.admin_password.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_have_default_values() {
        let section = GrafanaSection::default();
        assert_eq!(section.admin_user, "admin");
        assert_eq!(section.admin_password, "admin");
    }

    #[test]
    fn it_should_convert_to_grafana_config() {
        let section = GrafanaSection {
            admin_user: "custom_admin".to_string(),
            admin_password: "secure_password".to_string(),
        };

        let result = section.to_grafana_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.admin_user(), "custom_admin");
        assert_eq!(config.admin_password().expose_secret(), "secure_password");
    }

    #[test]
    fn it_should_convert_default_section_to_default_config() {
        let section = GrafanaSection::default();
        let result = section.to_grafana_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config, GrafanaConfig::default());
    }

    #[test]
    fn it_should_not_expose_password_in_debug_output() {
        let section = GrafanaSection {
            admin_user: "admin".to_string(),
            admin_password: "secret_password".to_string(),
        };

        let config = section.to_grafana_config().unwrap();
        let debug_output = format!("{config:?}");

        // Password should be redacted in debug output
        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains("secret_password"));
    }
}
