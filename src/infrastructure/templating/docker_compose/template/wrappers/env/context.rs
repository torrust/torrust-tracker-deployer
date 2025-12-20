//! Context for the env.tera template
//!
//! This module defines the structure and validation for environment variables
//! that will be rendered into the .env file for Docker Compose.
//!
//! The context is organized by service to mirror the structure of the .env template:
//! - Tracker service configuration
//! - `MySQL` service configuration (optional)

use serde::Serialize;

/// Configuration for the Tracker service
///
/// Contains environment variables for the Torrust Tracker container.
#[derive(Serialize, Debug, Clone)]
pub struct TrackerServiceConfig {
    /// The admin token for the Torrust Tracker HTTP API
    pub api_admin_token: String,
    /// Database driver type ("sqlite3" or "mysql")
    /// Controls which config template the container entrypoint uses
    pub database_driver: String,
}

/// Configuration for the `MySQL` service
///
/// Contains environment variables for the `MySQL` container.
/// Only included when `MySQL` driver is configured.
#[derive(Serialize, Debug, Clone)]
pub struct MySqlServiceConfig {
    /// `MySQL` root password
    pub root_password: String,
    /// `MySQL` database name
    pub database: String,
    /// `MySQL` user
    pub user: String,
    /// `MySQL` password
    pub password: String,
}

/// Configuration for the Grafana service
///
/// Contains environment variables for the Grafana container.
/// Only included when Grafana is enabled.
#[derive(Serialize, Debug, Clone)]
pub struct GrafanaServiceConfig {
    /// Grafana admin user
    pub admin_user: String,
    /// Grafana admin password (exposed from secrecy wrapper)
    pub admin_password: String,
}

/// Context for rendering the .env template
///
/// Contains all variables needed for the Docker Compose environment configuration,
/// organized by service to mirror the template structure.
#[derive(Serialize, Debug, Clone)]
pub struct EnvContext {
    /// Tracker service configuration
    pub tracker: TrackerServiceConfig,
    /// `MySQL` service configuration (only present when `MySQL` driver is configured)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mysql: Option<MySqlServiceConfig>,
    /// Grafana service configuration (only present when Grafana is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grafana: Option<GrafanaServiceConfig>,
}

impl EnvContext {
    /// Creates a new `EnvContext` with the tracker admin token (`SQLite` mode)
    ///
    /// # Arguments
    ///
    /// * `tracker_api_admin_token` - The admin token for tracker API authentication
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::env::EnvContext;
    ///
    /// let context = EnvContext::new("MySecretToken123".to_string());
    /// assert_eq!(context.tracker.api_admin_token, "MySecretToken123");
    /// assert_eq!(context.tracker.database_driver, "sqlite3");
    /// assert!(context.mysql.is_none());
    /// ```
    #[must_use]
    pub fn new(tracker_api_admin_token: String) -> Self {
        Self {
            tracker: TrackerServiceConfig {
                api_admin_token: tracker_api_admin_token,
                database_driver: "sqlite3".to_string(),
            },
            mysql: None,
            grafana: None,
        }
    }

    /// Creates a new `EnvContext` with `MySQL` credentials
    ///
    /// # Arguments
    ///
    /// * `tracker_api_admin_token` - The admin token for tracker API authentication
    /// * `mysql_root_password` - `MySQL` root password
    /// * `mysql_database` - `MySQL` database name
    /// * `mysql_user` - `MySQL` user
    /// * `mysql_password` - `MySQL` password
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::infrastructure::templating::docker_compose::template::wrappers::env::EnvContext;
    ///
    /// let context = EnvContext::new_with_mysql(
    ///     "MySecretToken123".to_string(),
    ///     "root_pass".to_string(),
    ///     "tracker_db".to_string(),
    ///     "tracker_user".to_string(),
    ///     "user_pass".to_string(),
    /// );
    /// assert_eq!(context.tracker.database_driver, "mysql");
    /// assert!(context.mysql.is_some());
    /// ```
    #[must_use]
    pub fn new_with_mysql(
        tracker_api_admin_token: String,
        mysql_root_password: String,
        mysql_database: String,
        mysql_user: String,
        mysql_password: String,
    ) -> Self {
        Self {
            tracker: TrackerServiceConfig {
                api_admin_token: tracker_api_admin_token,
                database_driver: "mysql".to_string(),
            },
            mysql: Some(MySqlServiceConfig {
                root_password: mysql_root_password,
                database: mysql_database,
                user: mysql_user,
                password: mysql_password,
            }),
            grafana: None,
        }
    }

    /// Get the tracker API admin token
    #[must_use]
    pub fn tracker_api_admin_token(&self) -> &str {
        &self.tracker.api_admin_token
    }

