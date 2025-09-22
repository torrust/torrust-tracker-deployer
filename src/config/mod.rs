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

pub use crate::command_wrappers::lxd::InstanceName;
pub use crate::command_wrappers::ssh::{SshConnection, SshCredentials};

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

    /// SSH credentials for remote connections.
    ///
    /// Contains SSH key paths and username settings that will be used
    /// when connecting to deployed instances. The host IP will be determined
    /// later when instances are provisioned.
    pub ssh_credentials: SshCredentials,

    /// Name for the instance to be deployed.
    ///
    /// This name will be used for creating LXD containers/VMs and referenced
    /// in various configuration files. Must follow LXD naming requirements:
    /// 1-63 characters, ASCII letters/numbers/dashes, cannot start with digit/dash,
    /// cannot end with dash.
    pub instance_name: InstanceName,

    /// Subdirectory name for Ansible-related files within the build directory.
    ///
    /// Ansible playbooks, inventory files, and configuration templates
    /// will be rendered to `build_dir/{ansible_subfolder}/`.
    pub ansible_subfolder: String,

    /// Subdirectory name for OpenTofu-related files within the build directory.
    ///
    /// OpenTofu/Terraform configuration files and state will be managed
    /// in `build_dir/{opentofu_subfolder}/`. Example: "tofu/lxd".
    pub opentofu_subfolder: String,

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
    /// The `ansible_subfolder` is set to "ansible" and `opentofu_subfolder` is set to "tofu/lxd" internally.
    ///
    /// ```rust
    /// # use std::net::{IpAddr, Ipv4Addr};
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::config::{Config, SshCredentials, InstanceName};
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     "ubuntu".to_string(),
    /// );
    /// let instance_name = InstanceName::new("my-instance".to_string()).unwrap();
    /// let config = Config::new(
    ///     true,                           // keep environment for debugging
    ///     ssh_credentials,
    ///     instance_name,
    ///     "templates".to_string(),
    ///     PathBuf::from("/path/to/project"),
    ///     PathBuf::from("/path/to/project/build"),
    /// );
    /// ```
    #[must_use]
    pub fn new(
        keep_env: bool,
        ssh_config: SshCredentials,
        instance_name: InstanceName,
        templates_dir: String,
        project_root: PathBuf,
        build_dir: PathBuf,
    ) -> Self {
        Self {
            keep_env,
            ssh_credentials: ssh_config,
            instance_name,
            ansible_subfolder: "ansible".to_string(),
            opentofu_subfolder: "tofu/lxd".to_string(),
            templates_dir,
            project_root,
            build_dir,
        }
    }
}
