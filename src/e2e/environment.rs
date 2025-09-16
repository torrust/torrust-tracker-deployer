use anyhow::{Context, Result};
use tempfile::TempDir;
use tracing::info;

use crate::config::{Config, SshCredentials};
use crate::container::Services;

use super::tasks::clean_and_prepare_templates::clean_and_prepare_templates;
use super::tasks::setup_ssh_key::setup_ssh_key;

/// Main test environment combining configuration and services
pub struct TestEnvironment {
    pub config: Config,
    pub services: Services,
    #[allow(dead_code)] // Kept to maintain temp directory lifetime for tests
    temp_dir: Option<tempfile::TempDir>,
}

impl TestEnvironment {
    /// Creates a new test environment with configuration and services
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Current directory cannot be determined
    /// - Temporary directory creation fails
    /// - SSH key setup fails
    /// - Template preparation fails
    pub fn new(keep_env: bool, templates_dir: String) -> Result<Self> {
        // Get project root (current directory when running from root)
        let project_root = std::env::current_dir()?;

        // Create temporary directory for SSH keys
        let temp_dir = TempDir::new().context("Failed to create temporary directory")?;

        // Setup SSH key
        let temp_ssh_key = temp_dir.path().join("testing_rsa");
        let temp_ssh_pub_key = temp_dir.path().join("testing_rsa.pub");
        setup_ssh_key(&project_root, &temp_dir)?;

        // Create SSH credentials (no host IP needed yet)
        let ssh_credentials =
            SshCredentials::new(temp_ssh_key, temp_ssh_pub_key, "torrust".to_string());

        // Create main configuration
        let config = Config::new(
            keep_env,
            ssh_credentials,
            templates_dir,
            project_root.clone(),
            project_root.join("build"),
        );

        // Create services using the configuration
        let services = Services::new(&config);

        // Clean and prepare templates directory
        clean_and_prepare_templates(&services)?;

        info!(
            environment = "temporary_directory",
            path = %temp_dir.path().display(),
            "Temporary directory created"
        );

        info!(
            environment = "templates_directory",
            path = %services.template_manager.templates_dir().display(),
            "Templates directory configured"
        );

        Ok(Self {
            config,
            services,
            temp_dir: Some(temp_dir),
        })
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        if !self.config.keep_env {
            // Try basic cleanup in case async cleanup failed
            // Using emergency_destroy for consistent OpenTofu handling
            let tofu_dir = self.config.build_dir.join(&self.config.opentofu_subfolder);

            drop(crate::command_wrappers::opentofu::emergency_destroy(
                &tofu_dir,
            ));
        }
    }
}
