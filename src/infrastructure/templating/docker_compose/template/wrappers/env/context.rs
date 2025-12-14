//! Context for the env.tera template
//!
//! This module defines the structure and validation for environment variables
//! that will be rendered into the .env file for Docker Compose.

use serde::Serialize;

/// Context for rendering the .env template
///
/// Contains all variables needed for the Docker Compose environment configuration.
#[derive(Serialize, Debug, Clone)]
pub struct EnvContext {
    /// The admin token for the Torrust Tracker HTTP API
    tracker_api_admin_token: String,
    /// `MySQL` root password (only used when `MySQL` driver is configured)
    #[serde(skip_serializing_if = "Option::is_none")]
    mysql_root_password: Option<String>,
    /// `MySQL` database name (only used when `MySQL` driver is configured)
    #[serde(skip_serializing_if = "Option::is_none")]
    mysql_database: Option<String>,
    /// `MySQL` user (only used when `MySQL` driver is configured)
    #[serde(skip_serializing_if = "Option::is_none")]
    mysql_user: Option<String>,
    /// `MySQL` password (only used when `MySQL` driver is configured)
    #[serde(skip_serializing_if = "Option::is_none")]
    mysql_password: Option<String>,
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
    /// assert_eq!(context.tracker_api_admin_token(), "MySecretToken123");
    /// ```
    #[must_use]
    pub fn new(tracker_api_admin_token: String) -> Self {
        Self {
            tracker_api_admin_token,
            mysql_root_password: None,
            mysql_database: None,
            mysql_user: None,
            mysql_password: None,
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
    #[must_use]
    pub fn new_with_mysql(
        tracker_api_admin_token: String,
        mysql_root_password: String,
        mysql_database: String,
        mysql_user: String,
        mysql_password: String,
    ) -> Self {
        Self {
            tracker_api_admin_token,
            mysql_root_password: Some(mysql_root_password),
            mysql_database: Some(mysql_database),
            mysql_user: Some(mysql_user),
            mysql_password: Some(mysql_password),
        }
    }

    /// Get the tracker API admin token
    #[must_use]
    pub fn tracker_api_admin_token(&self) -> &str {
        &self.tracker_api_admin_token
    }

    /// Get the `MySQL` root password (if configured)
    #[must_use]
    pub fn mysql_root_password(&self) -> Option<&str> {
        self.mysql_root_password.as_deref()
    }

    /// Get the `MySQL` database name (if configured)
    #[must_use]
    pub fn mysql_database(&self) -> Option<&str> {
        self.mysql_database.as_deref()
    }

    /// Get the `MySQL` user (if configured)
    #[must_use]
    pub fn mysql_user(&self) -> Option<&str> {
        self.mysql_user.as_deref()
    }

    /// Get the `MySQL` password (if configured)
    #[must_use]
    pub fn mysql_password(&self) -> Option<&str> {
        self.mysql_password.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_context_with_tracker_token() {
        let token = "TestToken123".to_string();
        let context = EnvContext::new(token.clone());

        assert_eq!(context.tracker_api_admin_token(), "TestToken123");
    }

    #[test]
    fn it_should_be_serializable() {
        let context = EnvContext::new("AdminToken456".to_string());

        // Verify it can be serialized (needed for Tera template rendering)
        let serialized = serde_json::to_string(&context).unwrap();
        assert!(serialized.contains("tracker_api_admin_token"));
        assert!(serialized.contains("AdminToken456"));
    }
}
