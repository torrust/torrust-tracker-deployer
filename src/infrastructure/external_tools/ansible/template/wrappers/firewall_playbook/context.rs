//! Context for firewall playbook template rendering
//!
//! This module provides the type-safe context for rendering the
//! `configure-firewall.yml.tera` template with validated SSH port configuration.

use serde::Serialize;
use thiserror::Error;

use crate::infrastructure::external_tools::ansible::template::wrappers::inventory::context::{
    AnsiblePort, AnsiblePortError,
};

/// Errors that can occur when creating a `FirewallPlaybookContext`
#[derive(Debug, Error)]
pub enum FirewallPlaybookContextError {
    /// Invalid SSH port
    #[error("Invalid SSH port: {0}")]
    InvalidSshPort(#[from] AnsiblePortError),

    /// Missing SSH port in context
    #[error("Missing SSH port - must be set before building")]
    MissingSshPort,
}

/// Context for rendering the firewall playbook template
///
/// This context contains the SSH port configuration needed to render
/// the `configure-firewall.yml.tera` template with proper SSH access rules.
#[derive(Serialize, Debug, Clone)]
pub struct FirewallPlaybookContext {
    /// SSH port to allow through the firewall
    ssh_port: AnsiblePort,
}

/// Builder for `FirewallPlaybookContext` with fluent interface
#[derive(Debug, Default)]
pub struct FirewallPlaybookContextBuilder {
    ssh_port: Option<AnsiblePort>,
}

impl FirewallPlaybookContextBuilder {
    /// Creates a new empty builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the SSH port for the builder
    #[must_use]
    pub fn with_ssh_port(mut self, ssh_port: AnsiblePort) -> Self {
        self.ssh_port = Some(ssh_port);
        self
    }

    /// Builds the `FirewallPlaybookContext`
    ///
    /// # Errors
    ///
    /// Returns an error if the SSH port is missing
    pub fn build(self) -> Result<FirewallPlaybookContext, FirewallPlaybookContextError> {
        let ssh_port = self
            .ssh_port
            .ok_or(FirewallPlaybookContextError::MissingSshPort)?;

        Ok(FirewallPlaybookContext { ssh_port })
    }
}

impl FirewallPlaybookContext {
    /// Creates a new `FirewallPlaybookContext` with the specified SSH port
    ///
    /// # Errors
    ///
    /// This method cannot fail with the current implementation since it takes
    /// already validated types, but returns Result for consistency with builder pattern
    pub fn new(ssh_port: AnsiblePort) -> Result<Self, FirewallPlaybookContextError> {
        Ok(Self { ssh_port })
    }

    /// Creates a new builder for `FirewallPlaybookContext` with fluent interface
    #[must_use]
    pub fn builder() -> FirewallPlaybookContextBuilder {
        FirewallPlaybookContextBuilder::new()
    }

    /// Get the SSH port
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.ssh_port.as_u16()
    }

    /// Get the SSH port as a string
    #[must_use]
    pub fn ssh_port_string(&self) -> String {
        self.ssh_port.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_firewall_context_with_builder() {
        let ssh_port = AnsiblePort::new(22).unwrap();
        let context = FirewallPlaybookContext::builder()
            .with_ssh_port(ssh_port)
            .build()
            .unwrap();

        assert_eq!(context.ssh_port(), 22);
    }

    #[test]
    fn it_should_create_firewall_context_directly() {
        let ssh_port = AnsiblePort::new(2222).unwrap();
        let context = FirewallPlaybookContext::new(ssh_port).unwrap();

        assert_eq!(context.ssh_port(), 2222);
    }

    #[test]
    fn it_should_fail_without_ssh_port() {
        let result = FirewallPlaybookContext::builder().build();

        assert!(result.is_err());
        match result {
            Err(FirewallPlaybookContextError::MissingSshPort) => {}
            _ => panic!("Expected MissingSshPort error"),
        }
    }

    #[test]
    fn it_should_serialize_context_to_json() {
        let ssh_port = AnsiblePort::new(22).unwrap();
        let context = FirewallPlaybookContext::new(ssh_port).unwrap();

        let json = serde_json::to_string(&context).unwrap();
        assert!(json.contains("22"));
        assert!(json.contains("ssh_port"));
    }

    #[test]
    fn it_should_support_custom_ssh_ports() {
        let ssh_port = AnsiblePort::new(2222).unwrap();
        let context = FirewallPlaybookContext::new(ssh_port).unwrap();

        assert_eq!(context.ssh_port(), 2222);
        assert_eq!(context.ssh_port_string(), "2222");
    }

    #[test]
    fn it_should_clone_context() {
        let ssh_port = AnsiblePort::new(22).unwrap();
        let context1 = FirewallPlaybookContext::new(ssh_port).unwrap();
        let context2 = context1.clone();

        assert_eq!(context1.ssh_port(), context2.ssh_port());
    }
}
