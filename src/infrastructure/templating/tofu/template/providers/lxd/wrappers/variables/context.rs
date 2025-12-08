//! # `OpenTofu` Variables Context
//!
//! Provides context structures for `OpenTofu` variables template rendering.
//!
//! This module contains the context object that holds runtime values for variable template rendering,
//! specifically for the `variables.tfvars.tera` template used in LXD infrastructure provisioning.
//!
//! ## Context Structure
//!
//! The `VariablesContext` holds:
//! - `instance_name` - The dynamic name for the VM/container instance
//!
//! ## Example Usage
//!
//! ```rust
//! use torrust_tracker_deployer_lib::infrastructure::templating::tofu::template::providers::lxd::wrappers::variables::VariablesContext;
//! use torrust_tracker_deployer_lib::adapters::lxd::instance::InstanceName;
//! use torrust_tracker_deployer_lib::domain::ProfileName;
//!
//! let context = VariablesContext::builder()
//!     .with_instance_name(InstanceName::new("my-test-vm".to_string()).unwrap())
//!     .with_profile_name(ProfileName::new("my-test-profile".to_string()).unwrap())
//!     .build()
//!     .unwrap();
//! ```

use serde::Serialize;
use thiserror::Error;

use crate::domain::{InstanceName, ProfileName};

/// Errors that can occur when building the variables context
#[derive(Error, Debug)]
pub enum VariablesContextError {
    /// Instance name is required but was not provided
    #[error("Instance name is required but was not provided")]
    MissingInstanceName,

    /// Profile name is required but was not provided
    #[error("Profile name is required but was not provided")]
    MissingProfileName,
}

/// Context for `OpenTofu` variables template rendering
///
/// Contains all runtime values needed to render `variables.tfvars.tera`
/// with dynamic instance naming and other configurable parameters.
#[derive(Debug, Clone, Serialize)]
pub struct VariablesContext {
    /// The name of the VM/container instance to be created
    pub instance_name: InstanceName,
    /// The name of the LXD profile to be created  
    pub profile_name: ProfileName,
}

/// Builder for creating `VariablesContext` instances
///
/// Provides a fluent interface for constructing the context with validation
/// to ensure all required fields are provided.
#[derive(Debug, Default)]
pub struct VariablesContextBuilder {
    instance_name: Option<InstanceName>,
    profile_name: Option<ProfileName>,
}

impl VariablesContextBuilder {
    /// Creates a new builder instance
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the instance name for the VM/container
    ///
    /// # Arguments
    ///
    /// * `instance_name` - The name to assign to the created instance
    #[must_use]
    pub fn with_instance_name(mut self, instance_name: InstanceName) -> Self {
        self.instance_name = Some(instance_name);
        self
    }

    /// Sets the profile name for the LXD profile
    ///
    /// # Arguments
    ///
    /// * `profile_name` - The name to assign to the LXD profile
    #[must_use]
    pub fn with_profile_name(mut self, profile_name: ProfileName) -> Self {
        self.profile_name = Some(profile_name);
        self
    }

    /// Builds the `VariablesContext` with validation
    ///
    /// # Returns
    ///
    /// * `Ok(VariablesContext)` if all required fields are present
    /// * `Err(VariablesContextError)` if validation fails
    ///
    /// # Errors
    ///
    /// Returns `MissingInstanceName` if instance name was not provided
    /// Returns `MissingProfileName` if profile name was not provided
    pub fn build(self) -> Result<VariablesContext, VariablesContextError> {
        let instance_name = self
            .instance_name
            .ok_or(VariablesContextError::MissingInstanceName)?;

        let profile_name = self
            .profile_name
            .ok_or(VariablesContextError::MissingProfileName)?;

        Ok(VariablesContext {
            instance_name,
            profile_name,
        })
    }
}

impl VariablesContext {
    /// Creates a new builder for constructing `VariablesContext`
    #[must_use]
    pub fn builder() -> VariablesContextBuilder {
        VariablesContextBuilder::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_variables_context_with_instance_name() {
        let context = VariablesContext::builder()
            .with_instance_name(InstanceName::new("test-vm".to_string()).unwrap())
            .with_profile_name(ProfileName::new("test-profile".to_string()).unwrap())
            .build()
            .unwrap();

        assert_eq!(context.instance_name.as_str(), "test-vm");
        assert_eq!(context.profile_name.as_str(), "test-profile");
    }

    #[test]
    fn it_should_serialize_to_json() {
        let context = VariablesContext::builder()
            .with_instance_name(InstanceName::new("test-vm".to_string()).unwrap())
            .with_profile_name(ProfileName::new("test-profile".to_string()).unwrap())
            .build()
            .unwrap();

        let json = serde_json::to_string(&context).unwrap();
        assert!(json.contains("test-vm"));
        assert!(json.contains("instance_name"));
        assert!(json.contains("test-profile"));
        assert!(json.contains("profile_name"));
    }

    #[test]
    fn it_should_build_context_with_builder_pattern() {
        let result = VariablesContext::builder()
            .with_instance_name(InstanceName::new("my-instance".to_string()).unwrap())
            .with_profile_name(ProfileName::new("my-profile".to_string()).unwrap())
            .build();

        assert!(result.is_ok());
        let context = result.unwrap();
        assert_eq!(context.instance_name.as_str(), "my-instance");
        assert_eq!(context.profile_name.as_str(), "my-profile");
    }

    #[test]
    fn it_should_fail_when_instance_name_is_missing() {
        let result = VariablesContext::builder()
            .with_profile_name(ProfileName::new("test-profile".to_string()).unwrap())
            .build();

        assert!(matches!(
            result.unwrap_err(),
            VariablesContextError::MissingInstanceName
        ));
    }

    #[test]
    fn it_should_fail_when_profile_name_is_missing() {
        let result = VariablesContext::builder()
            .with_instance_name(InstanceName::new("test-vm".to_string()).unwrap())
            .build();

        assert!(matches!(
            result.unwrap_err(),
            VariablesContextError::MissingProfileName
        ));
    }
}
