//! Register command handler implementation

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use tracing::{info, instrument};

use super::errors::RegisterCommandHandlerError;
use crate::adapters::ssh::{SshClient, SshConfig};
use crate::application::steps::RenderAnsibleTemplatesStep;
use crate::domain::environment::repository::{EnvironmentRepository, TypedEnvironmentRepository};
use crate::domain::environment::state::{Created, Provisioned};
use crate::domain::environment::Environment;
use crate::domain::{EnvironmentName, TemplateManager};
use crate::infrastructure::external_tools::ansible::AnsibleTemplateRenderer;

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
        info!(
            command = "register",
            environment = %env_name,
            instance_ip = %instance_ip,
            "Registering existing instance with environment"
        );

        // 1. Load the environment from storage
        let any_env = self
            .repository
            .inner()
            .load(env_name)
            .map_err(RegisterCommandHandlerError::RepositorySave)?;

        // 2. Check if environment exists
        let any_env = any_env.ok_or_else(|| RegisterCommandHandlerError::EnvironmentNotFound {
            name: env_name.clone(),
        })?;

        // 3. Validate environment is in Created state
        let environment =
            any_env
                .try_into_created()
                .map_err(|e| RegisterCommandHandlerError::InvalidState {
                    name: env_name.clone(),
                    current_state: e.to_string(),
                })?;

        // 4. Validate SSH connectivity (minimal validation for v1)
        self.validate_ssh_connectivity(&environment, instance_ip)?;

        // 5. Prepare for configuration (render Ansible templates so configure command will work)
        self.prepare_for_configuration(&environment, instance_ip)
            .await?;

        // 6. Register the instance by setting the IP and transitioning to Provisioned
        let provisioned = environment.register(instance_ip);

        // 7. Persist the updated state
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
    /// - Render Ansible templates with instance IP
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
        let ansible_template_renderer = Self::build_ansible_dependencies(environment);

        self.render_ansible_templates(&ansible_template_renderer, environment, instance_ip)
            .await?;

        Ok(())
    }

    /// Build dependencies for Ansible template rendering
    ///
    /// Creates the template renderer needed for Ansible configuration preparation.
    ///
    /// # Arguments
    ///
    /// * `environment` - The environment in Created state
    ///
    /// # Returns
    ///
    /// Returns `AnsibleTemplateRenderer` for rendering Ansible templates
    fn build_ansible_dependencies(
        environment: &Environment<Created>,
    ) -> Arc<AnsibleTemplateRenderer> {
        let template_manager = Arc::new(TemplateManager::new(environment.templates_dir()));

        Arc::new(AnsibleTemplateRenderer::new(
            environment.build_dir(),
            template_manager,
        ))
    }

    /// Render Ansible templates with the instance IP
    ///
    /// This renders the Ansible inventory and configuration templates so that
    /// the `configure` command can run Ansible playbooks against the registered instance.
    ///
    /// # Arguments
    ///
    /// * `ansible_template_renderer` - The renderer for Ansible templates
    /// * `environment` - The environment in Created state
    /// * `instance_ip` - IP address of the instance to register
    ///
    /// # Errors
    ///
    /// Returns `TemplateRenderingFailed` if Ansible template rendering fails.
    async fn render_ansible_templates(
        &self,
        ansible_template_renderer: &Arc<AnsibleTemplateRenderer>,
        environment: &Environment<Created>,
        instance_ip: IpAddr,
    ) -> Result<(), RegisterCommandHandlerError> {
        info!(
            instance_ip = %instance_ip,
            "Rendering Ansible templates for registered instance"
        );

        let ssh_credentials = environment.ssh_credentials();
        let ssh_port = environment.ssh_port();
        let ssh_socket_addr = SocketAddr::new(instance_ip, ssh_port);

        RenderAnsibleTemplatesStep::new(
            ansible_template_renderer.clone(),
            ssh_credentials.clone(),
            ssh_socket_addr,
        )
        .execute()
        .await
        .map_err(|e| RegisterCommandHandlerError::TemplateRenderingFailed {
            reason: e.to_string(),
        })?;

        info!(
            instance_ip = %instance_ip,
            "Ansible templates rendered successfully"
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Tests will be added after the domain layer changes are complete
}
