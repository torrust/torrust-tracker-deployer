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
//! let environment = Environment::new(env_name, ssh_credentials, 22);
//!
//! // Environment automatically generates paths
//! assert_eq!(*environment.data_dir(), PathBuf::from("data/e2e-config"));
//! assert_eq!(*environment.build_dir(), PathBuf::from("build/e2e-config"));
//! assert_eq!(environment.templates_dir(), PathBuf::from("data/e2e-config/templates"));
//!
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod name;
pub mod repository;
pub mod state;
mod trace_id;

// Test utilities (only available in test configuration)
#[cfg(test)]
pub mod testing;

// Re-export TraceId for use by state module
pub use trace_id::TraceId;

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
use std::net::IpAddr;
use std::path::PathBuf;

/// Directory name for trace files within an environment's data directory
pub const TRACES_DIR_NAME: &str = "traces";

/// Core environment data that remains constant across all states
///
/// This struct contains all fields that do not change when the environment
/// transitions between states. Extracting these fields eliminates repetitive
/// pattern matching in `AnyEnvironmentState` while maintaining the type-state
/// pattern's compile-time guarantees.
///
/// # Design Rationale
///
/// By separating state-independent data from the state machine, we:
/// - Eliminate repetitive pattern matching in `AnyEnvironmentState`
/// - Make it clear which data is constant vs. state-dependent
/// - Simplify state transitions (only the state field changes)
/// - Enable easier extension of environment configuration
///
/// # Field Overview
///
/// - **Identity**: `name`, `instance_name`, `profile_name`
/// - **Configuration**: `ssh_credentials`, `ssh_port`
/// - **Paths**: `build_dir`, `data_dir`
/// - **Runtime State**: `instance_ip` (populated after provisioning)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    /// The validated environment name
    pub(crate) name: EnvironmentName,

    /// The instance name for this environment (auto-generated)
    pub(crate) instance_name: InstanceName,

    /// The profile name for this environment (auto-generated)
    pub(crate) profile_name: ProfileName,

    /// SSH credentials for connecting to instances in this environment
    pub(crate) ssh_credentials: SshCredentials,

    /// SSH port for connecting to instances in this environment
    pub(crate) ssh_port: u16,

    /// Build directory for this environment (auto-generated)
    pub(crate) build_dir: PathBuf,

    /// Data directory for this environment (auto-generated)
    pub(crate) data_dir: PathBuf,

    /// Instance IP address (populated after provisioning)
    ///
    /// This field stores the IP address of the provisioned instance and is
    /// `None` until the environment has been successfully provisioned.
    /// Once set, it's carried through all subsequent state transitions.
    pub(crate) instance_ip: Option<IpAddr>,
}

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

    /// SSH port for connecting to instances in this environment
    ssh_port: u16,

    /// Build directory for this environment (auto-generated)
    build_dir: PathBuf,

    /// Data directory for this environment (auto-generated)
    data_dir: PathBuf,

    /// Instance IP address (populated after provisioning)
    ///
    /// This field stores the IP address of the provisioned instance and is
    /// `None` until the environment has been successfully provisioned.
    /// Once set, it's carried through all subsequent state transitions.
    instance_ip: Option<IpAddr>,

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
    /// * `ssh_port` - SSH port for connecting to instances
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
    /// let ssh_port = 22;
    /// let environment = Environment::new(env_name, ssh_credentials, ssh_port);
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
        ssh_credentials: SshCredentials,
        ssh_port: u16,
    ) -> Environment<Created> {
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
            ssh_port,
            build_dir,
            data_dir,
            instance_ip: None,
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
    /// This method automatically logs all state transitions at info level with
    /// structured fields for observability and audit trail purposes.
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
        // Log state transition for observability and audit trail
        tracing::info!(
            environment_name = %self.name,
            instance_name = %self.instance_name,
            from_state = std::any::type_name::<S>(),
            to_state = std::any::type_name::<T>(),
            "Environment state transition"
        );

        Environment {
            name: self.name,
            instance_name: self.instance_name,
            profile_name: self.profile_name,
            ssh_credentials: self.ssh_credentials,
            ssh_port: self.ssh_port,
            build_dir: self.build_dir,
            data_dir: self.data_dir,
            instance_ip: self.instance_ip,
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

    /// Returns the SSH port for this environment
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.ssh_port
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

    /// Returns the instance IP address if available
    ///
    /// The instance IP is populated after successful provisioning and is
    /// `None` for environments that haven't been provisioned yet.
    ///
    /// # Returns
    ///
    /// - `Some(IpAddr)` if the environment has been provisioned
    /// - `None` if the environment hasn't been provisioned yet
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deploy::domain::{Environment, EnvironmentName};
    /// use torrust_tracker_deploy::shared::{Username, ssh::SshCredentials};
    /// use std::path::PathBuf;
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// let env_name = EnvironmentName::new("test".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/test_rsa"),
    ///     PathBuf::from("keys/test_rsa.pub"),
    ///     ssh_username,
    /// );
    /// let environment = Environment::new(env_name, ssh_credentials, 22);
    ///
    /// // Before provisioning
    /// assert_eq!(environment.instance_ip(), None);
    ///
    /// // After provisioning (simulated)
    /// let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
    /// let environment = environment.with_instance_ip(ip);
    /// assert_eq!(environment.instance_ip(), Some(ip));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn instance_ip(&self) -> Option<IpAddr> {
        self.instance_ip
    }

    /// Sets the instance IP address for this environment
    ///
    /// This method is typically called by the `ProvisionCommand` after successfully
    /// provisioning the infrastructure and obtaining the instance's IP address.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address of the provisioned instance
    ///
    /// # Returns
    ///
    /// A new Environment instance with the IP address set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deploy::domain::{Environment, EnvironmentName};
    /// use torrust_tracker_deploy::shared::{Username, ssh::SshCredentials};
    /// use std::path::PathBuf;
    /// use std::net::{IpAddr, Ipv4Addr};
    ///
    /// let env_name = EnvironmentName::new("production".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/prod_rsa"),
    ///     PathBuf::from("keys/prod_rsa.pub"),
    ///     ssh_username,
    /// );
    /// let environment = Environment::new(env_name, ssh_credentials, 22);
    ///
    /// // Set IP after provisioning
    /// let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 42));
    /// let environment = environment.with_instance_ip(ip);
    ///
    /// assert_eq!(environment.instance_ip(), Some(ip));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_instance_ip(mut self, ip: IpAddr) -> Self {
        self.instance_ip = Some(ip);
        self
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
    /// let environment = Environment::new(env_name, ssh_credentials, 22);
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

    /// Returns the traces directory for this environment
    ///
    /// The traces directory is located at `data/{env_name}/traces/`
    /// and contains error trace files for failed operations.
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
    /// let environment = Environment::new(env_name, ssh_credentials, 22);
    ///
    /// assert_eq!(
    ///     environment.traces_dir(),
    ///     PathBuf::from("data/production/traces")
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn traces_dir(&self) -> PathBuf {
        self.data_dir.join(TRACES_DIR_NAME)
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
    /// let environment = Environment::new(env_name, ssh_credentials, 22);
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
    /// let environment = Environment::new(env_name, ssh_credentials, 22);
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
    /// let environment = Environment::new(env_name, ssh_credentials, 22);
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
    /// let environment = Environment::new(env_name, ssh_credentials, 22);
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
    use std::path::Path;
    use tempfile::TempDir;

    // ============================================================================
    // Test Fixtures - Builder Pattern
    // ============================================================================

    /// Test builder for creating Environment instances with sensible defaults
    ///
    /// This builder simplifies test setup by providing default values and allowing
    /// customization through a fluent API. It automatically manages temporary
    /// directories and creates all required value objects.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Simple environment with defaults
    /// let env = EnvironmentTestBuilder::new().build();
    ///
    /// // Customized environment
    /// let env = EnvironmentTestBuilder::new()
    ///     .with_name("staging")
    ///     .with_ssh_key_name("custom_key")
    ///     .build();
    ///
    /// // Environment with access to temp directory
    /// let (env, temp_dir) = EnvironmentTestBuilder::new()
    ///     .with_name("test-env")
    ///     .build_with_temp_dir();
    /// ```
    pub struct EnvironmentTestBuilder {
        env_name: String,
        ssh_key_name: String,
        ssh_username: String,
        temp_dir: TempDir,
    }

    impl EnvironmentTestBuilder {
        /// Creates a new builder with sensible defaults
        pub fn new() -> Self {
            Self {
                env_name: "test-env".to_string(),
                ssh_key_name: "test_key".to_string(),
                ssh_username: "torrust".to_string(),
                temp_dir: TempDir::new().expect("Failed to create temp directory"),
            }
        }

        /// Sets the environment name
        pub fn with_name(mut self, name: &str) -> Self {
            self.env_name = name.to_string();
            self
        }

        /// Sets the SSH key name (without .pub extension)
        pub fn with_ssh_key_name(mut self, key_name: &str) -> Self {
            self.ssh_key_name = key_name.to_string();
            self
        }

        /// Sets the SSH username
        #[allow(dead_code)]
        pub fn with_ssh_username(mut self, username: &str) -> Self {
            self.ssh_username = username.to_string();
            self
        }

        /// Builds an Environment in Created state
        ///
        /// This is the most common use case - creates an environment with
        /// auto-generated paths based on the environment name.
        pub fn build(self) -> Environment<Created> {
            let env_name = EnvironmentName::new(self.env_name).unwrap();
            let ssh_username = Username::new(self.ssh_username).unwrap();
            let temp_path = self.temp_dir.path();

            let ssh_credentials = SshCredentials::new(
                temp_path.join(&self.ssh_key_name),
                temp_path.join(format!("{}.pub", &self.ssh_key_name)),
                ssh_username,
            );

            let ssh_port = 22;

            Environment::new(env_name, ssh_credentials, ssh_port)
        }

        /// Builds an Environment and returns the `TempDir`
        ///
        /// Use this when you need access to the temp directory in your test,
        /// for example to verify paths or create additional test files.
        #[allow(dead_code)]
        pub fn build_with_temp_dir(self) -> (Environment<Created>, TempDir) {
            let env_name = EnvironmentName::new(self.env_name).unwrap();
            let ssh_username = Username::new(self.ssh_username).unwrap();
            let temp_path = self.temp_dir.path();

            let ssh_credentials = SshCredentials::new(
                temp_path.join(&self.ssh_key_name),
                temp_path.join(format!("{}.pub", &self.ssh_key_name)),
                ssh_username,
            );

            let ssh_port = 22;
            let environment = Environment::new(env_name, ssh_credentials, ssh_port);
            (environment, self.temp_dir)
        }

        /// Builds an Environment with custom paths
        ///
        /// Use this when you need full control over the data and build directories.
        /// Returns the environment, `data_dir`, `build_dir`, and `temp_dir`.
        pub fn build_with_custom_paths(self) -> (Environment<Created>, PathBuf, PathBuf, TempDir) {
            let temp_path = self.temp_dir.path();
            let data_dir = temp_path.join("data").join(&self.env_name);
            let build_dir = temp_path.join("build").join(&self.env_name);

            let env_name = EnvironmentName::new(self.env_name).unwrap();
            let ssh_username = Username::new(self.ssh_username).unwrap();
            let ssh_credentials = SshCredentials::new(
                temp_path.join(&self.ssh_key_name),
                temp_path.join(format!("{}.pub", &self.ssh_key_name)),
                ssh_username,
            );

            let instance_name =
                InstanceName::new(format!("torrust-tracker-vm-{}", env_name.as_str())).unwrap();
            let profile_name = ProfileName::new(format!("lxd-{}", env_name.as_str())).unwrap();

            let environment = Environment {
                name: env_name,
                instance_name,
                profile_name,
                ssh_credentials,
                ssh_port: 22,
                data_dir: data_dir.clone(),
                build_dir: build_dir.clone(),
                instance_ip: None,
                state: Created,
            };

            (environment, data_dir, build_dir, self.temp_dir)
        }

        /// Returns a reference to the temp directory path
        #[allow(dead_code)]
        pub fn temp_path(&self) -> &Path {
            self.temp_dir.path()
        }
    }

    impl Default for EnvironmentTestBuilder {
        fn default() -> Self {
            Self::new()
        }
    }

    // ============================================================================
    // Custom Assertion Helpers
    // ============================================================================

    /// Asserts that the environment's paths are within the temp directory
    #[allow(dead_code)]
    fn assert_paths_in_temp_dir(env: &Environment<impl Clone>, temp_path: &Path, env_name: &str) {
        assert!(
            env.data_dir().starts_with(temp_path),
            "data_dir should be in temp: {:?} not in {:?}",
            env.data_dir(),
            temp_path
        );
        assert!(
            env.build_dir().starts_with(temp_path),
            "build_dir should be in temp: {:?} not in {:?}",
            env.build_dir(),
            temp_path
        );
        assert!(
            env.data_dir().to_string_lossy().contains(env_name),
            "data_dir should contain env name: {:?}",
            env.data_dir()
        );
        assert!(
            env.build_dir().to_string_lossy().contains(env_name),
            "build_dir should contain env name: {:?}",
            env.build_dir()
        );
    }

    /// Asserts that SSH credentials match expected paths
    fn assert_ssh_credentials(
        env: &Environment<impl Clone>,
        expected_private: &Path,
        expected_public: &Path,
    ) {
        assert_eq!(
            env.ssh_private_key_path(),
            expected_private,
            "SSH private key path mismatch"
        );
        assert_eq!(
            env.ssh_public_key_path(),
            expected_public,
            "SSH public key path mismatch"
        );
    }

    /// Asserts that the instance name matches the expected format
    fn assert_instance_name_format(env: &Environment<impl Clone>, env_name: &str) {
        let expected = format!("torrust-tracker-vm-{env_name}");
        assert_eq!(
            env.instance_name().as_str(),
            expected,
            "Instance name should match expected format"
        );
    }

    /// Asserts that a path ends with the expected suffix
    #[allow(dead_code)]
    fn assert_path_ends_with(actual: &Path, expected_suffix: &str) {
        assert!(
            actual.to_string_lossy().ends_with(expected_suffix),
            "Path {actual:?} should end with {expected_suffix:?}"
        );
    }

    // ============================================================================
    // Tests
    // ============================================================================

    #[test]
    fn it_should_create_environment_with_auto_generated_paths() {
        // Arrange
        let (environment, data_dir, build_dir, temp_dir) = EnvironmentTestBuilder::new()
            .with_name("test-env")
            .with_ssh_key_name("testing_rsa")
            .build_with_custom_paths();
        let temp_path = temp_dir.path();

        // Act & Assert: Check basic fields
        assert_eq!(environment.name().as_str(), "test-env");
        assert_eq!(environment.ssh_username().as_str(), "torrust");

        // Assert: Check SSH credentials
        assert_ssh_credentials(
            &environment,
            &temp_path.join("testing_rsa"),
            &temp_path.join("testing_rsa.pub"),
        );

        // Assert: Check paths are in temp directory
        assert_eq!(*environment.data_dir(), data_dir);
        assert_eq!(*environment.build_dir(), build_dir);

        // Assert: Check instance name format
        assert_instance_name_format(&environment, "test-env");
    }

    #[test]
    fn it_should_generate_correct_template_directories() {
        // Arrange
        let (environment, data_dir, _build_dir, _temp_dir) = EnvironmentTestBuilder::new()
            .with_name("test-production")
            .with_ssh_key_name("prod_rsa")
            .build_with_custom_paths();

        // Act
        let templates_dir = environment.templates_dir();
        let ansible_dir = environment.ansible_templates_dir();
        let tofu_dir = environment.tofu_templates_dir();

        // Assert
        assert_eq!(templates_dir, data_dir.join("templates"));
        assert_eq!(ansible_dir, data_dir.join("templates").join("ansible"));
        assert_eq!(tofu_dir, data_dir.join("templates").join("tofu"));
    }

    #[test]
    fn it_should_generate_correct_build_directories() {
        // Arrange
        let (environment, _data_dir, build_dir, _temp_dir) = EnvironmentTestBuilder::new()
            .with_name("test-staging")
            .with_ssh_key_name("staging_rsa")
            .build_with_custom_paths();

        // Act
        let ansible_dir = environment.ansible_build_dir();
        let tofu_dir = environment.tofu_build_dir();

        // Assert
        assert_eq!(ansible_dir, build_dir.join("ansible"));
        assert_eq!(tofu_dir, build_dir.join("tofu"));
    }

    #[test]
    fn it_should_handle_different_environment_names() {
        // Arrange: Test cases with environment names and expected instance names
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
            // Arrange
            let (environment, data_dir, build_dir, _temp_dir) = EnvironmentTestBuilder::new()
                .with_name(env_name_str)
                .build_with_custom_paths();

            // Act & Assert
            assert_eq!(environment.instance_name().as_str(), expected_instance_name);
            assert_eq!(*environment.data_dir(), data_dir);
            assert_eq!(*environment.build_dir(), build_dir);
        }
    }

    #[test]
    fn it_should_be_serializable_to_json() {
        // Arrange
        let (environment, data_dir, build_dir, temp_dir) = EnvironmentTestBuilder::new()
            .with_name("test-serialization")
            .with_ssh_key_name("test_private_key")
            .build_with_custom_paths();
        let temp_path = temp_dir.path();

        // Act: Serialize to JSON
        let json = serde_json::to_string(&environment).unwrap();

        // Act: Deserialize back
        let deserialized: Environment = serde_json::from_str(&json).unwrap();

        // Assert: Check that all fields are preserved
        assert_eq!(deserialized.name().as_str(), "test-serialization");
        assert_instance_name_format(&deserialized, "test-serialization");
        assert_ssh_credentials(
            &deserialized,
            &temp_path.join("test_private_key"),
            &temp_path.join("test_private_key.pub"),
        );
        assert_eq!(*deserialized.data_dir(), data_dir);
        assert_eq!(*deserialized.build_dir(), build_dir);
    }

    #[test]
    fn it_should_support_common_e2e_environment_names() {
        // Arrange: Common E2E environment names
        let e2e_environments = vec!["test-e2e-config", "test-e2e-provision", "test-e2e-full"];

        for env_name_str in e2e_environments {
            // Arrange
            let environment = EnvironmentTestBuilder::new()
                .with_name(env_name_str)
                .with_ssh_key_name("testing_rsa")
                .build();

            // Act & Assert: Verify the environment is created successfully
            assert_eq!(environment.name().as_str(), env_name_str);
            assert!(environment
                .instance_name()
                .as_str()
                .starts_with("torrust-tracker-vm-"));
            assert!(environment
                .data_dir()
                .to_string_lossy()
                .contains(env_name_str));
            assert!(environment
                .build_dir()
                .to_string_lossy()
                .contains(env_name_str));
        }
    }

    #[test]
    fn it_should_handle_dash_separated_environment_names() {
        // Arrange
        let (environment, data_dir, build_dir, _temp_dir) = EnvironmentTestBuilder::new()
            .with_name("test-feature-user-auth")
            .with_ssh_key_name("feature_rsa")
            .build_with_custom_paths();

        // Act & Assert
        assert_instance_name_format(&environment, "test-feature-user-auth");
        assert_eq!(*environment.data_dir(), data_dir);
        assert_eq!(*environment.build_dir(), build_dir);
        assert_eq!(environment.templates_dir(), data_dir.join("templates"));
    }

    // State transition tests
    mod state_transitions {
        use super::*;

        /// Helper function to create a test environment for state transition tests
        fn create_test_environment() -> Environment<Created> {
            EnvironmentTestBuilder::new()
                .with_name("test-state")
                .build()
        }

        #[test]
        fn it_should_transition_to_destroyed_from_created() {
            // Arrange
            let env = create_test_environment();

            // Act
            let env = env.destroy();

            // Assert
            assert_eq!(*env.state(), Destroyed);
            assert_eq!(env.name().as_str(), "test-state");
        }

        #[test]
        fn it_should_complete_full_happy_path_transition() {
            // Arrange
            let env = create_test_environment();

            // Act: Complete happy path: Created -> Running
            let env = env
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured()
                .start_releasing()
                .released()
                .start_running();

            // Assert
            assert_eq!(*env.state(), Running);
            assert_eq!(env.name().as_str(), "test-state");

            // Act: Then destroy
            let env = env.destroy();

            // Assert
            assert_eq!(*env.state(), Destroyed);
        }

        #[test]
        fn it_should_preserve_all_fields_during_transitions() {
            // Arrange
            let env = create_test_environment();
            let initial_name = env.name().clone();
            let initial_instance_name = env.instance_name().clone();
            let initial_data_dir = env.data_dir().clone();
            let initial_build_dir = env.build_dir().clone();

            // Act: Go through several transitions
            let env = env
                .start_provisioning()
                .provisioned()
                .start_configuring()
                .configured();

            // Assert: Verify all fields are preserved
            assert_eq!(env.name(), &initial_name);
            assert_eq!(env.instance_name(), &initial_instance_name);
            assert_eq!(env.data_dir(), &initial_data_dir);
            assert_eq!(env.build_dir(), &initial_build_dir);
        }

        // State transition logging tests
        mod logging {
            use super::*;
            use tracing_test::traced_test;

            #[traced_test]
            #[test]
            fn it_should_log_state_transition_from_created_to_provisioning() {
                let env = create_test_environment();

                let _provisioning = env.start_provisioning();

                // Assert log contains expected fields
                assert!(logs_contain("Environment state transition"));
                assert!(logs_contain("environment_name=test-state"));
                assert!(logs_contain("from_state="));
                assert!(logs_contain("Created"));
                assert!(logs_contain("to_state="));
                assert!(logs_contain("Provisioning"));
            }

            #[traced_test]
            #[test]
            fn it_should_log_state_transition_with_instance_name() {
                let env = create_test_environment();

                let _provisioning = env.start_provisioning();

                assert!(logs_contain("instance_name=torrust-tracker-vm-test-state"));
            }

            #[traced_test]
            #[test]
            fn it_should_log_complete_state_transition_chain() {
                let env = create_test_environment();

                let _env = env
                    .start_provisioning()
                    .provisioned()
                    .start_configuring()
                    .configured();

                // Verify multiple transitions were logged
                assert!(logs_contain("Provisioning"));
                assert!(logs_contain("Provisioned"));
                assert!(logs_contain("Configuring"));
                assert!(logs_contain("Configured"));
            }

            #[traced_test]
            #[test]
            fn it_should_log_destroy_transition_from_any_state() {
                let env = create_test_environment();
                let env = env.start_provisioning();

                let _destroyed = env.destroy();

                assert!(logs_contain("Destroyed"));
            }
        }
    }
}
