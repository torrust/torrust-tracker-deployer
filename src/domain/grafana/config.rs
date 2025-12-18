//! Grafana configuration domain type

use serde::{Deserialize, Serialize};

use crate::shared::secrets::Password;

/// Grafana metrics visualization configuration
///
/// Configures Grafana service for displaying tracker metrics.
/// Grafana requires Prometheus to be enabled for metrics visualization.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GrafanaConfig {
    /// Grafana admin username
    pub admin_user: String,

    /// Grafana admin password (should be changed in production)
    ///
    /// Uses `Password` wrapper from secrecy crate for secure handling:
    /// - Automatic redaction in debug output (shows `[REDACTED]`)
    /// - Memory zeroing when the value is dropped
    /// - Explicit `.expose_secret()` calls required to access plaintext
    pub admin_password: Password,
}

impl Default for GrafanaConfig {
    fn default() -> Self {
        Self {
            admin_user: "admin".to_string(),
            admin_password: Password::new("admin"),
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
        };

        assert_eq!(config.admin_user, "custom_admin");
        assert_eq!(config.admin_password.expose_secret(), "custom_pass");
    }

    #[test]
    fn it_should_serialize_grafana_config_to_json() {
        let config = GrafanaConfig {
            admin_user: "admin".to_string(),
            admin_password: Password::new("secret123"),
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
        };

        let cloned = config.clone();

        assert_eq!(cloned.admin_user, config.admin_user);
        assert_eq!(
            cloned.admin_password.expose_secret(),
            config.admin_password.expose_secret()
        );
    }
}
