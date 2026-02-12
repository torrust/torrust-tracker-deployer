//! Validate Command Handler
//!
//! This module contains the application layer handler for the validate command.
//! It validates environment configuration files without creating deployments.

use std::convert::TryInto;
use std::fs;
use std::path::Path;

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::domain::environment::EnvironmentParams;

use super::errors::ValidateCommandHandlerError;

/// Application layer handler for validate command
///
/// This handler validates environment configuration files by:
/// 1. Parsing the JSON structure
/// 2. Validating field types and values
/// 3. Verifying referenced files exist (SSH keys)
/// 4. Checking domain constraints
pub struct ValidateCommandHandler;

impl ValidateCommandHandler {
    /// Create a new validate command handler
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Validate an environment configuration file
    ///
    /// This method performs comprehensive validation:
    /// - JSON syntax and structure
    /// - Field types and constraints
    /// - SSH key file existence
    /// - Domain business rules
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the configuration file to validate
    ///
    /// # Returns
    ///
    /// * `Ok(ValidationResult)` - Configuration is valid with details
    /// * `Err(ValidateCommandHandlerError)` - Validation failed with specific reason
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File cannot be read
    /// - JSON syntax is invalid
    /// - Required fields are missing
    /// - Field values violate constraints
    /// - Referenced SSH keys don't exist
    /// - Domain rules are violated
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::application::command_handlers::validate::ValidateCommandHandler;
    ///
    /// let handler = ValidateCommandHandler::new();
    /// let result = handler.validate(Path::new("envs/my-env.json"))?;
    ///
    /// println!("Configuration is valid!");
    /// println!("Environment name: {}", result.environment_name);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn validate(
        &self,
        config_path: &Path,
    ) -> Result<ValidationResult, ValidateCommandHandlerError> {
        // Step 1: Read file contents
        let content = fs::read_to_string(config_path).map_err(|source| {
            ValidateCommandHandlerError::FileReadFailed {
                path: config_path.to_path_buf(),
                source,
            }
        })?;

        // Step 2: Parse JSON to EnvironmentCreationConfig
        // This validates JSON syntax and maps to our structure
        let config: EnvironmentCreationConfig =
            serde_json::from_str(&content).map_err(|source| {
                ValidateCommandHandlerError::JsonParsingFailed {
                    path: config_path.to_path_buf(),
                    source,
                }
            })?;

        // Step 3: Convert to domain types (validates all constraints)
        // This includes:
        // - SSH key paths must be absolute (file existence checked at runtime)
        // - Port numbers must be valid
        // - Domain names must be well-formed
        // - All business rules must pass
        let _validated_params: EnvironmentParams = config
            .clone()
            .try_into()
            .map_err(ValidateCommandHandlerError::DomainValidationFailed)?;

        // All validation passed!
        Ok(ValidationResult {
            environment_name: config.environment.name.clone(),
            provider: config.provider.provider().to_string(),
            has_prometheus: config.prometheus.is_some(),
            has_grafana: config.grafana.is_some(),
            has_https: config.https.is_some(),
            has_backup: config.backup.is_some(),
        })
    }
}

impl Default for ValidateCommandHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of successful validation
///
/// Contains key information about the validated configuration
/// for user feedback.
#[allow(clippy::struct_excessive_bools)] // Intentional: presentation data with feature flags
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Name of the environment
    pub environment_name: String,

    /// Provider type (lxd or hetzner)
    pub provider: String,

    /// Whether Prometheus is configured
    pub has_prometheus: bool,

    /// Whether Grafana is configured
    pub has_grafana: bool,

    /// Whether HTTPS is configured
    pub has_https: bool,

    /// Whether backups are configured
    pub has_backup: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn it_should_validate_valid_configuration_when_all_fields_are_correct() {
        let handler = ValidateCommandHandler::new();

        // Create temp directory for test config
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let config_path = temp_dir.path().join("test-config.json");

        // Get absolute paths to test fixtures
        let project_root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        // Create test config with absolute paths
        let config_json = format!(
            r#"{{
    "environment": {{
        "name": "test-validation"
    }},
    "ssh_credentials": {{
        "private_key_path": "{private_key_path}",
        "public_key_path": "{public_key_path}",
        "username": "torrust",
        "port": 22
    }},
    "provider": {{
        "provider": "lxd",
        "profile_name": "test-profile"
    }},
    "tracker": {{
        "core": {{
            "database": {{
                "driver": "sqlite3",
                "database_name": "tracker.db"
            }},
            "private": false
        }},
        "udp_trackers": [
            {{
                "bind_address": "0.0.0.0:6969",
                "domain": "udp.tracker.local"
            }}
        ],
        "http_trackers": [
            {{
                "bind_address": "0.0.0.0:7070",
                "domain": "http.tracker.local"
            }}
        ],
        "http_api": {{
            "bind_address": "0.0.0.0:1212",
            "admin_token": "MyAccessToken",
            "domain": "api.tracker.local"
        }},
        "health_check_api": {{
            "bind_address": "0.0.0.0:1313",
            "domain": "health.tracker.local"
        }}
    }},
    "grafana": {{
        "admin_user": "admin",
        "admin_password": "admin-password",
        "domain": "grafana.tracker.local"
    }},
    "prometheus": {{
        "scrape_interval_in_secs": 15
    }}
}}"#
        );

        fs::write(&config_path, config_json).expect("Failed to write test config");

        // Run validation
        let result = handler.validate(&config_path);

        assert!(result.is_ok(), "Valid configuration should pass validation");
    }

    #[test]
    fn it_should_return_error_when_file_does_not_exist() {
        let handler = ValidateCommandHandler::new();

        let result = handler.validate(Path::new("/tmp/nonexistent.json"));

        assert!(
            matches!(
                result,
                Err(ValidateCommandHandlerError::FileReadFailed { .. })
            ),
            "Non-existent file should return FileReadFailed error"
        );
    }

    #[test]
    fn it_should_return_error_when_json_is_malformed() {
        let handler = ValidateCommandHandler::new();

        // Create a temporary file with invalid JSON
        let temp_file = std::env::temp_dir().join("invalid.json");
        fs::write(&temp_file, "{ invalid json }").unwrap();

        let result = handler.validate(&temp_file);

        // Cleanup
        drop(fs::remove_file(temp_file));

        assert!(
            matches!(
                result,
                Err(ValidateCommandHandlerError::JsonParsingFailed { .. })
            ),
            "Malformed JSON should return JsonParsingFailed error"
        );
    }
}
