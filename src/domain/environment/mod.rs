//! Environment Domain Module
//!
//! This module contains all environment-related domain entities and types.
//!
//! ## Submodules
//!
//! - `name` - Environment name validation and management
//! - `state` - State marker types and type erasure for environment state machine
//!
//! ## Main Entity
//!
//! The `Environment` entity encapsulates all environment-specific configuration for deployments.
//! Each environment represents an isolated deployment context with its own directories,
//! SSH keys, and instance naming.
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
//! use torrust_tracker_deploy::domain::environment::{Environment, name::EnvironmentName};
//! use torrust_tracker_deploy::shared::{Username, ssh::SshCredentials};
//! use std::path::PathBuf;
//!
//! let env_name = EnvironmentName::new("e2e-config".to_string())?;
//! let ssh_username = Username::new("torrust".to_string())?;
//! let ssh_credentials = SshCredentials::new(
//!     PathBuf::from("fixtures/testing_rsa"),
//!     PathBuf::from("fixtures/testing_rsa.pub"),
//!     ssh_username,
//! );
//! let environment = Environment::new(env_name, ssh_credentials);
//!
//! // Environment automatically generates paths
//! assert_eq!(*environment.data_dir(), PathBuf::from("data/e2e-config"));
//! assert_eq!(*environment.build_dir(), PathBuf::from("build/e2e-config"));
//! assert_eq!(environment.templates_dir(), PathBuf::from("data/e2e-config/templates"));
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod name;
pub mod state;

// Re-export commonly used types for convenience
pub use name::{EnvironmentName, EnvironmentNameError};
pub use state::{
    AnyEnvironmentState, ConfigureFailed, Configured, Configuring, Created, Destroyed,
    ProvisionFailed, Provisioned, Provisioning, ReleaseFailed, Released, Releasing, RunFailed,
    Running,
};

use crate::domain::{InstanceName, ProfileName};
use crate::shared::{ssh::SshCredentials, Username};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Environment configuration encapsulating all environment-specific settings
///
/// This entity represents a complete environment configuration including naming,
/// directory structure, SSH keys, and derived paths. It follows the principle of
/// environment isolation where each environment has its own separate resources.
///
/// # Type-State Pattern
///
/// The Environment uses the type-state pattern to enforce valid state transitions
/// at compile-time. Each state is represented by a distinct type parameter `S`,
/// ensuring that operations are only callable on appropriate states.
///
/// # Design Principles
///
/// - **Isolation**: Each environment is completely isolated from others
/// - **Consistency**: All paths follow the same naming pattern
/// - **Predictability**: Paths are derived automatically from environment name
/// - **Traceability**: All artifacts are organized by environment for debugging
/// - **Type Safety**: Invalid state transitions are prevented at compile-time
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
pub struct Environment<S = Created> {
    /// The validated environment name
    name: EnvironmentName,

    /// The instance name for this environment (auto-generated)
    instance_name: InstanceName,

    /// The profile name for this environment (auto-generated)
    profile_name: ProfileName,

    /// SSH credentials for connecting to instances in this environment
    ssh_credentials: SshCredentials,

    /// Build directory for this environment (auto-generated)
    build_dir: PathBuf,

    /// Data directory for this environment (auto-generated)
    data_dir: PathBuf,

    /// Current state of the environment in the deployment lifecycle
    state: S,
}

