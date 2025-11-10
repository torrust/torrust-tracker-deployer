//! Configuration Loader with Figment Integration
//!
//! This module provides configuration loading from JSON files using Figment.
//! Figment is used only in the presentation layer as a delivery mechanism,
//! following DDD architecture boundaries.

use std::path::Path;

use figment::{
    providers::{Format, Json},
    Figment,
};

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;

use super::errors::{ConfigFormat, CreateSubcommandError};

/// Configuration loader using Figment for JSON file parsing
///
/// This loader is part of the presentation layer and handles the specifics
/// of loading and parsing configuration files. It uses Figment for flexible
/// configuration file handling and validation.
///
/// # Architecture Note
///
/// Figment stays in the presentation layer as a delivery mechanism.
/// The domain layer remains independent of configuration parsing libraries.
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load environment creation configuration from a JSON file
    ///
    /// This method loads and parses a JSON configuration file, then validates
    /// it according to domain rules. All errors are wrapped in presentation
    /// layer error types with helpful guidance.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the JSON configuration file
    ///
    /// # Returns
    ///
    /// * `Ok(EnvironmentCreationConfig)` - Successfully loaded and validated configuration
    /// * `Err(CreateSubcommandError)` - File not found, parsing failed, or validation failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Configuration file doesn't exist
    /// - JSON parsing fails (syntax errors, type mismatches)
    /// - Domain validation fails (invalid names, missing SSH keys, etc.)
    ///
    /// All errors include detailed troubleshooting guidance via `.help()`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::presentation::controllers::create::ConfigLoader;
    ///
    /// let loader = ConfigLoader;
    /// let config = loader.load_from_file(Path::new("config/environment.json"))?;
    ///
    /// println!("Loaded environment: {}", config.environment.name);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn load_from_file(
        &self,
        config_path: &Path,
    ) -> Result<EnvironmentCreationConfig, CreateSubcommandError> {
        // Step 1: Verify file exists
        if !config_path.exists() {
            return Err(CreateSubcommandError::ConfigFileNotFound {
                path: config_path.to_path_buf(),
            });
        }

        // Step 2: Load with Figment
        // We don't use defaults here because we want explicit configuration
        let config: EnvironmentCreationConfig = Figment::new()
            .merge(Json::file(config_path))
            .extract()
            .map_err(|source| CreateSubcommandError::ConfigParsingFailed {
                path: config_path.to_path_buf(),
                format: ConfigFormat::Json,
                source: Box::new(source),
            })?;

        // Step 3: Validate using domain rules
        // This converts string-based config to domain types and validates
        config
            .clone()
            .to_environment_params()
            .map_err(|source| CreateSubcommandError::ConfigValidationFailed { source })?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn it_should_load_valid_json_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        // Use absolute paths to SSH keys to ensure they work regardless of current directory
        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        // Write a valid configuration file
        let config_json = format!(
            r#"{{
            "environment": {{
                "name": "test-env"
            }},
            "ssh_credentials": {{
                "private_key_path": "{private_key_path}",
                "public_key_path": "{public_key_path}"
            }}
        }}"#
        );
        fs::write(&config_path, config_json).unwrap();

        let loader = ConfigLoader;
        let result = loader.load_from_file(&config_path);

        assert!(result.is_ok(), "Should load valid configuration");
        let config = result.unwrap();
        assert_eq!(config.environment.name, "test-env");
    }

    #[test]
    fn it_should_return_error_for_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.json");

        let loader = ConfigLoader;
        let result = loader.load_from_file(&config_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            CreateSubcommandError::ConfigFileNotFound { path } => {
                assert_eq!(path, config_path);
            }
            _ => panic!("Expected ConfigFileNotFound error"),
        }
    }

    #[test]
    fn it_should_return_error_for_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.json");

        // Write invalid JSON (missing closing brace)
        fs::write(&config_path, r#"{"environment": {"name": "test""#).unwrap();

        let loader = ConfigLoader;
        let result = loader.load_from_file(&config_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            CreateSubcommandError::ConfigParsingFailed { path, format, .. } => {
                assert_eq!(path, config_path);
                assert!(matches!(format, ConfigFormat::Json));
            }
            _ => panic!("Expected ConfigParsingFailed error"),
        }
    }

    #[test]
    fn it_should_return_error_for_missing_required_fields() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("incomplete.json");

        // Write JSON with missing required fields
        fs::write(&config_path, r#"{"environment": {}}"#).unwrap();

        let loader = ConfigLoader;
        let result = loader.load_from_file(&config_path);

        assert!(result.is_err());
        // Should fail at parsing or validation stage
    }

    #[test]
    fn it_should_return_error_for_invalid_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid_name.json");

        // Use absolute paths to SSH keys to ensure they work regardless of current directory
        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        // Write config with invalid environment name
        let config_json = format!(
            r#"{{
            "environment": {{
                "name": "Invalid_Name_With_Underscore"
            }},
            "ssh_credentials": {{
                "private_key_path": "{private_key_path}",
                "public_key_path": "{public_key_path}"
            }}
        }}"#
        );
        fs::write(&config_path, config_json).unwrap();

        let loader = ConfigLoader;
        let result = loader.load_from_file(&config_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            CreateSubcommandError::ConfigValidationFailed { .. } => {
                // Expected - validation should catch invalid environment name
            }
            other => panic!("Expected ConfigValidationFailed, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_for_missing_ssh_keys() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("missing_keys.json");

        // Write config with non-existent SSH key files
        let config_json = r#"{
            "environment": {
                "name": "test-env"
            },
            "ssh_credentials": {
                "private_key_path": "/nonexistent/key",
                "public_key_path": "/nonexistent/key.pub"
            }
        }"#;
        fs::write(&config_path, config_json).unwrap();

        let loader = ConfigLoader;
        let result = loader.load_from_file(&config_path);

        assert!(result.is_err());
        match result.unwrap_err() {
            CreateSubcommandError::ConfigValidationFailed { .. } => {
                // Expected - validation should catch missing SSH keys
            }
            other => panic!("Expected ConfigValidationFailed, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_handle_config_with_optional_fields_having_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("with_defaults.json");

        // Use absolute paths to SSH keys to ensure they work regardless of current directory
        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        // Write config without optional fields (they should use defaults)
        let config_json = format!(
            r#"{{
            "environment": {{
                "name": "test-env"
            }},
            "ssh_credentials": {{
                "private_key_path": "{private_key_path}",
                "public_key_path": "{public_key_path}"
            }}
        }}"#
        );
        fs::write(&config_path, config_json).unwrap();

        let loader = ConfigLoader;
        let result = loader.load_from_file(&config_path);

        assert!(
            result.is_ok(),
            "Should load config with default values for optional fields"
        );
    }
}
