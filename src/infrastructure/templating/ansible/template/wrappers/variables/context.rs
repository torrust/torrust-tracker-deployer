use serde::Serialize;
use thiserror::Error;

/// Errors that can occur when creating an `AnsibleVariablesContext`
#[derive(Debug, Error)]
pub enum AnsibleVariablesContextError {
    /// Invalid SSH port
    #[error("Invalid SSH port: {0}")]
    InvalidSshPort(#[from] crate::infrastructure::templating::ansible::template::wrappers::inventory::context::AnsiblePortError),
}

/// Context for rendering the variables.yml.tera template
///
/// This context contains system configuration variables used across
/// Ansible playbooks (but NOT inventory connection variables).
#[derive(Serialize, Debug, Clone)]
pub struct AnsibleVariablesContext {
    /// SSH port to configure in firewall and other services
    ssh_port: u16,
}

impl AnsibleVariablesContext {
    /// Creates a new context with the specified SSH port
    ///
    /// # Errors
    ///
    /// Returns an error if the SSH port is invalid (0 or out of range)
    pub fn new(ssh_port: u16) -> Result<Self, AnsibleVariablesContextError> {
        // Validate SSH port using existing validation
        crate::infrastructure::templating::ansible::template::wrappers::inventory::context::AnsiblePort::new(ssh_port)?;

        Ok(Self { ssh_port })
    }

    /// Get the SSH port
    #[must_use]
    pub fn ssh_port(&self) -> u16 {
        self.ssh_port
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_create_context_with_valid_ssh_port() {
        let context = AnsibleVariablesContext::new(22).unwrap();
        assert_eq!(context.ssh_port(), 22);
    }

    #[test]
    fn it_should_create_context_with_custom_ssh_port() {
        let context = AnsibleVariablesContext::new(2222).unwrap();
        assert_eq!(context.ssh_port(), 2222);
    }

    #[test]
    fn it_should_create_context_with_high_port() {
        let context = AnsibleVariablesContext::new(65535).unwrap();
        assert_eq!(context.ssh_port(), 65535);
    }

    #[test]
    fn it_should_fail_with_port_zero() {
        let result = AnsibleVariablesContext::new(0);
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Invalid SSH port"));
    }

    #[test]
    fn it_should_implement_clone() {
        let context1 = AnsibleVariablesContext::new(22).unwrap();
        let context2 = context1.clone();
        assert_eq!(context1.ssh_port(), context2.ssh_port());
    }

    #[test]
    fn it_should_serialize_to_json() {
        let context = AnsibleVariablesContext::new(8022).unwrap();
        let json = serde_json::to_string(&context).unwrap();
        assert!(json.contains("\"ssh_port\":8022"));
    }

    #[test]
    fn it_should_display_error_message_correctly() {
        let error = AnsibleVariablesContext::new(0).unwrap_err();
        let error_msg = format!("{error}");
        assert!(error_msg.contains("Invalid SSH port"));
        assert!(error_msg.contains("Invalid port number: 0"));
    }
}