impl Environment {
    /// Creates a new Environment with auto-generated paths and instance name
    ///
    /// # Arguments
    ///
    /// * `name` - The validated environment name
    /// * `ssh_credentials` - SSH credentials for connecting to instances
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
    /// use torrust_tracker_deploy::shared::{Username, ssh::SshCredentials};
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("production".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/prod_rsa"),
    ///     PathBuf::from("keys/prod_rsa.pub"),
    ///     ssh_username,
    /// );
    /// let environment = Environment::new(env_name, ssh_credentials);
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
    pub fn new(name: EnvironmentName, ssh_credentials: SshCredentials) -> Environment<Created> {
        let env_str = name.as_str();

        // Generate instance name: torrust-tracker-vm-{env_name}
        let instance_name_str = format!("torrust-tracker-vm-{env_str}");
        let instance_name = InstanceName::new(instance_name_str)
            .expect("Generated instance name should always be valid");

        // Generate profile name: torrust-profile-{env_name}
        let profile_name_str = format!("torrust-profile-{env_str}");
        let profile_name = ProfileName::new(profile_name_str)
            .expect("Generated profile name should always be valid");

        // Generate environment-specific directories
        let data_dir = PathBuf::from("data").join(env_str);
        let build_dir = PathBuf::from("build").join(env_str);

        Environment {
            name,
            instance_name,
            profile_name,
            ssh_credentials,
            build_dir,
            data_dir,
            state: Created,
        }
    }
}

// Common transitions available from any state
impl<S> Environment<S> {
    /// Internal helper: Creates a new environment with a different state
    ///
    /// This is a private helper method used by state transition methods to avoid
    /// duplicating field copying code. It transfers all environment data while
    /// changing only the state type parameter.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target state type
    ///
    /// # Arguments
    ///
    /// * `new_state` - The new state instance to transition to
    ///
    /// # Returns
    ///
    /// A new `Environment<T>` with all fields preserved except the state
    fn with_state<T>(self, new_state: T) -> Environment<T> {
        Environment {
            name: self.name,
            instance_name: self.instance_name,
            profile_name: self.profile_name,
            ssh_credentials: self.ssh_credentials,
            build_dir: self.build_dir,
            data_dir: self.data_dir,
            state: new_state,
        }
    }

