//! Test Fixtures for Create Command Tests
//!
//! This module provides test fixtures and helper functions for testing
//! the create command presentation layer.

use std::fs;
use std::path::{Path, PathBuf};

/// Create a valid environment configuration JSON file
///
/// # Arguments
///
/// * `path` - Path where the config file should be created
/// * `env_name` - Name of the environment
///
/// # Returns
///
/// Returns the path to the created configuration file
pub fn create_valid_config(path: &Path, env_name: &str) -> PathBuf {
    // Use absolute paths to SSH keys to ensure they work regardless of current directory
    let project_root = env!("CARGO_MANIFEST_DIR");
    let private_key_path = format!("{project_root}/fixtures/testing_rsa");
    let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

    let config_json = format!(
        r#"{{
    "environment": {{
        "name": "{env_name}"
    }},
    "ssh_credentials": {{
        "private_key_path": "{private_key_path}",
        "public_key_path": "{public_key_path}"
    }}
}}"#
    );

    let config_path = path.join("config.json");
    fs::write(&config_path, config_json).unwrap();
    config_path
}

/// Create an invalid JSON configuration file
///
/// # Arguments
///
/// * `path` - Path where the config file should be created
///
/// # Returns
///
/// Returns the path to the created configuration file
pub fn create_invalid_json_config(path: &Path) -> PathBuf {
    let invalid_json = r#"{"environment": {"name": "test"#; // Missing closing braces
    let config_path = path.join("invalid.json");
    fs::write(&config_path, invalid_json).unwrap();
    config_path
}

/// Create a configuration with an invalid environment name
///
/// # Arguments
///
/// * `path` - Path where the config file should be created
///
/// # Returns
///
/// Returns the path to the created configuration file
pub fn create_config_with_invalid_name(path: &Path) -> PathBuf {
    // Use absolute paths to SSH keys to ensure they work regardless of current directory
    let project_root = env!("CARGO_MANIFEST_DIR");
    let private_key_path = format!("{project_root}/fixtures/testing_rsa");
    let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

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

    let config_path = path.join("invalid_name.json");
    fs::write(&config_path, config_json).unwrap();
    config_path
}

/// Create a configuration with missing SSH key files
///
/// # Arguments
///
/// * `path` - Path where the config file should be created
///
/// # Returns
///
/// Returns the path to the created configuration file
pub fn create_config_with_missing_keys(path: &Path) -> PathBuf {
    let config_json = r#"{
    "environment": {
        "name": "test-env"
    },
    "ssh_credentials": {
        "private_key_path": "/nonexistent/private_key",
        "public_key_path": "/nonexistent/public_key.pub"
    }
}"#;

    let config_path = path.join("missing_keys.json");
    fs::write(&config_path, config_json).unwrap();
    config_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_valid_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_valid_config(temp_dir.path(), "test-env");

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("test-env"));
        assert!(content.contains("fixtures/testing_rsa"));
    }

    #[test]
    fn it_should_create_invalid_json_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_invalid_json_config(temp_dir.path());

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        // Verify it's actually invalid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&content).is_err());
    }

    #[test]
    fn it_should_create_config_with_invalid_environment_name() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_config_with_invalid_name(temp_dir.path());

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("Invalid_Name_With_Underscore"));
    }

    #[test]
    fn it_should_create_config_with_missing_keys() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_config_with_missing_keys(temp_dir.path());

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("/nonexistent/private_key"));
    }
}
