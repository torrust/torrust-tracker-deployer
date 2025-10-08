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

/// Internal paths and configuration derived from user inputs
///
/// This struct contains fields that are derived automatically from user inputs
/// and are not directly controlled by users. These represent internal
/// implementation details for organizing build artifacts and data.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deploy::domain::environment::internal_config::InternalConfig;
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
    /// use torrust_tracker_deploy::domain::environment::internal_config::InternalConfig;
    /// use torrust_tracker_deploy::domain::environment::EnvironmentName;
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
        // Generate environment-specific directories
        let data_dir = PathBuf::from("data").join(env_name.as_str());
        let build_dir = PathBuf::from("build").join(env_name.as_str());

        Self {
            build_dir,
            data_dir,
        }
    }
}