    /// Get the database driver type
    #[must_use]
    pub fn database_driver(&self) -> &str {
        &self.tracker.database_driver
    }

    /// Adds Grafana configuration
    ///
    /// Exposes the admin password from the secrecy wrapper for template rendering.
    ///
    /// # Arguments
    ///
    /// * `admin_user` - Grafana admin username
    /// * `admin_password` - Grafana admin password (plain String, already exposed)
    #[must_use]
    pub fn with_grafana(mut self, admin_user: String, admin_password: String) -> Self {
        self.grafana = Some(GrafanaServiceConfig {
            admin_user,
            admin_password,
        });
        self
    }

    /// Get the Grafana admin user (if configured)
    #[must_use]
    pub fn grafana_admin_user(&self) -> Option<&str> {
        self.grafana.as_ref().map(|g| g.admin_user.as_str())
    }

    /// Get the Grafana admin password (if configured)
    #[must_use]
    pub fn grafana_admin_password(&self) -> Option<&str> {
        self.grafana.as_ref().map(|g| g.admin_password.as_str())
    }

    /// Get the `MySQL` root password (if configured)
    #[must_use]
    pub fn mysql_root_password(&self) -> Option<&str> {
        self.mysql.as_ref().map(|m| m.root_password.as_str())
    }

    /// Get the `MySQL` database name (if configured)
    #[must_use]
    pub fn mysql_database(&self) -> Option<&str> {
        self.mysql.as_ref().map(|m| m.database.as_str())
    }

    /// Get the `MySQL` user (if configured)
    #[must_use]
    pub fn mysql_user(&self) -> Option<&str> {
        self.mysql.as_ref().map(|m| m.user.as_str())
    }

    /// Get the `MySQL` password (if configured)
    #[must_use]
    pub fn mysql_password(&self) -> Option<&str> {
        self.mysql.as_ref().map(|m| m.password.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_context_with_tracker_token() {
        let token = "TestToken123".to_string();
        let context = EnvContext::new(token.clone());

        assert_eq!(context.tracker.api_admin_token, "TestToken123");
        assert_eq!(context.tracker.database_driver, "sqlite3");
        assert!(context.mysql.is_none());
    }

    #[test]
    fn it_should_create_context_with_mysql_configuration() {
        let context = EnvContext::new_with_mysql(
            "AdminToken456".to_string(),
            "root_pass".to_string(),
            "tracker_db".to_string(),
            "tracker_user".to_string(),
            "user_pass".to_string(),
        );

        assert_eq!(context.tracker.api_admin_token, "AdminToken456");
        assert_eq!(context.tracker.database_driver, "mysql");
        assert!(context.mysql.is_some());

        let mysql_config = context.mysql.as_ref().unwrap();
        assert_eq!(mysql_config.root_password, "root_pass");
        assert_eq!(mysql_config.database, "tracker_db");
        assert_eq!(mysql_config.user, "tracker_user");
        assert_eq!(mysql_config.password, "user_pass");
    }

    #[test]
    fn it_should_be_serializable() {
        let context = EnvContext::new("AdminToken456".to_string());

        // Verify it can be serialized (needed for Tera template rendering)
        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("tracker"));
        assert!(serialized.contains("api_admin_token"));
        assert!(serialized.contains("AdminToken456"));
    }

    #[test]
    fn it_should_serialize_mysql_config_when_present() {
        let context = EnvContext::new_with_mysql(
            "Token123".to_string(),
            "root".to_string(),
            "db".to_string(),
            "user".to_string(),
            "pass".to_string(),
        );

        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("mysql"));
        assert!(serialized.contains("root_password"));
    }

    #[test]
    fn it_should_not_serialize_mysql_config_when_absent() {
        let context = EnvContext::new("Token123".to_string());

        let serialized = serde_json::to_string(&context).unwrap();
        // MySQL section should not be present when None
        assert!(!serialized.contains("mysql"));
    }

    #[test]
    fn it_should_provide_backward_compatible_getters() {
        let context = EnvContext::new_with_mysql(
            "Token123".to_string(),
            "root_pass".to_string(),
            "tracker_db".to_string(),
            "tracker_user".to_string(),
            "user_pass".to_string(),
        );

        // Backward compatible getter methods
        assert_eq!(context.tracker_api_admin_token(), "Token123");
        assert_eq!(context.database_driver(), "mysql");
        assert_eq!(context.mysql_root_password(), Some("root_pass"));
        assert_eq!(context.mysql_database(), Some("tracker_db"));
        assert_eq!(context.mysql_user(), Some("tracker_user"));
        assert_eq!(context.mysql_password(), Some("user_pass"));
    }
}
