//! # Hetzner Cloud `OpenTofu` Variables Context
//!
//! Provides context structures for Hetzner Cloud `OpenTofu` variables template rendering.
//!
//! This module contains the context object that holds runtime values for variable template rendering,
//! specifically for the `variables.tfvars.tera` template used in Hetzner Cloud infrastructure provisioning.
//!
//! ## Context Structure
//!
//! The `VariablesContext` holds:
//! - `instance_name` - The dynamic name for the server instance
//! - `hcloud_api_token` - Hetzner Cloud API token for authentication
//! - `server_type` - Hetzner server type (e.g., cx22, cx32)
//! - `server_location` - Datacenter location (e.g., nbg1, fsn1)
//! - `server_image` - OS image (e.g., ubuntu-24.04)
//! - `ssh_public_key_content` - SSH public key content for server access
//!
//! ## Example Usage
//!
//! ```rust
//! use torrust_tracker_deployer_lib::infrastructure::external_tools::tofu::template::providers::hetzner::wrappers::variables::VariablesContext;
//! use torrust_tracker_deployer_lib::adapters::lxd::instance::InstanceName;
//!
//! let context = VariablesContext::builder()
//!     .with_instance_name(InstanceName::new("my-test-vm".to_string()).unwrap())
//!     .with_hcloud_api_token("my-api-token".to_string())
//!     .with_server_type("cx22".to_string())
//!     .with_server_location("nbg1".to_string())
//!     .with_server_image("ubuntu-24.04".to_string())
//!     .with_ssh_public_key_content("ssh-rsa AAAA...".to_string())
//!     .build()
//!     .unwrap();
//! ```

use serde::Serialize;
use thiserror::Error;

use crate::domain::InstanceName;

/// Errors that can occur when building the Hetzner variables context
#[derive(Error, Debug)]
pub enum VariablesContextError {
    /// Instance name is required but was not provided
    #[error("Instance name is required but was not provided")]
    MissingInstanceName,

    /// Hetzner Cloud API token is required but was not provided
    #[error("Hetzner Cloud API token is required but was not provided")]
    MissingHcloudApiToken,

    /// Server type is required but was not provided
    #[error("Server type is required but was not provided")]
    MissingServerType,

    /// Server location is required but was not provided
    #[error("Server location is required but was not provided")]
    MissingServerLocation,

    /// Server image is required but was not provided
    #[error("Server image is required but was not provided")]
    MissingServerImage,

    /// SSH public key content is required but was not provided
    #[error("SSH public key content is required but was not provided")]
    MissingSshPublicKeyContent,
}

/// Context for Hetzner Cloud `OpenTofu` variables template rendering
///
/// Contains all runtime values needed to render `variables.tfvars.tera`
/// with Hetzner Cloud-specific configuration parameters.
#[derive(Debug, Clone, Serialize)]
pub struct VariablesContext {
    /// The name of the server instance to be created
    pub instance_name: InstanceName,
    /// Hetzner Cloud API token for authentication (sensitive)
    pub hcloud_api_token: String,
    /// Hetzner server type (e.g., cx22, cx32, cpx11)
    pub server_type: String,
    /// Datacenter location (e.g., nbg1, fsn1, hel1)
    pub server_location: String,
    /// Operating system image (e.g., ubuntu-24.04)
    pub server_image: String,
    /// SSH public key content for server access
    pub ssh_public_key_content: String,
}

/// Builder for creating Hetzner `VariablesContext` instances
///
/// Provides a fluent interface for constructing the context with validation
/// to ensure all required fields are provided.
#[derive(Debug, Default)]
pub struct VariablesContextBuilder {
    instance_name: Option<InstanceName>,
    hcloud_api_token: Option<String>,
    server_type: Option<String>,
    server_location: Option<String>,
    server_image: Option<String>,
    ssh_public_key_content: Option<String>,
}

impl VariablesContextBuilder {
    /// Creates a new builder instance
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the instance name for the server
    ///
    /// # Arguments
    ///
    /// * `instance_name` - The name to assign to the created server
    #[must_use]
    pub fn with_instance_name(mut self, instance_name: InstanceName) -> Self {
        self.instance_name = Some(instance_name);
        self
    }

