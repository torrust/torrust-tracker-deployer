//! Environment State Assertions
//!
//! Provides assertion utilities for verifying environment state after
//! command execution in black-box tests.

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Assertions for verifying environment state after command execution
///
/// This struct provides methods to verify that the environment was created
/// correctly and that all expected files and directories exist with the
/// correct structure and content.
pub struct EnvironmentStateAssertions {
    workspace_path: PathBuf,
}

impl EnvironmentStateAssertions {
    /// Create a new assertions helper for the given workspace
    #[must_use]
    pub fn new<P: AsRef<Path>>(workspace_path: P) -> Self {
        Self {
            workspace_path: workspace_path.as_ref().to_path_buf(),
        }
    }

    /// Assert that the environment exists
    ///
    /// Verifies that the environment state file exists at the expected location.
    ///
    /// # Panics
    ///
    /// Panics if the environment file does not exist.
    pub fn assert_environment_exists(&self, env_name: &str) {
        let env_file_path = self.environment_json_path(env_name);
        assert!(
            env_file_path.exists(),
            "Environment file should exist at: {}",
            env_file_path.display()
        );
    }

    /// Assert that the environment is in the expected state
    ///
    /// Verifies that the environment state matches the expected state string.
    /// The state is determined by the top-level key in the environment JSON.
    ///
    /// # Panics
    ///
    /// Panics if the environment JSON cannot be read or if the state doesn't match.
    pub fn assert_environment_state_is(&self, env_name: &str, expected_state: &str) {
        let env_data = self
            .read_environment_json(env_name)
            .expect("Failed to read environment JSON");

        // Parse the environment state structure
        let state_key = env_data
            .as_object()
            .expect("Environment JSON should be an object")
            .keys()
            .next()
            .expect("Environment should have a state key");

        assert_eq!(
            state_key, expected_state,
            "Environment state should be '{expected_state}', but was '{state_key}'"
        );
    }

    /// Assert that the data directory structure exists
    ///
    /// Verifies that the environment's data directory and required files exist.
    ///
    /// # Panics
    ///
    /// Panics if the data directory or environment JSON file doesn't exist.
    #[allow(dead_code)]
    pub fn assert_data_directory_structure(&self, env_name: &str) {
        let data_dir = self.workspace_path.join("data").join(env_name);
        assert!(
            data_dir.exists(),
            "Data directory should exist at: {}",
            data_dir.display()
        );

        let env_json = data_dir.join("environment.json");
        assert!(
            env_json.exists(),
            "Environment JSON should exist at: {}",
            env_json.display()
        );
    }

    /// Assert that the trace directory exists
    ///
    /// Verifies that the environment's traces directory exists for observability.
    ///
    /// # Panics
    ///
    /// Panics if the traces directory doesn't exist.
    #[allow(dead_code)]
    pub fn assert_trace_directory_exists(&self, env_name: &str) {
        let traces_dir = self.workspace_path.join(env_name).join("traces");

        assert!(
            traces_dir.exists(),
            "Traces directory should exist at: {}",
            traces_dir.display()
        );
    }

    fn environment_json_path(&self, env_name: &str) -> PathBuf {
        self.workspace_path
            .join("data")
            .join(env_name)
            .join("environment.json")
    }

    fn read_environment_json(&self, env_name: &str) -> Result<Value> {
        let env_file_path = self.environment_json_path(env_name);
        let content = fs::read_to_string(&env_file_path).context(format!(
            "Failed to read environment file: {}",
            env_file_path.display()
        ))?;

        let json: Value =
            serde_json::from_str(&content).context("Failed to parse environment JSON")?;

        Ok(json)
    }
}
