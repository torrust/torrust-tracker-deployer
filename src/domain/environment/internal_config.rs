//! Internal Config Module
//!
//! This module contains the `InternalConfig` struct which holds internal
//! configuration derived from user inputs.
//!
//! ## Purpose
//!
//! Internal configuration represents automatically derived paths and settings
//! that are calculated from user inputs. These are implementation details not
//! directly controlled by users.
//!
//! ## Semantic Category
//!
//! **Internal Config** fields are:
//! - Calculated from user inputs
//! - Not directly controlled by users
//! - Examples: build directory, data directory
//!
//! Add new fields here when: Need internal paths or derived configuration.

use crate::domain::environment::EnvironmentName;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Base directory name for user data
const DATA_DIR_NAME: &str = "data";

/// Base directory name for build artifacts
const BUILD_DIR_NAME: &str = "build";

/// Internal paths and configuration derived from user inputs
///
/// This struct contains fields that are derived automatically from user inputs
/// and are not directly controlled by users. These represent internal
/// implementation details for organizing build artifacts and data.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::environment::internal_config::InternalConfig;
/// use std::path::PathBuf;
///
/// let internal_config = InternalConfig {
///     build_dir: PathBuf::from("build/production"),
///     data_dir: PathBuf::from("data/production"),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalConfig {
    /// Build directory for this environment (derived from environment name)
    pub build_dir: PathBuf,

    /// Data directory for this environment (derived from environment name)
    pub data_dir: PathBuf,
}

impl InternalConfig {
    /// Creates a new `InternalConfig` with auto-generated directories
    ///
    /// # Arguments
    ///
    /// * `env_name` - The environment name used to generate directories
    ///
    /// # Returns
    ///
    /// A new `InternalConfig` with:
    /// - `data_dir`: `data/{env_name}`
    /// - `build_dir`: `build/{env_name}`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::environment::internal_config::InternalConfig;
    /// use torrust_tracker_deployer_lib::domain::environment::EnvironmentName;
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("production".to_string())?;
    /// let config = InternalConfig::new(&env_name);
    ///
    /// assert_eq!(config.data_dir, PathBuf::from("data/production"));
    /// assert_eq!(config.build_dir, PathBuf::from("build/production"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn new(env_name: &EnvironmentName) -> Self {
        // Generate environment-specific directories (relative paths)
        let data_dir = PathBuf::from(DATA_DIR_NAME).join(env_name.as_str());
        let build_dir = PathBuf::from(BUILD_DIR_NAME).join(env_name.as_str());

        Self {
            build_dir,
            data_dir,
        }
    }

    /// Creates a new `InternalConfig` with directories relative to a working directory
    ///
    /// This version creates absolute paths by prepending the working directory
    /// to the generated data and build directories.
    ///
    /// # Arguments
    ///
    /// * `env_name` - The environment name used to generate directories
    /// * `working_dir` - The base working directory for operations
    ///
    /// # Returns
    ///
    /// A new `InternalConfig` with:
    /// - `data_dir`: `{working_dir}/data/{env_name}`
    /// - `build_dir`: `{working_dir}/build/{env_name}`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::environment::internal_config::InternalConfig;
    /// use torrust_tracker_deployer_lib::domain::environment::EnvironmentName;
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("production".to_string())?;
    /// let working_dir = PathBuf::from("/opt/deployments");
    /// let config = InternalConfig::with_working_dir(&env_name, &working_dir);
    ///
    /// assert_eq!(config.data_dir, PathBuf::from("/opt/deployments/data/production"));
    /// assert_eq!(config.build_dir, PathBuf::from("/opt/deployments/build/production"));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_working_dir(env_name: &EnvironmentName, working_dir: &std::path::Path) -> Self {
        // Generate environment-specific directories relative to working directory
        let data_dir = working_dir.join(DATA_DIR_NAME).join(env_name.as_str());
        let build_dir = working_dir.join(BUILD_DIR_NAME).join(env_name.as_str());

        Self {
            build_dir,
            data_dir,
        }
    }

    /// Returns the templates directory for this environment
    ///
    /// Path: `data/{env_name}/templates/`
    #[must_use]
    pub fn templates_dir(&self) -> PathBuf {
        self.data_dir.join(super::TEMPLATES_DIR_NAME)
    }

    /// Returns the traces directory for this environment
    ///
    /// Path: `data/{env_name}/traces/`
    #[must_use]
    pub fn traces_dir(&self) -> PathBuf {
        self.data_dir.join(super::TRACES_DIR_NAME)
    }

    /// Returns the ansible build directory
    ///
    /// Path: `build/{env_name}/ansible`
    #[must_use]
    pub fn ansible_build_dir(&self) -> PathBuf {
        self.build_dir.join(super::ANSIBLE_DIR_NAME)
    }

    /// Returns the `OpenTofu` build directory for the LXD provider
    ///
    /// Path: `build/{env_name}/tofu/lxd`
    #[must_use]
    pub fn tofu_build_dir(&self) -> PathBuf {
        self.build_dir
            .join(super::TOFU_DIR_NAME)
            .join(super::LXD_PROVIDER_NAME)
    }

    /// Returns the ansible templates directory
    ///
    /// Path: `data/{env_name}/templates/ansible`
    #[must_use]
    pub fn ansible_templates_dir(&self) -> PathBuf {
        self.templates_dir().join(super::ANSIBLE_DIR_NAME)
    }

    /// Returns the tofu templates directory
    ///
    /// Path: `data/{env_name}/templates/tofu`
    #[must_use]
    pub fn tofu_templates_dir(&self) -> PathBuf {
        self.templates_dir().join(super::TOFU_DIR_NAME)
    }
}