    /// Sets the Hetzner Cloud API token
    ///
    /// # Arguments
    ///
    /// * `hcloud_api_token` - The API token for Hetzner Cloud authentication
    #[must_use]
    pub fn with_hcloud_api_token(mut self, hcloud_api_token: String) -> Self {
        self.hcloud_api_token = Some(hcloud_api_token);
        self
    }

    /// Sets the server type
    ///
    /// # Arguments
    ///
    /// * `server_type` - The Hetzner server type (e.g., cx22)
    #[must_use]
    pub fn with_server_type(mut self, server_type: String) -> Self {
        self.server_type = Some(server_type);
        self
    }

    /// Sets the server location
    ///
    /// # Arguments
    ///
    /// * `server_location` - The datacenter location (e.g., nbg1)
    #[must_use]
    pub fn with_server_location(mut self, server_location: String) -> Self {
        self.server_location = Some(server_location);
        self
    }

    /// Sets the server image
    ///
    /// # Arguments
    ///
    /// * `server_image` - The OS image (e.g., ubuntu-24.04)
    #[must_use]
    pub fn with_server_image(mut self, server_image: String) -> Self {
        self.server_image = Some(server_image);
        self
    }

    /// Sets the SSH public key content
    ///
    /// # Arguments
    ///
    /// * `ssh_public_key_content` - The content of the SSH public key
    #[must_use]
    pub fn with_ssh_public_key_content(mut self, ssh_public_key_content: String) -> Self {
        self.ssh_public_key_content = Some(ssh_public_key_content);
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
    /// Returns appropriate error variant for each missing required field
    pub fn build(self) -> Result<VariablesContext, VariablesContextError> {
        let instance_name = self
            .instance_name
            .ok_or(VariablesContextError::MissingInstanceName)?;

        let hcloud_api_token = self
            .hcloud_api_token
            .ok_or(VariablesContextError::MissingHcloudApiToken)?;

        let server_type = self
            .server_type
            .ok_or(VariablesContextError::MissingServerType)?;

        let server_location = self
            .server_location
            .ok_or(VariablesContextError::MissingServerLocation)?;

        let server_image = self
            .server_image
            .ok_or(VariablesContextError::MissingServerImage)?;

        let ssh_public_key_content = self
            .ssh_public_key_content
            .ok_or(VariablesContextError::MissingSshPublicKeyContent)?;

        Ok(VariablesContext {
            instance_name,
            hcloud_api_token,
            server_type,
            server_location,
            server_image,
            ssh_public_key_content,
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

    fn create_valid_builder() -> VariablesContextBuilder {
        VariablesContext::builder()
            .with_instance_name(InstanceName::new("test-vm".to_string()).unwrap())
            .with_hcloud_api_token("test-token".to_string())
            .with_server_type("cx22".to_string())
            .with_server_location("nbg1".to_string())
            .with_server_image("ubuntu-24.04".to_string())
            .with_ssh_public_key_content("ssh-rsa AAAA... test@example.com".to_string())
    }

    #[test]
    fn it_should_create_variables_context_with_all_required_fields() {
        let context = create_valid_builder().build().unwrap();

        assert_eq!(context.instance_name.as_str(), "test-vm");
        assert_eq!(context.hcloud_api_token, "test-token");
        assert_eq!(context.server_type, "cx22");
        assert_eq!(context.server_location, "nbg1");
        assert_eq!(context.server_image, "ubuntu-24.04");
        assert_eq!(
            context.ssh_public_key_content,
            "ssh-rsa AAAA... test@example.com"
        );
    }

    #[test]
    fn it_should_serialize_to_json() {
        let context = create_valid_builder().build().unwrap();

        let json = serde_json::to_string(&context).unwrap();
        assert!(json.contains("test-vm"));
        assert!(json.contains("instance_name"));
        assert!(json.contains("hcloud_api_token"));
        assert!(json.contains("server_type"));
        assert!(json.contains("server_location"));
        assert!(json.contains("server_image"));
        assert!(json.contains("ssh_public_key_content"));
    }

    #[test]
    fn it_should_fail_when_instance_name_is_missing() {
        let result = VariablesContext::builder()
            .with_hcloud_api_token("test-token".to_string())
            .with_server_type("cx22".to_string())
            .with_server_location("nbg1".to_string())
            .with_server_image("ubuntu-24.04".to_string())
            .with_ssh_public_key_content("ssh-rsa AAAA...".to_string())
            .build();

        assert!(matches!(
            result.unwrap_err(),
            VariablesContextError::MissingInstanceName
        ));
    }

    #[test]
    fn it_should_fail_when_hcloud_api_token_is_missing() {
        let result = VariablesContext::builder()
            .with_instance_name(InstanceName::new("test-vm".to_string()).unwrap())
            .with_server_type("cx22".to_string())
            .with_server_location("nbg1".to_string())
            .with_server_image("ubuntu-24.04".to_string())
            .with_ssh_public_key_content("ssh-rsa AAAA...".to_string())
            .build();

        assert!(matches!(
            result.unwrap_err(),
            VariablesContextError::MissingHcloudApiToken
        ));
    }

    #[test]
    fn it_should_fail_when_server_type_is_missing() {
        let result = VariablesContext::builder()
            .with_instance_name(InstanceName::new("test-vm".to_string()).unwrap())
            .with_hcloud_api_token("test-token".to_string())
            .with_server_location("nbg1".to_string())
            .with_server_image("ubuntu-24.04".to_string())
            .with_ssh_public_key_content("ssh-rsa AAAA...".to_string())
            .build();

        assert!(matches!(
            result.unwrap_err(),
            VariablesContextError::MissingServerType
        ));
    }

    #[test]
    fn it_should_fail_when_server_location_is_missing() {
        let result = VariablesContext::builder()
            .with_instance_name(InstanceName::new("test-vm".to_string()).unwrap())
            .with_hcloud_api_token("test-token".to_string())
            .with_server_type("cx22".to_string())
            .with_server_image("ubuntu-24.04".to_string())
            .with_ssh_public_key_content("ssh-rsa AAAA...".to_string())
            .build();

        assert!(matches!(
            result.unwrap_err(),
            VariablesContextError::MissingServerLocation
        ));
    }

    #[test]
    fn it_should_fail_when_server_image_is_missing() {
        let result = VariablesContext::builder()
            .with_instance_name(InstanceName::new("test-vm".to_string()).unwrap())
            .with_hcloud_api_token("test-token".to_string())
            .with_server_type("cx22".to_string())
            .with_server_location("nbg1".to_string())
            .with_ssh_public_key_content("ssh-rsa AAAA...".to_string())
            .build();

        assert!(matches!(
            result.unwrap_err(),
            VariablesContextError::MissingServerImage
        ));
    }

    #[test]
    fn it_should_fail_when_ssh_public_key_content_is_missing() {
        let result = VariablesContext::builder()
            .with_instance_name(InstanceName::new("test-vm".to_string()).unwrap())
            .with_hcloud_api_token("test-token".to_string())
            .with_server_type("cx22".to_string())
            .with_server_location("nbg1".to_string())
            .with_server_image("ubuntu-24.04".to_string())
            .build();

        assert!(matches!(
            result.unwrap_err(),
            VariablesContextError::MissingSshPublicKeyContent
        ));
    }

    #[test]
    fn it_should_be_cloneable_when_cloned() {
        let context = create_valid_builder().build().unwrap();
        let cloned = context.clone();

        assert_eq!(
            context.instance_name.as_str(),
            cloned.instance_name.as_str()
        );
        assert_eq!(context.hcloud_api_token, cloned.hcloud_api_token);
    }

    #[test]
    fn it_should_implement_debug_trait_when_formatted() {
        let context = create_valid_builder().build().unwrap();
        let debug = format!("{context:?}");

        assert!(debug.contains("VariablesContext"));
        assert!(debug.contains("instance_name"));
    }
}
