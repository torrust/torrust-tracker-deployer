//! Environment Domain Entity
//!
//! This module defines the `Environment` domain entity which encapsulates all
//! environment-specific configuration for deployments. Each environment represents
//! an isolated deployment context with its own directories, SSH keys, and instance naming.
//!
//! ## Purpose
//!
//! The Environment entity provides:
//! - Environment-specific directory structure (`data/{env_name}/`, `build/{env_name}/`)
//! - Instance naming with conflict avoidance (`torrust-tracker-vm-{env_name}`)
//! - SSH key pair management per environment
//! - JSON serialization for future state persistence
//!
//! ## Usage Example
//!
//! ```rust
//! use torrust_tracker_deploy::domain::{Environment, EnvironmentName};
//! use torrust_tracker_deploy::shared::Username;
//! use std::path::PathBuf;
//!
//! let env_name = EnvironmentName::new("e2e-config".to_string())?;
//! let ssh_username = Username::new("torrust".to_string())?;
//! let environment = Environment::new(
//!     env_name,
//!     ssh_username,
//!     PathBuf::from("fixtures/testing_rsa"),
//!     PathBuf::from("fixtures/testing_rsa.pub"),
//! );
//!
//! // Environment automatically generates paths
//! assert_eq!(*environment.data_dir(), PathBuf::from("data/e2e-config"));
//! assert_eq!(*environment.build_dir(), PathBuf::from("build/e2e-config"));
//! assert_eq!(environment.templates_dir(), PathBuf::from("data/e2e-config/templates"));
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::config::InstanceName;
use crate::domain::EnvironmentName;
use crate::shared::Username;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Environment configuration encapsulating all environment-specific settings
///
/// This entity represents a complete environment configuration including naming,
/// directory structure, SSH keys, and derived paths. It follows the principle of
/// environment isolation where each environment has its own separate resources.
///
/// # Design Principles
///
/// - **Isolation**: Each environment is completely isolated from others
/// - **Consistency**: All paths follow the same naming pattern
/// - **Predictability**: Paths are derived automatically from environment name
/// - **Traceability**: All artifacts are organized by environment for debugging
///
/// # Directory Structure
///
/// ```text
/// data/{env_name}/
///   templates/         # Environment-specific templates
/// build/{env_name}/    # Environment-specific build artifacts
/// ```
///
/// # Instance Naming
///
/// Instance names follow the pattern: `torrust-tracker-vm-{env_name}`
/// This ensures multiple environments can run concurrently without conflicts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    /// The validated environment name
    name: EnvironmentName,

    /// The instance name for this environment (auto-generated)
    instance_name: InstanceName,

    /// SSH username for connecting to instances in this environment
    ssh_username: Username,

    /// Path to the SSH private key for this environment
    ssh_private_key_path: PathBuf,

    /// Path to the SSH public key for this environment
    ssh_public_key_path: PathBuf,

    /// Build directory for this environment (auto-generated)
    build_dir: PathBuf,

    /// Data directory for this environment (auto-generated)
    data_dir: PathBuf,
}

