//! User Inputs Module
//!
//! This module contains the `UserInputs` struct which holds all user-provided
//! configuration when creating an environment.
//!
//! ## Purpose
//!
//! User inputs represent the immutable configuration choices made by the user
//! when creating an environment. These fields never change throughout the
//! environment's lifecycle.
//!
//! ## Semantic Category
//!
//! **User Inputs** are:
//! - Provided by the user when creating an environment
//! - Immutable throughout environment lifecycle
//! - Examples: name, SSH credentials, port numbers
//!
//! Add new fields here when: User needs to configure something at environment creation time.

use crate::domain::environment::EnvironmentName;
use crate::domain::{InstanceName, ProfileName};
use crate::shared::ssh::SshCredentials;
use serde::{Deserialize, Serialize};

/// User-provided configuration when creating an environment
///
/// This struct contains all fields that are provided by the user when creating
/// an environment. These fields are immutable throughout the environment lifecycle
/// and represent the user's configuration choices.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer::domain::{InstanceName, ProfileName, EnvironmentName};
/// use torrust_tracker_deployer::domain::environment::user_inputs::UserInputs;
/// use torrust_tracker_deployer::shared::{Username, ssh::SshCredentials};
/// use std::path::PathBuf;
///
/// let user_inputs = UserInputs {
///     name: EnvironmentName::new("production".to_string())?,
///     instance_name: InstanceName::new("torrust-tracker-vm-production".to_string())?,
///     profile_name: ProfileName::new("torrust-profile-production".to_string())?,
///     ssh_credentials: SshCredentials::new(
///         PathBuf::from("keys/prod_rsa"),
///         PathBuf::from("keys/prod_rsa.pub"),
///         Username::new("torrust".to_string())?,
///     ),
///     ssh_port: 22,
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInputs {
    /// The validated environment name
    pub name: crate::domain::environment::EnvironmentName,

    /// The instance name for this environment (auto-generated from name)
    pub instance_name: InstanceName,

    /// The profile name for this environment (auto-generated from name)
    pub profile_name: ProfileName,

    /// SSH credentials for connecting to instances in this environment
    pub ssh_credentials: SshCredentials,

    /// SSH port for connecting to instances in this environment
    pub ssh_port: u16,
}

impl UserInputs {
    /// Creates a new `UserInputs` with auto-generated instance and profile names
    ///
    /// # Arguments
    ///
    /// * `name` - The validated environment name
    /// * `ssh_credentials` - SSH credentials for connecting to instances
    /// * `ssh_port` - SSH port for connecting to instances
    ///
    /// # Returns
    ///
    /// A new `UserInputs` with:
    /// - Auto-generated instance name: `torrust-tracker-vm-{env_name}`
    /// - Auto-generated profile name: `torrust-profile-{env_name}`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer::domain::environment::{EnvironmentName, UserInputs};
    /// use torrust_tracker_deployer::shared::{Username, ssh::SshCredentials};
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
    /// let user_inputs = UserInputs::new(&env_name, ssh_credentials, 22);
    ///
    /// assert_eq!(user_inputs.instance_name.as_str(), "torrust-tracker-vm-production");
    /// assert_eq!(user_inputs.profile_name.as_str(), "torrust-profile-production");
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
        let instance_name = Self::generate_instance_name(name);
        let profile_name = Self::generate_profile_name(name);

        Self {
            name: name.clone(),
            instance_name,
            profile_name,
            ssh_credentials,
            ssh_port,
        }
    }

    // ========================================================================
    // Private Helper Methods
    // ========================================================================

    /// Generates an instance name from the environment name
    ///
    /// Format: `torrust-tracker-vm-{env_name}`
    ///
    /// # Panics
    ///
    /// This function does not panic. The generated instance name is guaranteed
    /// to be valid for any valid environment name.
    fn generate_instance_name(env_name: &EnvironmentName) -> InstanceName {
        let instance_name_str = format!("torrust-tracker-vm-{}", env_name.as_str());
        InstanceName::new(instance_name_str)
            .expect("Generated instance name should always be valid")
    }

    /// Generates a profile name from the environment name
    ///
    /// Format: `torrust-profile-{env_name}`
    ///
    /// # Panics
    ///
    /// This function does not panic. The generated profile name is guaranteed
    /// to be valid for any valid environment name.
    fn generate_profile_name(env_name: &EnvironmentName) -> ProfileName {
        let profile_name_str = format!("torrust-profile-{}", env_name.as_str());
        ProfileName::new(profile_name_str).expect("Generated profile name should always be valid")
    }
}
