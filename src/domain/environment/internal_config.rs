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
