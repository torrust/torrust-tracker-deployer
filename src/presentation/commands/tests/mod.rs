//! Shared Test Utilities for Command Handlers
//!
//! This module provides common test utilities, fixtures, and helpers for testing
//! command handlers in the presentation layer. It reduces duplication and ensures
//! consistent test patterns across all commands.
//!
//! # Purpose
//!
//! - Provides shared test context with temporary directory management
//! - Offers reusable test configuration generators
//! - Ensures consistent test setup across command tests
//! - Reduces code duplication in test modules
//!
//! # Usage
//!
//! ```rust
//! use crate::presentation::commands::tests::{TestContext, create_valid_config};
//!
//! #[test]
//! fn it_should_create_environment_from_valid_config() {
//!     let context = TestContext::new();
//!     let config_path = create_valid_config(context.working_dir(), "test-env");
//!     
//!     let result = handle_environment_creation(&config_path, context.working_dir());
//!     assert!(result.is_ok());
//! }
//! ```

use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// ============================================================================
// PUBLIC API - Test Context
// ============================================================================

/// Test context with temporary directory and common test dependencies
///
/// Provides a clean, isolated testing environment with automatic cleanup.
/// The temporary directory is automatically deleted when the context is dropped.
///
/// # Example
///
/// ```rust
/// use crate::presentation::commands::tests::TestContext;
///
/// #[test]
/// fn it_should_work_with_temp_directory() {
///     let context = TestContext::new();
///     
///     // Use the working directory for tests
///     let config_path = context.working_dir().join("config.json");
///     std::fs::write(&config_path, "{}").unwrap();
///     
///     assert!(config_path.exists());
///     // Cleanup happens automatically when context is dropped
/// }
/// ```
pub struct TestContext {
    _temp_dir: TempDir,
    working_dir: PathBuf,
}

impl TestContext {
    /// Create a new test context with a temporary working directory
    ///
    /// # Panics
    ///
    /// Panics if the temporary directory cannot be created
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temporary directory");
        let working_dir = temp_dir.path().to_path_buf();

        Self {
            _temp_dir: temp_dir,
            working_dir,
        }
    }

    /// Get the working directory path for this test context
    ///
    /// The working directory is guaranteed to exist and will be automatically
    /// cleaned up when the context is dropped.
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PUBLIC API - Configuration Generators
// ============================================================================

/// Create a valid environment configuration JSON file
///
/// This function creates a properly formatted configuration file with valid
/// SSH credentials pointing to test fixtures. The generated configuration
/// can be used for testing successful environment creation flows.
///
/// # Arguments
///
/// * `path` - Directory where the config file should be created
/// * `env_name` - Name of the environment (must be valid)
///
/// # Returns
///
/// Returns the full path to the created configuration file
///
/// # Example
///
/// ```rust
/// use tempfile::TempDir;
/// use crate::presentation::commands::tests::create_valid_config;
///
/// #[test]
/// fn it_should_load_valid_config() {
///     let temp_dir = TempDir::new().unwrap();
///     let config_path = create_valid_config(temp_dir.path(), "my-env");
///     
///     assert!(config_path.exists());
///     // Config can now be used with config loaders
/// }
/// ```
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
    fs::write(&config_path, config_json).expect("Failed to write config file");
    config_path
}

/// Create an invalid JSON configuration file for error testing
///
/// This function creates a malformed JSON file to test error handling
/// for JSON parsing failures. The file is syntactically invalid.
///
/// # Arguments
///
/// * `path` - Directory where the config file should be created
///
/// # Returns
///
/// Returns the full path to the created configuration file
///
/// # Example
///
/// ```rust
/// use tempfile::TempDir;
/// use crate::presentation::commands::tests::create_invalid_json_config;
///
/// #[test]
/// fn it_should_reject_invalid_json() {
///     let temp_dir = TempDir::new().unwrap();
///     let config_path = create_invalid_json_config(temp_dir.path());
///     
///     let result = load_config(&config_path);
///     assert!(result.is_err());
/// }
/// ```
pub fn create_invalid_json_config(path: &Path) -> PathBuf {
    let invalid_json = r#"{"environment": {"name": "test"#; // Missing closing braces
    let config_path = path.join("invalid.json");
    fs::write(&config_path, invalid_json).expect("Failed to write invalid config file");
    config_path
}

/// Create a configuration with an invalid environment name
///
/// This function creates a valid JSON file but with an environment name
/// that violates naming rules (contains underscores). Useful for testing
/// validation logic.
///
/// # Arguments
///
/// * `path` - Directory where the config file should be created
///
/// # Returns
///
/// Returns the full path to the created configuration file
///
/// # Example
///
/// ```rust
/// use tempfile::TempDir;
/// use crate::presentation::commands::tests::create_config_with_invalid_name;
///
/// #[test]
/// fn it_should_reject_invalid_environment_name() {
///     let temp_dir = TempDir::new().unwrap();
///     let config_path = create_config_with_invalid_name(temp_dir.path());
///     
///     let result = load_config(&config_path);
///     assert!(matches!(result, Err(ConfigError::ValidationFailed(_))));
/// }
/// ```
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
    fs::write(&config_path, config_json).expect("Failed to write config file with invalid name");
    config_path
}

/// Create a configuration with missing SSH key files
///
/// This function creates a valid JSON configuration that references
/// non-existent SSH key files. Useful for testing file validation logic.
///
/// # Arguments
///
/// * `path` - Directory where the config file should be created
///
/// # Returns
///
/// Returns the full path to the created configuration file
///
/// # Example
///
/// ```rust
/// use tempfile::TempDir;
/// use crate::presentation::commands::tests::create_config_with_missing_keys;
///
/// #[test]
/// fn it_should_reject_missing_ssh_keys() {
///     let temp_dir = TempDir::new().unwrap();
///     let config_path = create_config_with_missing_keys(temp_dir.path());
///     
///     let result = load_config(&config_path);
///     assert!(matches!(result, Err(ConfigError::ValidationFailed(_))));
/// }
/// ```
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
    fs::write(&config_path, config_json).expect("Failed to write config file with missing keys");
    config_path
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_test_context_with_working_directory() {
        let context = TestContext::new();

        assert!(context.working_dir().exists());
        assert!(context.working_dir().is_dir());
    }

    #[test]
    fn it_should_clean_up_temp_directory_after_test() {
        let temp_path = {
            let context = TestContext::new();
            let path = context.working_dir().to_path_buf();
            assert!(path.exists());
            path
        }; // context dropped here

        // Verify cleanup happened
        assert!(
            !temp_path.exists(),
            "Temporary directory should be cleaned up after context is dropped"
        );
    }

    #[test]
    fn it_should_create_valid_config_file() {
        let context = TestContext::new();
        let config_path = create_valid_config(context.working_dir(), "test-env");

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("test-env"));
        assert!(content.contains("fixtures/testing_rsa"));
    }

    #[test]
    fn it_should_create_invalid_json_config_file() {
        let context = TestContext::new();
        let config_path = create_invalid_json_config(context.working_dir());

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        // Verify it's actually invalid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&content).is_err());
    }

    #[test]
    fn it_should_create_config_with_invalid_environment_name() {
        let context = TestContext::new();
        let config_path = create_config_with_invalid_name(context.working_dir());

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("Invalid_Name_With_Underscore"));
    }

    #[test]
    fn it_should_create_config_with_missing_keys() {
        let context = TestContext::new();
        let config_path = create_config_with_missing_keys(context.working_dir());

        assert!(config_path.exists());
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("/nonexistent/private_key"));
    }
}
