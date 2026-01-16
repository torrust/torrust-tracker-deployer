//! Grafana configuration domain type

use serde::{Deserialize, Serialize};

use crate::domain::tls::TlsConfig;
use crate::shared::secrets::Password;

/// Grafana metrics visualization configuration
///
/// Configures Grafana service for displaying tracker metrics.
/// Grafana requires Prometheus to be enabled for metrics visualization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GrafanaConfig {
    /// Grafana admin username
    admin_user: String,

    /// Grafana admin password (should be changed in production)
    ///
    /// Uses `Password` wrapper from secrecy crate for secure handling:
    /// - Automatic redaction in debug output (shows `[REDACTED]`)
    /// - Memory zeroing when the value is dropped
    /// - Explicit `.expose_secret()` calls required to access plaintext
    admin_password: Password,

    /// TLS configuration for HTTPS termination via Caddy (optional)
    ///
    /// When present, Grafana will be accessible via HTTPS through
    /// the Caddy reverse proxy.
    #[serde(skip_serializing_if = "Option::is_none")]
    tls: Option<TlsConfig>,
}

impl GrafanaConfig {
    /// Creates a new Grafana configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::grafana::GrafanaConfig;
    ///
    /// let config = GrafanaConfig::new("admin".to_string(), "password".to_string());
    /// assert_eq!(config.admin_user(), "admin");
    /// ```
    #[must_use]
    pub fn new(admin_user: String, admin_password: String) -> Self {
        Self {
            admin_user,
            admin_password: Password::new(admin_password),
            tls: None,
        }
    }

    /// Creates a new Grafana configuration with TLS
    #[must_use]
    pub fn with_tls(admin_user: String, admin_password: String, tls: TlsConfig) -> Self {
        Self {
            admin_user,
            admin_password: Password::new(admin_password),
            tls: Some(tls),
        }
    }

    /// Returns the admin username
    #[must_use]
    pub fn admin_user(&self) -> &str {
        &self.admin_user
    }

    /// Returns the admin password
    #[must_use]
    pub fn admin_password(&self) -> &Password {
        &self.admin_password
    }

    /// Returns the TLS domain if configured
    #[must_use]
    pub fn tls_domain(&self) -> Option<&str> {
        self.tls.as_ref().map(TlsConfig::domain)
    }

    /// Returns the TLS configuration if present
    #[must_use]
    pub fn tls(&self) -> Option<&TlsConfig> {
        self.tls.as_ref()
    }
}

impl Default for GrafanaConfig {
    fn default() -> Self {
        Self {
            admin_user: "admin".to_string(),
            admin_password: Password::new("admin"),
            tls: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_grafana_config_with_default_values() {
        let config = GrafanaConfig::default();

        assert_eq!(config.admin_user, "admin");
        assert_eq!(config.admin_password.expose_secret(), "admin");
    }

    #[test]
    fn it_should_create_grafana_config_with_custom_values() {
        let config = GrafanaConfig {
            admin_user: "custom_admin".to_string(),
            admin_password: Password::new("custom_pass"),
            tls: None,
        };

        assert_eq!(config.admin_user, "custom_admin");
        assert_eq!(config.admin_password.expose_secret(), "custom_pass");
    }

    #[test]
    fn it_should_serialize_grafana_config_to_json() {
        let config = GrafanaConfig {
            admin_user: "admin".to_string(),
            admin_password: Password::new("secret123"),
            tls: None,
        };

        let json = serde_json::to_string(&config).expect("Failed to serialize");

        assert!(json.contains("\"admin_user\":\"admin\""));
        assert!(json.contains("\"admin_password\":\"secret123\""));
    }

    #[test]
    fn it_should_deserialize_grafana_config_from_json() {
        let json = r#"{"admin_user":"admin","admin_password":"secret123"}"#;

        let config: GrafanaConfig = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(config.admin_user, "admin");
        assert_eq!(config.admin_password.expose_secret(), "secret123");
    }

    #[test]
    fn it_should_redact_password_in_debug_output() {
        let config = GrafanaConfig {
            admin_user: "admin".to_string(),
            admin_password: Password::new("super_secret"),
            tls: None,
        };

        let debug_output = format!("{config:?}");

        assert!(debug_output.contains("admin_user: \"admin\""));
        assert!(debug_output.contains("Password(SecretBox<str>([REDACTED]"));
        assert!(!debug_output.contains("super_secret"));
    }

    #[test]
    fn it_should_clone_grafana_config() {
        let config = GrafanaConfig {
            admin_user: "admin".to_string(),
            admin_password: Password::new("password"),
            tls: None,
        };

        let cloned = config.clone();

        assert_eq!(cloned.admin_user, config.admin_user);
        assert_eq!(
            cloned.admin_password.expose_secret(),
            config.admin_password.expose_secret()
        );
    }
}
