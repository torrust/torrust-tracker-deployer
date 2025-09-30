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
    /// Whether to keep the deployment environment after completion.
    ///
    /// When `false`, the environment will be automatically cleaned up (destroyed)
    /// after the deployment process completes. When `true`, the environment
    /// will be left running for manual inspection or reuse.
    pub keep_env: bool,

    /// Directory containing template files for rendering configurations.
    ///
    /// This directory should contain subdirectories for different template
    /// types (e.g., "ansible/", "tofu/") with template files that will be
    /// processed and rendered to the build directory.
    pub templates_dir: String,

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
    ///     true,                           // keep environment for debugging
    ///     "templates".to_string(),
    ///     PathBuf::from("/path/to/project"),
    ///     PathBuf::from("/path/to/project/build"),
    /// );
    /// ```
    #[must_use]
    pub fn new(
        keep_env: bool,
        templates_dir: String,
        project_root: PathBuf,
        build_dir: PathBuf,
    ) -> Self {
        Self {
            keep_env,
            templates_dir,
            project_root,
            build_dir,
        }
    }
}
