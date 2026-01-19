//! Grafana configuration domain type

use serde::{Deserialize, Serialize};

use crate::shared::domain_name::DomainName;
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

    /// Domain name for the service (optional)
    ///
    /// When present, defines the hostname for accessing Grafana.
    /// Can be used with or without TLS proxy.
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<DomainName>,

    /// Whether TLS termination via Caddy is enabled
    ///
    /// When true, Grafana will be accessible via HTTPS through
    /// the Caddy reverse proxy using the configured domain.
    use_tls_proxy: bool,
}

impl GrafanaConfig {
    /// Creates a new Grafana configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::grafana::GrafanaConfig;
    ///
    /// let config = GrafanaConfig::new(
    ///     "admin".to_string(),
    ///     "password".to_string(),
    ///     None,
    ///     false,
    /// );
    /// assert_eq!(config.admin_user(), "admin");
    /// ```
    #[must_use]
    pub fn new(
        admin_user: String,
        admin_password: String,
        domain: Option<DomainName>,
        use_tls_proxy: bool,
    ) -> Self {
        Self {
            admin_user,
            admin_password: Password::new(admin_password),
            domain,
            use_tls_proxy,
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

    /// Returns the domain if configured
    #[must_use]
    pub fn domain(&self) -> Option<&DomainName> {
        self.domain.as_ref()
    }

    /// Returns the TLS domain if TLS proxy is enabled
    ///
    /// Returns the domain only when both domain is set AND `use_tls_proxy` is true.
    /// This is used to determine if the service should be accessed via HTTPS.
    #[must_use]
    pub fn tls_domain(&self) -> Option<&str> {
        if self.use_tls_proxy {
            self.domain.as_ref().map(DomainName::as_str)
        } else {
            None
        }
    }

    /// Returns whether TLS proxy is enabled
    #[must_use]
    pub fn use_tls_proxy(&self) -> bool {
        self.use_tls_proxy
    }
}

impl Default for GrafanaConfig {
    fn default() -> Self {
        Self {
            admin_user: "admin".to_string(),
            admin_password: Password::new("admin"),
            domain: None,
            use_tls_proxy: false,
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
        assert!(config.domain.is_none());
        assert!(!config.use_tls_proxy);
    }

    #[test]
    fn it_should_create_grafana_config_with_custom_values() {
        let config = GrafanaConfig {
            admin_user: "custom_admin".to_string(),
            admin_password: Password::new("custom_pass"),
            domain: None,
            use_tls_proxy: false,
        };

        assert_eq!(config.admin_user, "custom_admin");
        assert_eq!(config.admin_password.expose_secret(), "custom_pass");
    }

    #[test]
    fn it_should_create_grafana_config_with_domain_and_tls_proxy() {
        let domain = DomainName::new("grafana.example.com").unwrap();
        let config = GrafanaConfig::new(
            "admin".to_string(),
            "password".to_string(),
            Some(domain.clone()),
            true,
        );

        assert_eq!(config.domain(), Some(&domain));
        assert!(config.use_tls_proxy());
        assert_eq!(config.tls_domain(), Some("grafana.example.com"));
    }

    #[test]
    fn it_should_return_none_for_tls_domain_when_tls_proxy_disabled() {
        let domain = DomainName::new("grafana.example.com").unwrap();
        let config = GrafanaConfig::new(
            "admin".to_string(),
            "password".to_string(),
            Some(domain.clone()),
            false,
        );

        assert_eq!(config.domain(), Some(&domain));
        assert!(!config.use_tls_proxy());
        assert!(config.tls_domain().is_none());
    }

    #[test]
    fn it_should_serialize_grafana_config_to_json() {
        let config = GrafanaConfig {
            admin_user: "admin".to_string(),
            admin_password: Password::new("secret123"),
            domain: None,
            use_tls_proxy: false,
        };

        let json = serde_json::to_string(&config).expect("Failed to serialize");

        assert!(json.contains("\"admin_user\":\"admin\""));
        assert!(json.contains("\"admin_password\":\"secret123\""));
        assert!(json.contains("\"use_tls_proxy\":false"));
    }

    #[test]
    fn it_should_deserialize_grafana_config_from_json() {
        let json = r#"{"admin_user":"admin","admin_password":"secret123","use_tls_proxy":false}"#;

        let config: GrafanaConfig = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(config.admin_user, "admin");
        assert_eq!(config.admin_password.expose_secret(), "secret123");
        assert!(!config.use_tls_proxy);
    }

    #[test]
    fn it_should_redact_password_in_debug_output() {
        let config = GrafanaConfig {
            admin_user: "admin".to_string(),
            admin_password: Password::new("super_secret"),
            domain: None,
            use_tls_proxy: false,
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
            domain: None,
            use_tls_proxy: false,
        };

        let cloned = config.clone();

        assert_eq!(cloned.admin_user, config.admin_user);
        assert_eq!(
            cloned.admin_password.expose_secret(),
            config.admin_password.expose_secret()
        );
    }
}