    /// Transitions from any state to Destroyed state
    ///
    /// This method can be called from any state to destroy the environment.
    /// It indicates that all infrastructure resources have been released.
    #[must_use]
    pub fn destroy(self) -> Environment<Destroyed> {
        self.with_state(Destroyed)
    }
}

// Type Erasure: Typed → Runtime conversions (into_any)
// Generic implementations for all states
impl<S> Environment<S> {
    /// Returns a reference to the current state
    #[must_use]
    pub fn state(&self) -> &S {
        &self.state
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

    /// Returns the profile name for this environment
    ///
    /// Returns the unique LXD profile name based on the environment name
    /// to ensure profile isolation between different test environments.
    #[must_use]
    pub fn profile_name(&self) -> &ProfileName {
        &self.profile_name
    }

    /// Returns the SSH credentials for this environment
    #[must_use]
    pub fn ssh_credentials(&self) -> &SshCredentials {
        &self.ssh_credentials
    }

    /// Returns the SSH username for this environment
    #[must_use]
    pub fn ssh_username(&self) -> &Username {
        &self.ssh_credentials.ssh_username
    }

    /// Returns the SSH private key path for this environment
    #[must_use]
    pub fn ssh_private_key_path(&self) -> &PathBuf {
        &self.ssh_credentials.ssh_priv_key_path
    }

    /// Returns the SSH public key path for this environment
    #[must_use]
    pub fn ssh_public_key_path(&self) -> &PathBuf {
        &self.ssh_credentials.ssh_pub_key_path
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
    /// use torrust_tracker_deploy::shared::{Username, ssh::SshCredentials};
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("staging".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/staging_rsa"),
    ///     PathBuf::from("keys/staging_rsa.pub"),
    ///     ssh_username,
    /// );
    /// let environment = Environment::new(env_name, ssh_credentials);
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
    /// use torrust_tracker_deploy::shared::{Username, ssh::SshCredentials};
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("dev".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/dev_rsa"),
    ///     PathBuf::from("keys/dev_rsa.pub"),
    ///     ssh_username,
    /// );
    /// let environment = Environment::new(env_name, ssh_credentials);
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
    /// use torrust_tracker_deploy::shared::{Username, ssh::SshCredentials};
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("test".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/test_rsa"),
    ///     PathBuf::from("keys/test_rsa.pub"),
    ///     ssh_username,
    /// );
    /// let environment = Environment::new(env_name, ssh_credentials);
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
    /// use torrust_tracker_deploy::shared::{Username, ssh::SshCredentials};
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("integration".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/integration_rsa"),
    ///     PathBuf::from("keys/integration_rsa.pub"),
    ///     ssh_username,
    /// );
    /// let environment = Environment::new(env_name, ssh_credentials);
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
    /// use torrust_tracker_deploy::shared::{Username, ssh::SshCredentials};
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("load-test".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/load-test-rsa"),
    ///     PathBuf::from("keys/load-test-rsa.pub"),
    ///     ssh_username,
    /// );
    /// let environment = Environment::new(env_name, ssh_credentials);
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
    use crate::shared::ssh::SshCredentials;
    use tempfile::TempDir;

    #[test]
    fn it_should_create_environment_with_auto_generated_paths() {
        // Use a temporary directory to avoid creating real directories in the project
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        // Create a custom Environment constructor for testing that uses temporary paths
        let env_name = EnvironmentName::new("test-env".to_string()).unwrap();
        let ssh_username = Username::new("torrust".to_string()).unwrap();
        let ssh_credentials = SshCredentials::new(
            temp_path.join("testing_rsa"),
            temp_path.join("testing_rsa.pub"),
            ssh_username.clone(),
        );

        // Create environment with custom data/build dirs that point to temp
        let data_dir = temp_path.join("data").join("test-env");
        let build_dir = temp_path.join("build").join("test-env");
        let instance_name =
            InstanceName::new(format!("torrust-tracker-vm-{}", env_name.as_str())).unwrap();
        let profile_name = ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap();

        let environment = Environment {
            name: env_name.clone(),
            instance_name,
            profile_name,
            ssh_credentials: ssh_credentials.clone(),
            data_dir: data_dir.clone(),
            build_dir: build_dir.clone(),
            state: Created,
        };

        // Check basic fields
        assert_eq!(*environment.name(), env_name);
        assert_eq!(*environment.ssh_username(), ssh_username);
        assert_eq!(
            *environment.ssh_private_key_path(),
            temp_path.join("testing_rsa")
        );
        assert_eq!(
            *environment.ssh_public_key_path(),
            temp_path.join("testing_rsa.pub")
        );

        // Check auto-generated paths now point to temp directory
        assert_eq!(*environment.data_dir(), data_dir);
        assert_eq!(*environment.build_dir(), build_dir);

        // Check instance name
        assert_eq!(
            environment.instance_name().as_str(),
            "torrust-tracker-vm-test-env"
        );
    }

    #[test]
    fn it_should_generate_correct_template_directories() {
        // Use a temporary directory to avoid creating real directories in the project
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let env_name = EnvironmentName::new("test-production".to_string()).unwrap();
        let ssh_username = Username::new("torrust".to_string()).unwrap();
        let ssh_credentials = SshCredentials::new(
            temp_path.join("prod_rsa"),
            temp_path.join("prod_rsa.pub"),
            ssh_username,
        );

        // Create environment with custom paths that point to temp
        let data_dir = temp_path.join("data").join("test-production");
        let build_dir = temp_path.join("build").join("test-production");
        let instance_name =
            InstanceName::new(format!("torrust-tracker-vm-{}", env_name.as_str())).unwrap();
        let profile_name = ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap();

        let environment = Environment {
            name: env_name,
            instance_name,
            profile_name,
            ssh_credentials,
            data_dir: data_dir.clone(),
            build_dir: build_dir.clone(),
            state: Created,
        };

        assert_eq!(environment.templates_dir(), data_dir.join("templates"));
        assert_eq!(
            environment.ansible_templates_dir(),
            data_dir.join("templates").join("ansible")
        );
        assert_eq!(
            environment.tofu_templates_dir(),
            data_dir.join("templates").join("tofu")
        );
    }

    #[test]
    fn it_should_generate_correct_build_directories() {
        // Use a temporary directory to avoid creating real directories in the project
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let env_name = EnvironmentName::new("test-staging".to_string()).unwrap();
        let ssh_username = Username::new("torrust".to_string()).unwrap();
        let ssh_credentials = SshCredentials::new(
            temp_path.join("staging_rsa"),
            temp_path.join("staging_rsa.pub"),
            ssh_username,
        );

        // Create environment with custom paths that point to temp
        let data_dir = temp_path.join("data").join("test-staging");
        let build_dir = temp_path.join("build").join("test-staging");
        let instance_name =
            InstanceName::new(format!("torrust-tracker-vm-{}", env_name.as_str())).unwrap();
        let profile_name = ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap();

        let environment = Environment {
            name: env_name,
            instance_name,
            profile_name,
            ssh_credentials,
            data_dir: data_dir.clone(),
            build_dir: build_dir.clone(),
            state: Created,
        };

        assert_eq!(environment.ansible_build_dir(), build_dir.join("ansible"));
        assert_eq!(environment.tofu_build_dir(), build_dir.join("tofu"));
    }

    #[test]
    fn it_should_handle_different_environment_names() {
        // Use a temporary directory to avoid creating real directories in the project
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let test_cases = vec![
            ("test-dev", "torrust-tracker-vm-test-dev"),
            (
                "test-e2e-provision",
                "torrust-tracker-vm-test-e2e-provision",
            ),
            ("test-integration", "torrust-tracker-vm-test-integration"),
            ("test-release-v1-2", "torrust-tracker-vm-test-release-v1-2"),
        ];

        for (env_name_str, expected_instance_name) in test_cases {
            let env_name = EnvironmentName::new(env_name_str.to_string()).unwrap();
            let ssh_username = Username::new("torrust".to_string()).unwrap();
            let ssh_credentials = SshCredentials::new(
                temp_path.join("test_key"),
                temp_path.join("test_key.pub"),
                ssh_username,
            );

            // Create environment with custom paths that point to temp
            let data_dir = temp_path.join("data").join(env_name_str);
            let build_dir = temp_path.join("build").join(env_name_str);
            let instance_name =
                InstanceName::new(format!("torrust-tracker-vm-{}", env_name.as_str())).unwrap();
            let profile_name = ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap();

            let environment = Environment {
                name: env_name,
                instance_name: instance_name.clone(),
                profile_name,
                ssh_credentials,
                data_dir: data_dir.clone(),
                build_dir: build_dir.clone(),
                state: Created,
            };

            assert_eq!(environment.instance_name().as_str(), expected_instance_name);
            assert_eq!(*environment.data_dir(), data_dir);
            assert_eq!(*environment.build_dir(), build_dir);
        }
    }

    #[test]
    fn it_should_be_serializable_to_json() {
        // Use a temporary directory to avoid creating real directories in the project
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let env_name = EnvironmentName::new("test-serialization".to_string()).unwrap();
        let ssh_username = Username::new("torrust".to_string()).unwrap();
        let ssh_credentials = SshCredentials::new(
            temp_path.join("test_private_key"),
            temp_path.join("test_public_key"),
            ssh_username,
        );

        // Create environment with custom paths that point to temp
        let data_dir = temp_path.join("data").join("test-serialization");
        let build_dir = temp_path.join("build").join("test-serialization");
        let instance_name =
            InstanceName::new(format!("torrust-tracker-vm-{}", env_name.as_str())).unwrap();
        let profile_name = ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap();

        let environment = Environment {
            name: env_name,
            instance_name,
            profile_name,
            ssh_credentials,
            data_dir: data_dir.clone(),
            build_dir: build_dir.clone(),
            state: Created,
        };

        // Serialize to JSON
        let json = serde_json::to_string(&environment).unwrap();

        // Deserialize back
        let deserialized: Environment = serde_json::from_str(&json).unwrap();

        // Check that all fields are preserved
        assert_eq!(deserialized.name().as_str(), "test-serialization");
        assert_eq!(
            deserialized.instance_name().as_str(),
            "torrust-tracker-vm-test-serialization"
        );
        assert_eq!(
            *deserialized.ssh_private_key_path(),
            temp_path.join("test_private_key")
        );
        assert_eq!(
            *deserialized.ssh_public_key_path(),
            temp_path.join("test_public_key")
        );
        assert_eq!(*deserialized.data_dir(), data_dir);
        assert_eq!(*deserialized.build_dir(), build_dir);
    }

    #[test]
    fn it_should_support_common_e2e_environment_names() {
        // Use a temporary directory to avoid creating real directories in the project
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let e2e_environments = vec!["test-e2e-config", "test-e2e-provision", "test-e2e-full"];

        for env_name_str in e2e_environments {
            let env_name = EnvironmentName::new(env_name_str.to_string()).unwrap();
            let ssh_username = Username::new("torrust".to_string()).unwrap();
            let ssh_credentials = SshCredentials::new(
                temp_path.join("testing_rsa"),
                temp_path.join("testing_rsa.pub"),
                ssh_username,
            );

            // Create environment with custom paths that point to temp
            let data_dir = temp_path.join("data").join(env_name_str);
            let build_dir = temp_path.join("build").join(env_name_str);
            let instance_name =
                InstanceName::new(format!("torrust-tracker-vm-{}", env_name.as_str())).unwrap();
            let profile_name = ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap();

            let environment = Environment {
                name: env_name,
                instance_name,
                profile_name,
                ssh_credentials,
                data_dir: data_dir.clone(),
                build_dir: build_dir.clone(),
                state: Created,
            };

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
        // Use a temporary directory to avoid creating real directories in the project
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path();

        let env_name = EnvironmentName::new("test-feature-user-auth".to_string()).unwrap();
        let ssh_username = Username::new("torrust".to_string()).unwrap();
        let ssh_credentials = SshCredentials::new(
            temp_path.join("feature_rsa"),
            temp_path.join("feature_rsa.pub"),
            ssh_username,
        );

        // Create environment with custom paths that point to temp
        let data_dir = temp_path.join("data").join("test-feature-user-auth");
        let build_dir = temp_path.join("build").join("test-feature-user-auth");
        let instance_name =
            InstanceName::new(format!("torrust-tracker-vm-{}", env_name.as_str())).unwrap();
        let profile_name = ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap();

        let environment = Environment {
            name: env_name,
            instance_name,
            profile_name,
            ssh_credentials,
            data_dir: data_dir.clone(),
            build_dir: build_dir.clone(),
            state: Created,
        };

        assert_eq!(
            environment.instance_name().as_str(),
            "torrust-tracker-vm-test-feature-user-auth"
        );
        assert_eq!(*environment.data_dir(), data_dir);
        assert_eq!(*environment.build_dir(), build_dir);
        assert_eq!(environment.templates_dir(), data_dir.join("templates"));
    }

    // State transition tests
    mod state_transitions {
        use super::*;

        fn create_test_environment() -> Environment<Created> {
            let env_name = EnvironmentName::new("test-state".to_string()).unwrap();
            let ssh_username = Username::new("torrust".to_string()).unwrap();
            let ssh_credentials = SshCredentials::new(
                PathBuf::from("test_key"),
                PathBuf::from("test_key.pub"),
                ssh_username,
            );
            Environment::new(env_name, ssh_credentials)
        }

        #[test]
        fn it_should_transition_to_destroyed_from_created() {
            let env = create_test_environment();
            let env = env.destroy();

            assert_eq!(*env.state(), Destroyed);
            assert_eq!(env.name().as_str(), "test-state");
        }

        #[test]
        fn it_should_complete_full_happy_path_transition() {
            let env = create_test_environment();

            // Complete happy path: Created -> Running
            let env = env
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released()
                .start_running();

            assert_eq!(*env.state(), Running);
            assert_eq!(env.name().as_str(), "test-state");

            // Then destroy
            let env = env.destroy();
            assert_eq!(*env.state(), Destroyed);
        }

        #[test]
        fn it_should_preserve_all_fields_during_transitions() {
            let env = create_test_environment();
            let initial_name = env.name().clone();
            let initial_instance_name = env.instance_name().clone();
            let initial_data_dir = env.data_dir().clone();
            let initial_build_dir = env.build_dir().clone();

            // Go through several transitions
            let env = env
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured();

            // Verify all fields are preserved
            assert_eq!(env.name(), &initial_name);
            assert_eq!(env.instance_name(), &initial_instance_name);
            assert_eq!(env.data_dir(), &initial_data_dir);
            assert_eq!(env.build_dir(), &initial_build_dir);
        }
    }
}
