//! Configuration management for deployment settings
//!
//! This module provides the `Config` struct which manages essential file system
//! paths for the deployment process including template sources and build outputs.
//!
//! ## Current Configuration Areas
//!
//! - File system paths for templates and build outputs
//! - Project root directory for resolving relative paths
//!
//! ## Path Relationships
//!
//! Typically, the paths have this relationship:
//! - `project_root`: `/path/to/torrust-tracker-deploy`
//! - `templates_dir`: `{project_root}/templates`
//! - `build_dir`: `{project_root}/build`
//!
//! The configuration is typically created once at deployment start and passed
//! throughout the system to ensure consistent path resolution across all components.

use std::path::PathBuf;

/// Configuration parameters for deployment environments.
///
/// Centralizes all deployment-related configuration including file paths,
/// service connection details, and runtime behavior settings.
///
/// Created once at deployment start and passed to [`Services::new()`](crate::testing::e2e::container::Services::new).
pub struct Config {
    /// Directory containing template files for rendering configurations.
    ///
    /// This directory should contain subdirectories for different template
    /// types (e.g., "ansible/", "tofu/") with template files that will be
    /// processed and rendered to the build directory.
    pub templates_dir: PathBuf,

    /// Root directory of the project.
    ///
    /// Used for resolving relative paths and locating project resources
    /// such as SSH key fixtures and project-specific configuration files.
    pub project_root: PathBuf,

    /// Directory where rendered configuration files will be written.
    ///
    /// All processed templates and generated configuration files are written
    /// to subdirectories within this build directory. This directory is
    /// typically git-ignored to avoid committing generated files.
    pub build_dir: PathBuf,
}

impl Config {
    /// Creates a new configuration with the provided parameters.
    ///
    /// ```rust
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deployer_lib::config::Config;
    /// let config = Config::new(
    ///     PathBuf::from("/home/user/project/templates"),
    ///     PathBuf::from("/home/user/project"),
    ///     PathBuf::from("/home/user/project/build"),
    /// );
    /// ```
    #[must_use]
    pub fn new(templates_dir: PathBuf, project_root: PathBuf, build_dir: PathBuf) -> Self {
        Self {
            templates_dir,
            project_root,
            build_dir,
        }
    }
}
