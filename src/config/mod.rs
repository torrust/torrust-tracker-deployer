use std::path::PathBuf;

pub mod ssh;
pub use ssh::SshConfig;

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

    /// SSH configuration for remote connections.
    ///
    /// Contains SSH key path and username settings for connecting to
    /// deployed instances during the deployment process.
    pub ssh_config: SshConfig,

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

    /// Original inventory content for restoration during cleanup.
    ///
    /// Stores the original Ansible inventory file content so it can be
    /// restored if the deployment process modifies it. Used internally
    /// for cleanup operations.
    pub original_inventory: Option<String>,
}

impl Config {
    /// Creates a new configuration with the provided parameters.
    ///
    /// Sets `original_inventory` to `None` - this field is populated internally during cleanup operations.
    /// The `ansible_subfolder` is set to "ansible" and `opentofu_subfolder` is set to "tofu/lxd" internally.
    ///
    /// ```rust
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::config::{Config, SshConfig};
    /// let ssh_config = SshConfig::new(
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     PathBuf::from("/home/user/.ssh/deploy_key.pub"),
    ///     "ubuntu".to_string(),
    /// );
    /// let config = Config::new(
    ///     true,                           // keep environment for debugging
    ///     ssh_config,
    ///     "templates".to_string(),
    ///     PathBuf::from("/path/to/project"),
    ///     PathBuf::from("/path/to/project/build"),
    /// );
    /// ```
    #[must_use]
    pub fn new(
        keep_env: bool,
        ssh_config: SshConfig,
        templates_dir: String,
        project_root: PathBuf,
        build_dir: PathBuf,
    ) -> Self {
        Self {
            keep_env,
            ssh_config,
            ansible_subfolder: "ansible".to_string(),
            opentofu_subfolder: "tofu/lxd".to_string(),
            templates_dir,
            project_root,
            build_dir,
            original_inventory: None,
        }
    }
}
