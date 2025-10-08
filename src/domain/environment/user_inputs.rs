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
/// use torrust_tracker_deploy::domain::{InstanceName, ProfileName, EnvironmentName};
/// use torrust_tracker_deploy::domain::environment::user_inputs::UserInputs;
/// use torrust_tracker_deploy::shared::{Username, ssh::SshCredentials};
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
