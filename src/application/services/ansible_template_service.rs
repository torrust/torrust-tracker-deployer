//! Ansible Template Service
//!
//! This service is responsible for rendering Ansible templates with runtime
//! configuration. It's used by multiple command handlers (Provision, Register)
//! to prepare Ansible inventory and playbook files before configuration.
//!
//! ## Usage
//!
//! The service is injected with its dependencies (template renderer) at construction
//! time and receives only the data needed to render templates at execution time.
//!
//! ```rust,ignore
//! use torrust_tracker_deployer_lib::application::services::AnsibleTemplateService;
//!
//! // Create service with dependencies
//! let service = AnsibleTemplateService::new(ansible_template_renderer);
//!
//! // Render templates with user inputs and instance IP
//! service.render_templates(&user_inputs, instance_ip).await?;
//! ```

use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use thiserror::Error;
use tracing::info;

use crate::application::steps::RenderAnsibleTemplatesStep;
use crate::domain::environment::UserInputs;
use crate::domain::TemplateManager;
use crate::infrastructure::templating::ansible::AnsibleProjectGenerator;
use crate::shared::clock::Clock;

/// Errors that can occur during Ansible template rendering
#[derive(Error, Debug)]
pub enum AnsibleTemplateServiceError {
    /// Template rendering failed
    #[error("Failed to render Ansible templates: {reason}")]
    RenderingFailed {
        /// Detailed reason for the failure
        reason: String,
    },
}

/// Service for rendering Ansible templates with runtime configuration
///
/// This service encapsulates the logic for rendering Ansible inventory and
/// configuration templates. It's designed to be shared across command handlers
/// that need to prepare Ansible files (e.g., Provision, Register).
///
/// ## Design
///
/// The service follows dependency injection principles:
/// - Dependencies (template renderer) are injected at construction time
/// - Runtime data (SSH credentials, IP, port) is passed to the render method
///
/// This allows the service to be configured once and reused with different
/// runtime parameters.
pub struct AnsibleTemplateService {
    ansible_template_renderer: Arc<AnsibleProjectGenerator>,
    clock: Arc<dyn Clock>,
}

impl AnsibleTemplateService {
    /// Create a new `AnsibleTemplateService`
    ///
    /// # Arguments
    ///
    /// * `ansible_template_renderer` - The renderer for Ansible templates
    /// * `clock` - The clock for generating timestamps
    #[must_use]
    pub fn new(
        ansible_template_renderer: Arc<AnsibleProjectGenerator>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            ansible_template_renderer,
            clock,
        }
    }

    /// Build an `AnsibleTemplateService` from environment paths
    ///
    /// This is a factory method that creates the service with all necessary
    /// dependencies based on the environment's template and build directories.
    ///
    /// # Arguments
    ///
    /// * `templates_dir` - Directory containing the source templates
    /// * `build_dir` - Directory where rendered templates will be written
    /// * `clock` - The clock for generating timestamps
    ///
    /// # Returns
    ///
    /// Returns a configured `AnsibleTemplateService` ready for template rendering
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::path::PathBuf;
    /// use std::sync::Arc;
    /// use torrust_tracker_deployer_lib::application::services::AnsibleTemplateService;
    /// use torrust_tracker_deployer_lib::shared::clock::SystemClock;
    ///
    /// let service = AnsibleTemplateService::from_paths(
    ///     PathBuf::from("templates"),
    ///     PathBuf::from("build/my-env"),
    ///     Arc::new(SystemClock),
    /// );
    /// ```
    #[must_use]
    pub fn from_paths(templates_dir: PathBuf, build_dir: PathBuf, clock: Arc<dyn Clock>) -> Self {
        let template_manager = Arc::new(TemplateManager::new(templates_dir));

        let ansible_template_renderer =
            Arc::new(AnsibleProjectGenerator::new(build_dir, template_manager));

        Self::new(ansible_template_renderer, clock)
    }

    /// Render Ansible templates with the provided runtime configuration
    ///
    /// This renders the Ansible inventory and configuration templates so that
    /// Ansible playbooks can be executed against the target instance.
    ///
    /// # Arguments
    ///
    /// * `user_inputs` - User-provided environment configuration (SSH credentials, tracker config, etc.)
    /// * `instance_ip` - IP address of the provisioned instance (runtime output)
    /// * `ssh_port_override` - Optional SSH port override (takes precedence over `user_inputs.ssh_port`)
    ///
    /// # Errors
    ///
    /// Returns `AnsibleTemplateServiceError::RenderingFailed` if template rendering fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use std::net::IpAddr;
    ///
    /// let service = AnsibleTemplateService::new(renderer);
    /// service.render_templates(&user_inputs, "192.168.1.100".parse().unwrap(), None).await?;
    /// ```
    pub async fn render_templates(
        &self,
        user_inputs: &UserInputs,
        instance_ip: IpAddr,
        ssh_port_override: Option<u16>,
    ) -> Result<(), AnsibleTemplateServiceError> {
        let effective_ssh_port = ssh_port_override.unwrap_or(user_inputs.ssh_port());

        info!(
            instance_ip = %instance_ip,
            ssh_port = effective_ssh_port,
            ssh_port_override = ?ssh_port_override,
            "Rendering Ansible templates"
        );

        let ssh_socket_addr = SocketAddr::new(instance_ip, effective_ssh_port);

        RenderAnsibleTemplatesStep::new(
            self.ansible_template_renderer.clone(),
            user_inputs.ssh_credentials().clone(),
            ssh_socket_addr,
            user_inputs.tracker().clone(),
            user_inputs.grafana().cloned(),
            self.clock.clone(),
        )
        .execute()
        .await
        .map_err(|e| AnsibleTemplateServiceError::RenderingFailed {
            reason: e.to_string(),
        })?;

        info!(
            instance_ip = %instance_ip,
            ssh_port = effective_ssh_port,
            "Ansible templates rendered successfully"
        );

        Ok(())
    }
}
