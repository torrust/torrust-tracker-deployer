//! Configuration management for deployment settings
//!
//! This module provides the `Config` struct which centralizes all deployment-related
//! configuration including file paths, service connections, SSH credentials,
//! and runtime behavior settings.
//!
//! ## Key Configuration Areas
//!
//! - SSH credentials and connection settings
//! - File system paths for templates and build outputs
//! - Deployment behavior flags (cleanup, verbosity, etc.)
//! - Tool-specific configuration (Ansible, `OpenTofu`)
//!
//! The configuration is typically created once at deployment start and passed
//! throughout the system to ensure consistent settings across all components.

use std::path::PathBuf;

/// Configuration parameters for deployment environments.
///
/// Centralizes all deployment-related configuration including file paths,
/// service connection details, and runtime behavior settings.
///
/// Created once at deployment start and passed to [`Services::new()`](crate::container::Services::new).
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
    /// # use torrust_tracker_deploy::config::Config;
    /// let config = Config::new(
    ///     PathBuf::from("templates"),
    ///     PathBuf::from("/path/to/project"),
    ///     PathBuf::from("/path/to/project/build"),
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
