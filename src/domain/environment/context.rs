//! Environment Context Module
//!
//! This module contains the `EnvironmentContext` struct which composes three
//! semantic types to organize state-independent environment data.
//!
//! ## Purpose
//!
//! The `EnvironmentContext` separates immutable environment configuration from
//! the mutable state machine, and further organizes that configuration into
//! three distinct semantic categories:
//!
//! 1. **User Inputs** - Configuration provided by users
//! 2. **Internal Config** - Derived paths for organizing artifacts
//! 3. **Runtime Outputs** - Data generated during deployment
//!
//! ## Benefits
//!
//! - **Reduced pattern matching**: Access common fields without matching on state (83% reduction)
//! - **Clear semantic boundaries**: Types document the purpose of each field
//! - **Developer guidance**: Clear where to add new fields based on their purpose
//! - **Simplified state transitions**: Only the state changes, context remains constant
//! - **Easier extension**: Adding fields is straightforward with clear categorization
//!
//! ## Three-Way Semantic Split
//!
//! ### When to Add Fields
//!
//! - **`UserInputs`**: User needs to configure something at environment creation time
//! - **`InternalConfig`**: Need internal paths or derived configuration
//! - **`RuntimeOutputs`**: Operations produce new data about deployed infrastructure
//!
//! ### Design Rationale
//!
//! By organizing fields into three semantic categories, we make it immediately
//! clear where each piece of information comes from and guide developers on
//! where to add new fields as the application evolves.

use crate::adapters::ssh::SshCredentials;
use crate::domain::environment::{EnvironmentName, InternalConfig, RuntimeOutputs, UserInputs};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Complete environment context composed of three semantic types
///
/// The context is split into three logical categories:
/// 1. **User Inputs** (`user_inputs`): Configuration provided by users
/// 2. **Internal Config** (`internal_config`): Derived paths for organizing artifacts
/// 3. **Runtime Outputs** (`runtime_outputs`): Data generated during deployment
///
/// This separation makes it clear where each piece of information comes from
/// and helps developers understand where to add new fields.
///
/// # Design Rationale
///
/// By separating state-independent data from the state machine and organizing
/// it into three semantic categories, we:
/// - Eliminate repetitive pattern matching in `AnyEnvironmentState`
/// - Make it clear which data is constant vs. state-dependent
/// - Provide semantic clarity about the purpose of each field
/// - Guide developers where to add new fields based on their purpose
/// - Simplify state transitions (only the state field changes)
/// - Enable easier extension of environment configuration
///
/// # Three Semantic Categories
///
/// - **User Inputs**: Immutable user configuration (name, SSH credentials, port)
/// - **Internal Config**: Derived paths (`build_dir`, `data_dir`)
/// - **Runtime Outputs**: Generated during deployment (`instance_ip`, future metrics)
///
/// # Examples
///
/// `EnvironmentContext` is typically created internally by `Environment::new()`:
///
/// ```rust
/// use torrust_tracker_deployer_lib::domain::environment::{Environment, EnvironmentName};
/// use torrust_tracker_deployer_lib::shared::Username;
/// use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
/// use std::path::PathBuf;
///
/// let env_name = EnvironmentName::new("production".to_string())?;
/// let ssh_username = Username::new("torrust".to_string())?;
/// let ssh_credentials = SshCredentials::new(
///     PathBuf::from("keys/prod_rsa"),
///     PathBuf::from("keys/prod_rsa.pub"),
///     ssh_username,
/// );
///
/// // Environment::new() creates the EnvironmentContext internally
/// let environment = Environment::new(env_name, ssh_credentials, 22);
///
/// // Access the context through the environment
/// let context = environment.context();
/// // Context holds all state-independent data for the environment
///
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    /// User-provided configuration
    pub user_inputs: UserInputs,

    /// Internal paths and derived configuration
    pub internal_config: InternalConfig,

    /// Runtime outputs from deployment operations
    pub runtime_outputs: RuntimeOutputs,
}

