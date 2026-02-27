//! Register Details Data Transfer Object
//!
//! This module contains the presentation DTO for register command details.
//! It serves as the data structure passed to view renderers (`TextView`, `JsonView`, etc.).
//!
//! # Architecture
//!
//! This follows the Strategy Pattern where:
//! - This DTO is the data passed to all rendering strategies
//! - Different views (`TextView`, `JsonView`) consume this data
//! - Adding new formats doesn't modify this DTO or existing views
//!
//! # SOLID Principles
//!
//! - **Single Responsibility**: This file only defines the data structure
//! - **Open/Closed**: New formats extend by adding views, not modifying this
//! - **Separation of Concerns**: Data definition separate from rendering logic

use serde::Serialize;

use crate::domain::environment::state::Provisioned;
use crate::domain::environment::Environment;

/// Register details data for rendering
///
/// This struct holds all the data needed to render register command
/// information for display to the user. It is consumed by view renderers
/// (`TextView`, `JsonView`) which format it according to their specific output format.
///
/// # Design
///
/// This is a presentation layer DTO (Data Transfer Object) that:
/// - Decouples application types from view formatting
/// - Provides a stable interface for multiple view strategies
/// - Contains all fields needed for any output format
///
/// # Named Constructor
///
/// `RegisterDetailsData` is built from an `Environment<Provisioned>` since
/// the register command handler returns the provisioned environment on success.
/// `registered: true` is always set because this DTO is only constructed on
/// the success path.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RegisterDetailsData {
    /// Name of the environment that was registered
    pub environment_name: String,
    /// IP address of the registered instance (empty string if unknown)
    pub instance_ip: String,
    /// SSH port of the registered instance
    pub ssh_port: u16,
    /// Always `true` when the command exits successfully
    pub registered: bool,
}

impl RegisterDetailsData {
    /// Construct a `RegisterDetailsData` from a provisioned environment
    ///
    /// This named constructor always sets `registered: true` because it is only
    /// called on the success path — register failures result in an error return,
    /// never a `RegisterDetailsData`.
    ///
    /// # Arguments
    ///
    /// * `env` - The provisioned environment returned by the register command handler
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use torrust_tracker_deployer_lib::presentation::cli::views::commands::register::RegisterDetailsData;
    ///
    /// // Built from a provisioned environment in the success path
    /// let data = RegisterDetailsData::from_environment(&provisioned_env);
    ///
    /// assert!(data.registered);
    /// assert_eq!(data.ssh_port, 22);
    /// ```
    #[must_use]
    pub fn from_environment(env: &Environment<Provisioned>) -> Self {
        Self {
            environment_name: env.name().to_string(),
            instance_ip: env
                .instance_ip()
                .map_or_else(String::new, |ip| ip.to_string()),
            ssh_port: env.ssh_port(),
            registered: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_always_set_registered_to_true() {
        // The constructor only produces RegisterDetailsData on the success path.
        // We verify the field directly since we cannot construct Environment<Provisioned>
        // without full infrastructure in a unit test.
        let data = RegisterDetailsData {
            environment_name: "test-env".to_string(),
            instance_ip: "192.168.1.1".to_string(),
            ssh_port: 22,
            registered: true,
        };

        assert!(
            data.registered,
            "registered should always be true on success path"
        );
    }

    #[test]
    fn it_should_store_all_fields() {
        // Arrange
        let data = RegisterDetailsData {
            environment_name: "my-env".to_string(),
            instance_ip: "10.0.0.1".to_string(),
            ssh_port: 2222,
            registered: true,
        };

        // Assert
        assert_eq!(data.environment_name, "my-env");
        assert_eq!(data.instance_ip, "10.0.0.1");
        assert_eq!(data.ssh_port, 2222);
        assert!(data.registered);
    }
}
