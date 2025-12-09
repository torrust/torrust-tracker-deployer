//! Register command handler implementation

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::RegisterCommandHandlerError;
use crate::adapters::ssh::{SshClient, SshConfig};
use crate::application::services::AnsibleTemplateService;
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::environment::state::{Created, Provisioned};
use crate::domain::environment::Environment;
use crate::domain::EnvironmentName;

/// `RegisterCommandHandler` registers existing instances with environments
///
/// This command handler provides an alternative path to `ProvisionCommandHandler`.
/// Instead of provisioning new infrastructure, it registers an existing instance
/// (VM, physical server, or container) with an environment.
///
/// # State Management
///
/// The command handler integrates with the type-state pattern for environment lifecycle:
/// - Accepts `Environment<Created>` as input
/// - Returns `Environment<Provisioned>` on success
///
/// This allows the environment to continue with `configure`, `release`, and `run`
/// commands just like a normally provisioned environment.
///
/// # Workflow
///
/// 1. Load environment from repository (must be in Created state)
/// 2. Validate SSH connectivity to the provided IP address
/// 3. Render Ansible templates with the instance IP
/// 4. Update runtime outputs with the instance IP and provision method
/// 5. Transition to Provisioned state
/// 6. Persist the updated environment
pub struct RegisterCommandHandler {
    repository: TypedEnvironmentRepository,
}

impl RegisterCommandHandler {
    /// Create a new `RegisterCommandHandler`
    #[must_use]
    pub fn new(repository: Arc<dyn EnvironmentRepository>) -> Self {
        Self {
            repository: TypedEnvironmentRepository::new(repository),
        }
    }

    /// Execute the register workflow
    ///
    /// # Arguments
    ///
    /// * `env_name` - The name of the environment to register the instance with
    /// * `instance_ip` - The IP address of the existing instance
    ///
    /// # Returns
    ///
    /// Returns the environment in Provisioned state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Environment not found or not in `Created` state
    /// * SSH connectivity validation fails
    /// * Ansible template rendering fails
    /// * Unable to persist the environment state
    #[instrument(
        name = "register_command",
        skip_all,
        fields(
            command_type = "register",
            environment = %env_name,
            instance_ip = %instance_ip
        )
    )]
    pub async fn execute(
        &self,
        env_name: &EnvironmentName,
        instance_ip: IpAddr,
    ) -> Result<Environment<Provisioned>, RegisterCommandHandlerError> {
        let environment = self.load_created_environment(env_name)?;

        self.validate_ssh_connectivity(&environment, instance_ip)?;

        self.prepare_for_configuration(&environment, instance_ip)
            .await?;

        let provisioned = environment.register(instance_ip);

        self.repository.save_provisioned(&provisioned)?;

        info!(
            command = "register",
            environment = %provisioned.name(),
            instance_ip = ?provisioned.instance_ip(),
            "Instance registered successfully"
        );

        Ok(provisioned)
    }

    /// Validate SSH connectivity to the instance
    ///
    /// This performs a minimal validation by attempting to establish an SSH connection
    /// to the instance using the credentials from the environment.
    ///
    /// # Errors
    ///
    /// Returns `ConnectivityFailed` if unable to connect via SSH.
    #[allow(clippy::unused_self)] // Method may use self in future for configuration
    fn validate_ssh_connectivity(
        &self,
        environment: &Environment<Created>,
        instance_ip: IpAddr,
    ) -> Result<(), RegisterCommandHandlerError> {
        info!(
            instance_ip = %instance_ip,
            "Validating SSH connectivity to instance"
        );

        let ssh_credentials = environment.ssh_credentials();
        let ssh_port = environment.ssh_port();

        let ssh_socket_addr = SocketAddr::new(instance_ip, ssh_port);
        let ssh_config = SshConfig::new(ssh_credentials.clone(), ssh_socket_addr);
        let ssh_client = SshClient::new(ssh_config);

        let connected = ssh_client.test_connectivity().map_err(|source| {
            RegisterCommandHandlerError::ConnectivityFailed {
                address: instance_ip,
                reason: source.to_string(),
            }
        })?;

        if !connected {
            return Err(RegisterCommandHandlerError::ConnectivityFailed {
                address: instance_ip,
                reason: "SSH connection test returned false".to_string(),
            });
        }

        info!(
            instance_ip = %instance_ip,
            "SSH connectivity validated successfully"
        );

        Ok(())
    }

    /// Prepare for configuration stages
    ///
    /// This method handles preparation for future configuration stages:
    /// - Render Ansible templates with user inputs and instance IP
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Created state
    /// * `instance_ip` - IP address of the instance to register
    ///
    /// # Errors
    ///
    /// Returns an error if Ansible template rendering fails
    async fn prepare_for_configuration(
        &self,
        environment: &Environment<Created>,
        instance_ip: IpAddr,
    ) -> Result<(), RegisterCommandHandlerError> {
        let ansible_template_service = AnsibleTemplateService::from_paths(
            environment.templates_dir(),
            environment.build_dir().clone(),
        );

        ansible_template_service
            .render_templates(&environment.context().user_inputs, instance_ip)
            .await
            .map_err(|e| RegisterCommandHandlerError::TemplateRenderingFailed {
                reason: e.to_string(),
            })?;

        Ok(())
    }

    /// Load environment from storage and validate it is in `Created` state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * Persistence error occurs during load
    /// * Environment does not exist
    /// * Environment is not in `Created` state
    fn load_created_environment(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Created>, RegisterCommandHandlerError> {
        let any_env = self
            .repository
            .inner()
            .load(env_name)
            .map_err(RegisterCommandHandlerError::RepositorySave)?;

        let any_env = any_env.ok_or_else(|| RegisterCommandHandlerError::EnvironmentNotFound {
            name: env_name.clone(),
        })?;

        any_env
            .try_into_created()
            .map_err(|e| RegisterCommandHandlerError::InvalidState {
                name: env_name.clone(),
                current_state: e.to_string(),
            })
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added after the domain layer changes are complete
}