impl EnvironmentContext {
    /// Creates a new `EnvironmentContext` with auto-generated names and paths
    ///
    /// # Arguments
    ///
    /// * `name` - The validated environment name
    /// * `ssh_credentials` - SSH credentials for connecting to instances
    /// * `ssh_port` - SSH port for connecting to instances
    ///
    /// # Returns
    ///
    /// A new `EnvironmentContext` with:
    /// - Auto-generated instance name: `torrust-tracker-vm-{env_name}`
    /// - Auto-generated profile name: `torrust-profile-{env_name}`
    /// - Auto-generated data and build directories
    /// - Empty runtime outputs
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::environment::{EnvironmentContext, EnvironmentName};
    /// use torrust_tracker_deployer_lib::shared::Username;
    /// use torrust_tracker_deployer_lib::adapters::ssh::SshCredentials;
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("production".to_string())?;
    /// let ssh_username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/prod_rsa"),
    ///     PathBuf::from("keys/prod_rsa.pub"),
    ///     ssh_username,
    /// );
    ///
    /// let context = EnvironmentContext::new(&env_name, ssh_credentials, 22);
    ///
    /// assert_eq!(context.user_inputs.instance_name.as_str(), "torrust-tracker-vm-production");
    /// assert_eq!(context.user_inputs.profile_name.as_str(), "torrust-profile-production");
    /// assert_eq!(context.internal_config.data_dir, PathBuf::from("./data/production"));
    /// assert_eq!(context.internal_config.build_dir, PathBuf::from("./build/production"));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Panics
    ///
    /// This function does not panic. All name generation is guaranteed to succeed
    /// for valid environment names.
    #[must_use]
    pub fn new(name: &EnvironmentName, ssh_credentials: SshCredentials, ssh_port: u16) -> Self {
        Self {
            user_inputs: UserInputs::new(name, ssh_credentials, ssh_port),
            internal_config: InternalConfig::new(name),
            runtime_outputs: RuntimeOutputs { instance_ip: None },
        }
    }

    /// Creates a new environment context with directories relative to a working directory
    ///
    /// This version creates absolute paths for data and build directories by
    /// using the provided working directory as the base.
    ///
    /// # Arguments
    ///
    /// * `name` - The environment name
    /// * `ssh_credentials` - SSH credentials for accessing the instance
    /// * `ssh_port` - SSH port (typically 22)
    /// * `working_dir` - The base working directory for operations
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::domain::environment::{EnvironmentContext, EnvironmentName};
    /// use torrust_tracker_deployer_lib::adapters::SshCredentials;
    /// use torrust_tracker_deployer_lib::shared::Username;
    /// use std::path::PathBuf;
    ///
    /// let env_name = EnvironmentName::new("production".to_string())?;
    /// let username = Username::new("torrust".to_string())?;
    /// let ssh_credentials = SshCredentials::new(
    ///     PathBuf::from("keys/prod_rsa"),
    ///     PathBuf::from("keys/prod_rsa.pub"),
    ///     username,
    /// );
    /// let working_dir = PathBuf::from("/opt/deployments");
    ///
    /// let context = EnvironmentContext::with_working_dir(&env_name, ssh_credentials, 22, &working_dir);
    ///
    /// assert_eq!(context.user_inputs.instance_name.as_str(), "torrust-tracker-vm-production");
    /// assert_eq!(context.internal_config.data_dir, PathBuf::from("/opt/deployments/data/production"));
    /// assert_eq!(context.internal_config.build_dir, PathBuf::from("/opt/deployments/build/production"));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn with_working_dir(
        name: &EnvironmentName,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
        working_dir: &std::path::Path,
    ) -> Self {
        Self {
            user_inputs: UserInputs::new(name, ssh_credentials, ssh_port),
            internal_config: InternalConfig::with_working_dir(name, working_dir),
            runtime_outputs: RuntimeOutputs { instance_ip: None },
        }
    }

    /// Returns the SSH username for this environment
    #[must_use]
    pub fn ssh_username(&self) -> &crate::shared::Username {
        &self.user_inputs.ssh_credentials.ssh_username
    }

    /// Returns the SSH private key path for this environment
    #[must_use]
    pub fn ssh_private_key_path(&self) -> &PathBuf {
        &self.user_inputs.ssh_credentials.ssh_priv_key_path
    }

    /// Returns the SSH public key path for this environment
    #[must_use]
    pub fn ssh_public_key_path(&self) -> &PathBuf {
        &self.user_inputs.ssh_credentials.ssh_pub_key_path
    }

    /// Returns the templates directory for this environment
    ///
    /// Path: `data/{env_name}/templates/`
    #[must_use]
    pub fn templates_dir(&self) -> PathBuf {
        self.internal_config.templates_dir()
    }

    /// Returns the traces directory for this environment
    ///
    /// Path: `data/{env_name}/traces/`
    #[must_use]
    pub fn traces_dir(&self) -> PathBuf {
        self.internal_config.traces_dir()
    }

    /// Returns the ansible build directory
    ///
    /// Path: `build/{env_name}/ansible`
    #[must_use]
    pub fn ansible_build_dir(&self) -> PathBuf {
        self.internal_config.ansible_build_dir()
    }

    /// Returns the tofu build directory
    ///
    /// Path: `build/{env_name}/tofu`
    #[must_use]
    pub fn tofu_build_dir(&self) -> PathBuf {
        self.internal_config.tofu_build_dir()
    }

    /// Returns the ansible templates directory
    ///
    /// Path: `data/{env_name}/templates/ansible`
    #[must_use]
    pub fn ansible_templates_dir(&self) -> PathBuf {
        self.internal_config.ansible_templates_dir()
    }

    /// Returns the tofu templates directory
    ///
    /// Path: `data/{env_name}/templates/tofu`
    #[must_use]
    pub fn tofu_templates_dir(&self) -> PathBuf {
        self.internal_config.tofu_templates_dir()
    }
}