impl Environment {
    /// Creates a new Environment with auto-generated paths and instance name
    ///
    /// # Arguments
    ///
    /// * `name` - The validated environment name
    /// * `ssh_username` - SSH username for connecting to instances
    /// * `ssh_private_key_path` - Path to the SSH private key file
    /// * `ssh_public_key_path` - Path to the SSH public key file
    ///
    /// # Returns
    ///
    /// A new Environment instance with all paths and instance name automatically
    /// generated based on the environment name.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deploy::domain::{Environment, EnvironmentName};
    /// use torrust_tracker_deploy::shared::Username;
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("production".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let environment = Environment::new(
    ///     env_name,
    ///     ssh_username,
    ///     PathBuf::from("keys/prod_rsa"),
    ///     PathBuf::from("keys/prod_rsa.pub"),
    /// );
    ///
    /// assert_eq!(environment.instance_name().as_str(), "torrust-tracker-vm-production");
    /// assert_eq!(*environment.data_dir(), PathBuf::from("data/production"));
    /// assert_eq!(*environment.build_dir(), PathBuf::from("build/production"));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Panics
    ///
    /// This function does not panic. All instance name generation is guaranteed
    /// to succeed for valid environment names.
    #[must_use]
    pub fn new(
        name: EnvironmentName,
        ssh_username: Username,
        ssh_private_key_path: PathBuf,
        ssh_public_key_path: PathBuf,
    ) -> Self {
        let env_str = name.as_str();

        // Generate instance name: torrust-tracker-vm-{env_name}
        let instance_name_str = format!("torrust-tracker-vm-{env_str}");
        let instance_name = InstanceName::new(instance_name_str)
            .expect("Generated instance name should always be valid");

        // Generate environment-specific directories
        let data_dir = PathBuf::from("data").join(env_str);
        let build_dir = PathBuf::from("build").join(env_str);

        Self {
            name,
            instance_name,
            ssh_username,
            ssh_private_key_path,
            ssh_public_key_path,
            build_dir,
            data_dir,
        }
    }

    /// Returns the environment name
    #[must_use]
    pub fn name(&self) -> &EnvironmentName {
        &self.name
    }

    /// Returns the instance name for this environment
    #[must_use]
    pub fn instance_name(&self) -> &InstanceName {
        &self.instance_name
    }

    /// Returns the SSH username for this environment
    #[must_use]
    pub fn ssh_username(&self) -> &Username {
        &self.ssh_username
    }

    /// Returns the SSH private key path for this environment
    #[must_use]
    pub fn ssh_private_key_path(&self) -> &PathBuf {
        &self.ssh_private_key_path
    }

    /// Returns the SSH public key path for this environment
    #[must_use]
    pub fn ssh_public_key_path(&self) -> &PathBuf {
        &self.ssh_public_key_path
    }

    /// Returns the build directory for this environment
    #[must_use]
    pub fn build_dir(&self) -> &PathBuf {
        &self.build_dir
    }

    /// Returns the data directory for this environment
    #[must_use]
    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    /// Returns the templates directory for this environment
    ///
    /// The templates directory is located at `data/{env_name}/templates/`
    /// and contains environment-specific template files.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deploy::domain::{Environment, EnvironmentName};
    /// use torrust_tracker_deploy::shared::Username;
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("staging".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let environment = Environment::new(
    ///     env_name,
    ///     ssh_username,
    ///     PathBuf::from("keys/staging_rsa"),
    ///     PathBuf::from("keys/staging_rsa.pub"),
    /// );
    ///
    /// assert_eq!(
    ///     environment.templates_dir(),
    ///     PathBuf::from("data/staging/templates")
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn templates_dir(&self) -> PathBuf {
        self.data_dir.join("templates")
    }

    /// Returns the ansible build directory for this environment
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deploy::domain::{Environment, EnvironmentName};
    /// use torrust_tracker_deploy::shared::Username;
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("dev".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let environment = Environment::new(
    ///     env_name,
    ///     ssh_username,
    ///     PathBuf::from("keys/dev_rsa"),
    ///     PathBuf::from("keys/dev_rsa.pub"),
    /// );
    ///
    /// assert_eq!(
    ///     environment.ansible_build_dir(),
    ///     PathBuf::from("build/dev/ansible")
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn ansible_build_dir(&self) -> PathBuf {
        self.build_dir.join("ansible")
    }

    /// Returns the tofu build directory for this environment
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deploy::domain::{Environment, EnvironmentName};
    /// use torrust_tracker_deploy::shared::Username;
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("test".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let environment = Environment::new(
    ///     env_name,
    ///     ssh_username,
    ///     PathBuf::from("keys/test_rsa"),
    ///     PathBuf::from("keys/test_rsa.pub"),
    /// );
    ///
    /// assert_eq!(
    ///     environment.tofu_build_dir(),
    ///     PathBuf::from("build/test/tofu")
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn tofu_build_dir(&self) -> PathBuf {
        self.build_dir.join("tofu")
    }

    /// Returns the ansible templates directory for this environment
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deploy::domain::{Environment, EnvironmentName};
    /// use torrust_tracker_deploy::shared::Username;
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("integration".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let environment = Environment::new(
    ///     env_name,
    ///     ssh_username,
    ///     PathBuf::from("keys/integration_rsa"),
    ///     PathBuf::from("keys/integration_rsa.pub"),
    /// );
    ///
    /// assert_eq!(
    ///     environment.ansible_templates_dir(),
    ///     PathBuf::from("data/integration/templates/ansible")
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn ansible_templates_dir(&self) -> PathBuf {
        self.templates_dir().join("ansible")
    }

    /// Returns the tofu templates directory for this environment
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deploy::domain::{Environment, EnvironmentName};
    /// use torrust_tracker_deploy::shared::Username;
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("load-test".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let environment = Environment::new(
    ///     env_name,
    ///     ssh_username,
    ///     PathBuf::from("keys/load-test-rsa"),
    ///     PathBuf::from("keys/load-test-rsa.pub"),
    /// );
    ///
    /// assert_eq!(
    ///     environment.tofu_templates_dir(),
    ///     PathBuf::from("data/load-test/templates/tofu")
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn tofu_templates_dir(&self) -> PathBuf {
        self.templates_dir().join("tofu")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::EnvironmentName;

    #[test]
    fn it_should_create_environment_with_auto_generated_paths() {
        let env_name = EnvironmentName::new("e2e-config".to_string()).unwrap();
        let ssh_username = Username::new("torrust".to_string()).unwrap();
        let environment = Environment::new(
            env_name.clone(),
            ssh_username.clone(),
            PathBuf::from("fixtures/testing_rsa"),
            PathBuf::from("fixtures/testing_rsa.pub"),
        );

        // Check basic fields
        assert_eq!(*environment.name(), env_name);
        assert_eq!(*environment.ssh_username(), ssh_username);
        assert_eq!(
            *environment.ssh_private_key_path(),
            PathBuf::from("fixtures/testing_rsa")
        );
        assert_eq!(
            *environment.ssh_public_key_path(),
            PathBuf::from("fixtures/testing_rsa.pub")
        );

        // Check auto-generated paths
        assert_eq!(*environment.data_dir(), PathBuf::from("data/e2e-config"));
        assert_eq!(*environment.build_dir(), PathBuf::from("build/e2e-config"));

        // Check instance name
        assert_eq!(
            environment.instance_name().as_str(),
            "torrust-tracker-vm-e2e-config"
        );
    }

    #[test]
    fn it_should_generate_correct_template_directories() {
        let env_name = EnvironmentName::new("production".to_string()).unwrap();
        let ssh_username = Username::new("torrust".to_string()).unwrap();
        let environment = Environment::new(
            env_name,
            ssh_username,
            PathBuf::from("keys/prod_rsa"),
            PathBuf::from("keys/prod_rsa.pub"),
        );

        assert_eq!(
            environment.templates_dir(),
            PathBuf::from("data/production/templates")
        );
        assert_eq!(
            environment.ansible_templates_dir(),
            PathBuf::from("data/production/templates/ansible")
        );
        assert_eq!(
            environment.tofu_templates_dir(),
            PathBuf::from("data/production/templates/tofu")
        );
    }

    #[test]
    fn it_should_generate_correct_build_directories() {
        let env_name = EnvironmentName::new("staging".to_string()).unwrap();
        let ssh_username = Username::new("torrust".to_string()).unwrap();
        let environment = Environment::new(
            env_name,
            ssh_username,
            PathBuf::from("keys/staging_rsa"),
            PathBuf::from("keys/staging_rsa.pub"),
        );

        assert_eq!(
            environment.ansible_build_dir(),
            PathBuf::from("build/staging/ansible")
        );
        assert_eq!(
            environment.tofu_build_dir(),
            PathBuf::from("build/staging/tofu")
        );
    }

    #[test]
    fn it_should_handle_different_environment_names() {
        let test_cases = vec![
            ("dev", "torrust-tracker-vm-dev"),
            ("e2e-provision", "torrust-tracker-vm-e2e-provision"),
            ("test-integration", "torrust-tracker-vm-test-integration"),
            ("release-v1-2", "torrust-tracker-vm-release-v1-2"),
        ];

        for (env_name_str, expected_instance_name) in test_cases {
            let env_name = EnvironmentName::new(env_name_str.to_string()).unwrap();
            let ssh_username = Username::new("torrust".to_string()).unwrap();
            let environment = Environment::new(
                env_name,
                ssh_username,
                PathBuf::from("test_key"),
                PathBuf::from("test_key.pub"),
            );

            assert_eq!(environment.instance_name().as_str(), expected_instance_name);
            assert_eq!(
                *environment.data_dir(),
                PathBuf::from(format!("data/{env_name_str}"))
            );
            assert_eq!(
                *environment.build_dir(),
                PathBuf::from(format!("build/{env_name_str}"))
            );
        }
    }

    #[test]
    fn it_should_be_serializable_to_json() {
        let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
        let ssh_username = Username::new("torrust".to_string()).unwrap();
        let environment = Environment::new(
            env_name,
            ssh_username,
            PathBuf::from("test_private_key"),
            PathBuf::from("test_public_key"),
        );

        // Serialize to JSON
        let json = serde_json::to_string(&environment).unwrap();

        // Deserialize back
        let deserialized: Environment = serde_json::from_str(&json).unwrap();

        // Check that all fields are preserved
        assert_eq!(deserialized.name().as_str(), "test-env");
        assert_eq!(
            deserialized.instance_name().as_str(),
            "torrust-tracker-vm-test-env"
        );
        assert_eq!(
            *deserialized.ssh_private_key_path(),
            PathBuf::from("test_private_key")
        );
        assert_eq!(
            *deserialized.ssh_public_key_path(),
            PathBuf::from("test_public_key")
        );
        assert_eq!(*deserialized.data_dir(), PathBuf::from("data/test-env"));
        assert_eq!(*deserialized.build_dir(), PathBuf::from("build/test-env"));
    }

    #[test]
    fn it_should_support_common_e2e_environment_names() {
        let e2e_environments = vec!["e2e-config", "e2e-provision", "e2e-full"];

        for env_name_str in e2e_environments {
            let env_name = EnvironmentName::new(env_name_str.to_string()).unwrap();
            let ssh_username = Username::new("torrust".to_string()).unwrap();
            let environment = Environment::new(
                env_name,
                ssh_username,
                PathBuf::from("fixtures/testing_rsa"),
                PathBuf::from("fixtures/testing_rsa.pub"),
            );

            // Verify the environment is created successfully
            assert_eq!(environment.name().as_str(), env_name_str);
            assert!(environment
                .instance_name()
                .as_str()
                .starts_with("torrust-tracker-vm-"));
            assert!(environment
                .data_dir
                .to_string_lossy()
                .contains(env_name_str));
            assert!(environment
                .build_dir
                .to_string_lossy()
                .contains(env_name_str));
        }
    }

    #[test]
    fn it_should_handle_dash_separated_environment_names() {
        let env_name = EnvironmentName::new("feature-user-auth".to_string()).unwrap();
        let ssh_username = Username::new("torrust".to_string()).unwrap();
        let environment = Environment::new(
            env_name,
            ssh_username,
            PathBuf::from("keys/feature_rsa"),
            PathBuf::from("keys/feature_rsa.pub"),
        );

        assert_eq!(
            environment.instance_name().as_str(),
            "torrust-tracker-vm-feature-user-auth"
        );
        assert_eq!(
            *environment.data_dir(),
            PathBuf::from("data/feature-user-auth")
        );
        assert_eq!(
            *environment.build_dir(),
            PathBuf::from("build/feature-user-auth")
        );
        assert_eq!(
            environment.templates_dir(),
            PathBuf::from("data/feature-user-auth/templates")
        );
    }
}
