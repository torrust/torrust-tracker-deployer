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

    /// Path to the SSH private key file for remote connections.
    ///
    /// This key will be used by the SSH client to authenticate with remote
    /// instances created during deployment. The corresponding public key
    /// should be authorized on the target instances.
    pub ssh_key_path: PathBuf,

    /// Username for SSH connections to remote instances.
    ///
    /// This username will be used when establishing SSH connections to
    /// deployed instances. Common values include "ubuntu", "root", or "torrust".
    pub ssh_username: String,

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
    #[allow(dead_code)] // Still used for SSH key fixtures and cleanup
    pub project_root: PathBuf,

    /// Directory where rendered configuration files will be written.
    ///
    /// All processed templates and generated configuration files are written
    /// to subdirectories within this build directory. This directory is
    /// typically git-ignored to avoid committing generated files.
    #[allow(dead_code)] // Will be used in template rendering
    pub build_dir: PathBuf,

    /// Original inventory content for restoration during cleanup.
    ///
    /// Stores the original Ansible inventory file content so it can be
    /// restored if the deployment process modifies it. Used internally
    /// for cleanup operations.
    #[allow(dead_code)] // Used for cleanup but not directly accessed
    pub original_inventory: Option<String>,
}

impl Config {
    /// Creates a new configuration with the provided parameters.
    ///
    /// Sets `original_inventory` to `None` - this field is populated internally during cleanup operations.
    ///
    /// ```rust
    /// # use std::path::PathBuf;
    /// # use torrust_tracker_deploy::config::Config;
    /// let config = Config::new(
    ///     true,                           // keep environment for debugging
    ///     PathBuf::from("/home/user/.ssh/deploy_key"),
    ///     "ubuntu".to_string(),
    ///     "ansible".to_string(),
    ///     "tofu/lxd".to_string(),
    ///     "templates".to_string(),
    ///     PathBuf::from("/path/to/project"),
    ///     PathBuf::from("/path/to/project/build"),
    /// );
    /// ```
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        keep_env: bool,
        ssh_key_path: PathBuf,
        ssh_username: String,
        ansible_subfolder: String,
        opentofu_subfolder: String,
        templates_dir: String,
        project_root: PathBuf,
        build_dir: PathBuf,
    ) -> Self {
        Self {
            keep_env,
            ssh_key_path,
            ssh_username,
            ansible_subfolder,
            opentofu_subfolder,
            templates_dir,
            project_root,
            build_dir,
            original_inventory: None,
        }
    }
}
