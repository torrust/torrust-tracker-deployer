use super::{AnsibleHost, AnsiblePort, InventoryContext, InventoryContextError, SshPrivateKeyFile};
use crate::infrastructure::templating::TemplateMetadata;

/// Builder for `InventoryContext` with fluent interface
#[derive(Debug, Default)]
#[allow(clippy::struct_field_names)] // Field names mirror Ansible inventory variables
pub struct InventoryContextBuilder {
    metadata: Option<TemplateMetadata>,
    ansible_host: Option<AnsibleHost>,
    ansible_ssh_private_key_file: Option<SshPrivateKeyFile>,
    ansible_port: Option<AnsiblePort>,
    ansible_user: Option<String>,
}

impl InventoryContextBuilder {
    /// Creates a new empty builder
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the template metadata for the builder
    #[must_use]
    pub fn with_metadata(mut self, metadata: TemplateMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets the Ansible host for the builder.
    #[must_use]
    pub fn with_host(mut self, ansible_host: AnsibleHost) -> Self {
        self.ansible_host = Some(ansible_host);
        self
    }

    /// Sets the SSH port for the builder.
    #[must_use]
    pub fn with_ssh_port(mut self, ansible_port: AnsiblePort) -> Self {
        self.ansible_port = Some(ansible_port);
        self
    }

    /// Sets the SSH private key file path for the builder.
    #[must_use]
    pub fn with_ssh_priv_key_path(mut self, ssh_private_key_file: SshPrivateKeyFile) -> Self {
        self.ansible_ssh_private_key_file = Some(ssh_private_key_file);
        self
    }

    /// Sets the Ansible user for the builder.
    #[must_use]
    pub fn with_ansible_user(mut self, ansible_user: String) -> Self {
        self.ansible_user = Some(ansible_user);
        self
    }

    /// Builds the `InventoryContext`
    ///
    /// # Errors
    ///
    /// Returns an error if any required field is missing
    pub fn build(self) -> Result<InventoryContext, InventoryContextError> {
        // Use default metadata if not provided (for backwards compatibility with tests)
        let metadata = self.metadata.unwrap_or_else(|| {
            use crate::shared::clock::{Clock, SystemClock};
            TemplateMetadata::new(SystemClock.now())
        });

        let ansible_host = self
            .ansible_host
            .ok_or(InventoryContextError::MissingAnsibleHost)?;

        let ansible_ssh_private_key_file = self
            .ansible_ssh_private_key_file
            .ok_or(InventoryContextError::MissingSshPrivateKeyFile)?;

        let ansible_port = self
            .ansible_port
            .ok_or(InventoryContextError::MissingSshPort)?;

        let ansible_user = self
            .ansible_user
            .ok_or(InventoryContextError::MissingAnsibleUser)?;

        InventoryContext::new(
            metadata,
            ansible_host,
            ansible_ssh_private_key_file,
            ansible_port,
            ansible_user,
        )
    }
}
