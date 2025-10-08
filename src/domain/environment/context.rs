//! Environment Context Module
//!
//! This module contains the `EnvironmentContext` struct which holds all
//! state-independent environment data.
//!
//! ## Purpose
//!
//! The `EnvironmentContext` separates immutable environment configuration from
//! the mutable state machine, enabling:
//! - Reduced pattern matching in `AnyEnvironmentState`
//! - Clear distinction between constant and changing data
//! - Simplified state transitions
//! - Easier extension of environment configuration
//!
//! ## Design
//!
//! By extracting state-independent fields into a dedicated context type,
//! we eliminate repetitive 13-arm pattern matching across multiple accessor
//! methods, reducing code duplication by approximately 83%.

use crate::domain::{InstanceName, ProfileName};
use crate::shared::ssh::SshCredentials;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::path::PathBuf;

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
///
/// # Examples
///
/// `EnvironmentContext` is typically created internally by `Environment::new()`:
///
/// ```rust
/// use torrust_tracker_deploy::domain::environment::{Environment, EnvironmentName};
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
    /// The validated environment name
    pub(crate) name: crate::domain::environment::EnvironmentName,

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

impl EnvironmentContext {
    /// Returns the SSH username for this environment
    #[must_use]
    pub fn ssh_username(&self) -> &crate::shared::Username {
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

    /// Returns the templates directory for this environment
    ///
    /// Path: `data/{env_name}/templates/`
    #[must_use]
    pub fn templates_dir(&self) -> PathBuf {
        self.data_dir.join("templates")
    }

    /// Returns the traces directory for this environment
    ///
    /// Path: `data/{env_name}/traces/`
    #[must_use]
    pub fn traces_dir(&self) -> PathBuf {
        self.data_dir.join(super::TRACES_DIR_NAME)
    }

    /// Returns the ansible build directory
    ///
    /// Path: `build/{env_name}/ansible`
    #[must_use]
    pub fn ansible_build_dir(&self) -> PathBuf {
        self.build_dir.join("ansible")
    }

    /// Returns the tofu build directory
    ///
    /// Path: `build/{env_name}/tofu`
    #[must_use]
    pub fn tofu_build_dir(&self) -> PathBuf {
        self.build_dir.join("tofu")
    }

    /// Returns the ansible templates directory
    ///
    /// Path: `data/{env_name}/templates/ansible`
    #[must_use]
    pub fn ansible_templates_dir(&self) -> PathBuf {
        self.templates_dir().join("ansible")
    }

    /// Returns the tofu templates directory
    ///
    /// Path: `data/{env_name}/templates/tofu`
    #[must_use]
    pub fn tofu_templates_dir(&self) -> PathBuf {
        self.templates_dir().join("tofu")
    }
}
