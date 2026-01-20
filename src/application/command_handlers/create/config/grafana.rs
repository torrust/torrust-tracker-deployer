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

use crate::shared::DomainName;

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
///
/// With TLS proxy configuration:
/// ```json
/// {
///     "admin_user": "admin",
///     "admin_password": "admin",
///     "domain": "grafana.example.com",
///     "use_tls_proxy": true
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

    /// Domain name for external HTTPS access (optional)
    ///
    /// When present, defines the domain at which Grafana will be accessible.
    /// Caddy uses this for automatic certificate management.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,

    /// Whether to use TLS proxy via Caddy (default: false)
    ///
    /// When true:
    /// - Caddy handles HTTPS termination with automatic certificates
    /// - Requires a domain to be configured
    /// - Grafana is accessed via HTTPS through Caddy
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub use_tls_proxy: Option<bool>,
}

impl Default for GrafanaSection {
    fn default() -> Self {
        let default_config = GrafanaConfig::default();
        Self {
            admin_user: default_config.admin_user().to_string(),
            admin_password: default_config.admin_password().expose_secret().to_string(),
            domain: None,
            use_tls_proxy: None,
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
    /// Returns `CreateConfigError::InvalidDomain` if the domain is invalid.
    /// Returns `CreateConfigError::TlsProxyWithoutDomain` if `use_tls_proxy`
    /// is true but no domain is provided.
    pub fn to_grafana_config(&self) -> Result<GrafanaConfig, CreateConfigError> {
        let use_tls_proxy = self.use_tls_proxy.unwrap_or(false);

        // Validate: use_tls_proxy requires domain
        if use_tls_proxy && self.domain.is_none() {
            return Err(CreateConfigError::TlsProxyWithoutDomain {
                service_type: "Grafana".to_string(),
                bind_address: "N/A (hardcoded port 3000)".to_string(),
            });
        }

        // Parse domain if present
        let domain =
            match &self.domain {
                Some(domain_str) => Some(DomainName::new(domain_str).map_err(|e| {
                    CreateConfigError::InvalidDomain {
                        domain: domain_str.clone(),
                        reason: e.to_string(),
                    }
                })?),
                None => None,
            };

        Ok(GrafanaConfig::new(
            self.admin_user.clone(),
            self.admin_password.clone(),
            domain,
            use_tls_proxy,
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
        assert!(section.domain.is_none());
        assert!(section.use_tls_proxy.is_none());
    }

    #[test]
    fn it_should_convert_to_grafana_config() {
        let section = GrafanaSection {
            admin_user: "custom_admin".to_string(),
            admin_password: "secure_password".to_string(),
            domain: None,
            use_tls_proxy: None,
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
            domain: None,
            use_tls_proxy: None,
        };

        let config = section.to_grafana_config().unwrap();
        let debug_output = format!("{config:?}");

        // Password should be redacted in debug output
        assert!(debug_output.contains("[REDACTED]"));
        assert!(!debug_output.contains("secret_password"));
    }

    #[test]
    fn it_should_convert_with_domain_and_tls_proxy() {
        let section = GrafanaSection {
            admin_user: "admin".to_string(),
            admin_password: "password".to_string(),
            domain: Some("grafana.example.com".to_string()),
            use_tls_proxy: Some(true),
        };

        let result = section.to_grafana_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.tls_domain(), Some("grafana.example.com"));
        assert!(config.use_tls_proxy());
    }

    #[test]
    fn it_should_convert_with_domain_without_tls_proxy() {
        let section = GrafanaSection {
            admin_user: "admin".to_string(),
            admin_password: "password".to_string(),
            domain: Some("grafana.example.com".to_string()),
            use_tls_proxy: Some(false),
        };

        let result = section.to_grafana_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(
            config.domain(),
            Some(&DomainName::new("grafana.example.com").unwrap())
        );
        assert!(!config.use_tls_proxy());
    }

    #[test]
    fn it_should_return_error_when_tls_proxy_enabled_without_domain() {
        let section = GrafanaSection {
            admin_user: "admin".to_string(),
            admin_password: "password".to_string(),
            domain: None,
            use_tls_proxy: Some(true),
        };

        let result = section.to_grafana_config();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(
            err,
            CreateConfigError::TlsProxyWithoutDomain { .. }
        ));
    }

    #[test]
    fn it_should_return_error_for_invalid_domain() {
        let section = GrafanaSection {
            admin_user: "admin".to_string(),
            admin_password: "password".to_string(),
            domain: Some(String::new()),
            use_tls_proxy: Some(true),
        };

        let result = section.to_grafana_config();
        assert!(result.is_err());

        let err = result.unwrap_err();
        assert!(matches!(err, CreateConfigError::InvalidDomain { .. }));
    }
}
