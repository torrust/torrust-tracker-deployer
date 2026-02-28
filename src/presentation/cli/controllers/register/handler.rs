//! Register Command Handler
//!
//! This module handles the register command execution at the presentation layer,
//! including environment validation, IP parsing, and user interaction.

use std::cell::RefCell;
use std::net::IpAddr;
use std::sync::Arc;

use parking_lot::ReentrantMutex;

use crate::application::command_handlers::RegisterCommandHandler;
use crate::domain::environment::name::EnvironmentName;
use crate::domain::environment::repository::EnvironmentRepository;
use crate::domain::environment::state::Provisioned;
use crate::domain::environment::Environment;
use crate::presentation::cli::input::cli::OutputFormat;
use crate::presentation::cli::views::commands::register::{
    JsonView, RegisterDetailsData, TextView,
};
use crate::presentation::cli::views::progress::ProgressReporter;
use crate::presentation::cli::views::Render;
use crate::presentation::cli::views::UserOutput;
use crate::shared::clock::Clock;

use super::errors::RegisterSubcommandError;

/// Steps in the register workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RegisterStep {
    ValidateInput,
    CreateCommandHandler,
    RegisterInstance,
}

impl RegisterStep {
    /// All steps in execution order
    const ALL: &'static [Self] = &[
        Self::ValidateInput,
        Self::CreateCommandHandler,
        Self::RegisterInstance,
    ];

    /// Total number of steps
    const fn count() -> usize {
        Self::ALL.len()
    }

    /// User-facing description for the step
    fn description(self) -> &'static str {
        match self {
            Self::ValidateInput => "Validating input",
            Self::CreateCommandHandler => "Creating command handler",
            Self::RegisterInstance => "Registering instance",
        }
    }
}

/// Presentation layer controller for register command workflow
///
/// Coordinates user interaction, progress reporting, and input validation
/// before delegating to the application layer `RegisterCommandHandler`.
///
/// # Responsibilities
///
/// - Validate user input (environment name format, IP address format)
/// - Show progress updates to the user
/// - Format success/error messages for display
/// - Delegate business logic to application layer
///
/// # Architecture
///
/// This controller sits in the presentation layer and handles all user-facing
/// concerns. It delegates actual business logic to the application layer's
/// `RegisterCommandHandler`, maintaining clear separation of concerns.
pub struct RegisterCommandController {
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    clock: Arc<dyn Clock>,
    progress: ProgressReporter,
}

impl RegisterCommandController {
    /// Create a new register command controller
    #[allow(clippy::needless_pass_by_value)] // Constructor takes ownership of Arc parameters
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        clock: Arc<dyn Clock>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(user_output, RegisterStep::count());

        Self {
            repository,
            clock,
            progress,
        }
    }

    /// Execute the complete register workflow
    ///
    /// Orchestrates all steps of the register command:
    /// 1. Validate environment name
    /// 2. Parse and validate IP address
    /// 3. Create command handler
    /// 4. Register the instance
    /// 5. Complete with success message
    ///
    /// # Arguments
    ///
    /// * `environment_name` - The name of the environment to register the instance with
    /// * `instance_ip_str` - The IP address string of the existing instance
    /// * `ssh_port` - Optional SSH port (overrides environment config if provided)
    /// * `output_format` - Output format (text or JSON)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Environment name is invalid
    /// - IP address format is invalid
    /// - Environment is not found or not in Created state
    /// - SSH connectivity validation fails
    #[allow(clippy::result_large_err)]
    pub async fn execute(
        &mut self,
        environment_name: &str,
        instance_ip_str: &str,
        ssh_port: Option<u16>,
        output_format: OutputFormat,
    ) -> Result<Environment<Provisioned>, RegisterSubcommandError> {
        let (env_name, instance_ip) = self.validate_input(environment_name, instance_ip_str)?;

        let handler = self.create_command_handler()?;

        let provisioned = self
            .register_instance(&handler, &env_name, instance_ip, ssh_port)
            .await?;

        self.complete_workflow(&provisioned, output_format)?;

        Ok(provisioned)
    }

    /// Validate input: environment name and IP address
    #[allow(clippy::result_large_err)]
    fn validate_input(
        &mut self,
        name: &str,
        ip_str: &str,
    ) -> Result<(EnvironmentName, IpAddr), RegisterSubcommandError> {
        self.progress
            .start_step(RegisterStep::ValidateInput.description())?;

        let env_name = EnvironmentName::new(name.to_string()).map_err(|source| {
            RegisterSubcommandError::InvalidEnvironmentName {
                name: name.to_string(),
                source,
            }
        })?;

        let instance_ip: IpAddr = ip_str.parse().map_err(|e: std::net::AddrParseError| {
            RegisterSubcommandError::InvalidIpAddress {
                value: ip_str.to_string(),
                reason: e.to_string(),
            }
        })?;

        self.progress.complete_step(None)?;

        Ok((env_name, instance_ip))
    }

    /// Create the application layer command handler
    #[allow(clippy::result_large_err)]
    fn create_command_handler(
        &mut self,
    ) -> Result<RegisterCommandHandler, RegisterSubcommandError> {
        self.progress
            .start_step(RegisterStep::CreateCommandHandler.description())?;

        let handler = RegisterCommandHandler::new(
            self.clock.clone(),
            Arc::clone(&self.repository) as Arc<dyn EnvironmentRepository>,
        );

        self.progress.complete_step(None)?;

        Ok(handler)
    }

    /// Register the instance using the command handler
    #[allow(clippy::result_large_err)]
    async fn register_instance(
        &mut self,
        handler: &RegisterCommandHandler,
        env_name: &EnvironmentName,
        instance_ip: IpAddr,
        ssh_port: Option<u16>,
    ) -> Result<Environment<Provisioned>, RegisterSubcommandError> {
        self.progress
            .start_step(RegisterStep::RegisterInstance.description())?;

        let provisioned = handler
            .execute(env_name, instance_ip, ssh_port)
            .await
            .map_err(|source| RegisterSubcommandError::RegisterOperationFailed {
                name: env_name.to_string(),
                source: Box::new(source),
            })?;

        self.progress.complete_step(None)?;

        Ok(provisioned)
    }

    /// Complete the workflow with success message
    ///
    /// Dispatches to `TextView` or `JsonView` based on `output_format`.
    #[allow(clippy::result_large_err)]
    fn complete_workflow(
        &mut self,
        provisioned: &Environment<Provisioned>,
        output_format: OutputFormat,
    ) -> Result<(), RegisterSubcommandError> {
        let data = RegisterDetailsData::from_environment(provisioned);
        match output_format {
            OutputFormat::Text => {
                self.progress.complete(&TextView::render(&data)?)?;
            }
            OutputFormat::Json => {
                self.progress.result(&JsonView::render(&data)?)?;
            }
        }
        Ok(())
    }
}
