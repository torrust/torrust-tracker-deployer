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
}

impl EnvContext {
    /// Creates a new `EnvContext` with the tracker admin token
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
        }
    }

    /// Get the tracker API admin token
    #[must_use]
    pub fn tracker_api_admin_token(&self) -> &str {
        &self.tracker_api_admin_token
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
