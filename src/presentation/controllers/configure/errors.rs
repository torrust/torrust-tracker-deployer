//! Error types for the Configure Subcommand
//!
//! This module defines error types that can occur during CLI configure command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use thiserror::Error;

use crate::application::command_handlers::configure::errors::ConfigureCommandHandlerError;
use crate::domain::environment::name::EnvironmentNameError;
use crate::presentation::views::progress::ProgressReporterError;

/// Configure command specific errors
///
/// This enum contains all error variants specific to the configure command,
/// including environment validation, repository access, and configuration failures.
/// Each variant includes relevant context and actionable error messages.
#[derive(Debug, Error)]
pub enum ConfigureSubcommandError {
    // ===== Environment Validation Errors =====
    /// Environment name validation failed
    ///
    /// The provided environment name doesn't meet the validation requirements.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Invalid environment name '{name}': {source}
Tip: Environment names must be 1-63 characters, start with letter/digit, contain only letters/digits/hyphens")]
    InvalidEnvironmentName {
        name: String,
        #[source]
        source: EnvironmentNameError,
    },

    /// Environment not found or inaccessible
    ///
    /// The environment couldn't be loaded from persistent storage.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Environment '{name}' not found or inaccessible from data directory '{data_dir}'
Tip: Check if environment exists: ls -la {data_dir}/"
    )]
    EnvironmentNotAccessible { name: String, data_dir: String },

    /// Environment is not in Provisioned state
    ///
    /// The environment is in the wrong state for configuration.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Environment '{name}' is in '{current_state}' state, but 'Provisioned' state is required for configuration
Tip: Only environments in 'Provisioned' state can be configured"
    )]
    InvalidEnvironmentState { name: String, current_state: String },

    // ===== Repository Access Errors =====
    /// Repository operation failed
    ///
    /// Failed to create or access the environment repository.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to access environment repository at '{data_dir}': {reason}
Tip: Check directory permissions and disk space"
    )]
    RepositoryAccessFailed { data_dir: String, reason: String },

    // ===== Configure Operation Errors =====
    /// Configure operation failed
    ///
    /// The configuration process encountered an error during execution.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to configure environment '{name}': {source}
Tip: Check logs and try running with --log-output file-and-stderr for more details"
    )]
    ConfigureOperationFailed {
        name: String,
        #[source]
        source: Box<ConfigureCommandHandlerError>,
    },

    // ===== Internal Errors =====
    /// Progress reporting failed
    ///
    /// Failed to report progress to the user due to an internal error.
    /// This indicates a critical internal error.
    #[error(
        "Failed to report progress: {source}
Tip: This is a critical bug - please report it with full logs using --log-output file-and-stderr"
    )]
    ProgressReportingFailed {
        #[source]
        source: ProgressReporterError,
    },
}

// ============================================================================
// ERROR CONVERSIONS
// ============================================================================

impl From<ProgressReporterError> for ConfigureSubcommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}

impl ConfigureSubcommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    ///
    /// # Example
    ///
    /// Using with Container and `ExecutionContext` (recommended):
    ///
    /// ```rust
    /// use std::path::Path;
    /// use std::sync::Arc;
    /// use torrust_tracker_deployer_lib::bootstrap::Container;
    /// use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
    /// use torrust_tracker_deployer_lib::presentation::controllers::configure;
    /// use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
    ///
    /// let container = Container::new(VerbosityLevel::Normal, Path::new("."));
    /// let context = ExecutionContext::new(Arc::new(container));
    ///
    /// if let Err(e) = context
    ///     .container()
    ///     .create_configure_controller()
    ///     .execute("test-env")
    /// {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// ```
    ///
    /// Direct usage (for testing):
    ///
    /// ```rust
    /// use std::path::{Path, PathBuf};
    /// use std::sync::Arc;
    /// use std::time::Duration;
    /// use parking_lot::ReentrantMutex;
    /// use std::cell::RefCell;
    /// use torrust_tracker_deployer_lib::presentation::controllers::configure::handler::ConfigureCommandController;
    /// use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
    /// use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
    /// use torrust_tracker_deployer_lib::shared::clock::SystemClock;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(VerbosityLevel::Normal))));
    /// let data_dir = PathBuf::from("./data");
    /// let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    /// let repository = repository_factory.create(data_dir);
    /// let clock = Arc::new(SystemClock);
    /// if let Err(e) = ConfigureCommandController::new(repository, clock, output).execute("test-env") {
    ///     eprintln!("Error: {e}");
    ///     eprintln!("\nTroubleshooting:\n{}", e.help());
    /// }
    /// # }
    /// ```
    #[must_use]
    #[allow(clippy::too_many_lines)] // Help text is comprehensive for user guidance
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidEnvironmentName { .. } => {
                "Invalid Environment Name - Detailed Troubleshooting:

1. Check environment name format:
   - Length: Must be 1-63 characters
   - Start: Must begin with a letter or digit
   - Characters: Only letters, digits, and hyphens allowed
   - No special characters: Avoid spaces, underscores, dots

2. Valid examples:
   - 'production'
   - 'staging-01'
   - 'dev-environment'

3. Invalid examples:
   - 'prod_01' (underscore not allowed)
   - '-production' (cannot start with hyphen)
   - 'prod.env' (dots not allowed)

For more information, see environment naming documentation."
            }

            Self::EnvironmentNotAccessible { .. } => {
                "Environment Not Accessible - Detailed Troubleshooting:

1. Verify environment exists:
   - List environments: ls -la data/
   - Check for environment.json file in data/<environment-name>/

2. Check file permissions:
   - Read permission: chmod +r data/<environment-name>/environment.json
   - Directory access: chmod +rx data/<environment-name>/

3. Verify data directory:
   - Ensure data/ directory exists
   - Check disk space: df -h
   - Verify no file locks: lsof data/

4. Common solutions:
   - Create environment first: torrust-tracker-deployer create environment -f <config-file>
   - Provision environment: torrust-tracker-deployer provision <env-name>
   - Check working directory: pwd
   - Verify correct environment name

For more information, see the provision command documentation."
            }

            Self::InvalidEnvironmentState { .. } => {
                "Invalid Environment State - Detailed Troubleshooting:

The configure command requires an environment in 'Provisioned' state.

1. Check current state:
   - View environment file: cat data/<environment-name>/environment.json
   - Look for 'state' field

2. State transitions:
   - Provisioned → Configuring → Configured (success path)
   - Provisioned → Configuring → ConfigureFailed (error path)

3. If environment is already Configured:
   - Environment is ready to use
   - No need to configure again
   - Proceed to next deployment steps

4. If environment is in ConfigureFailed state:
   - Review error logs to understand failure
   - Fix underlying issues (network, permissions, etc.)
   - Try configuring again

5. If environment is not yet Provisioned:
   - Provision environment first: torrust-tracker-deployer provision <env-name>
   - Wait for provisioning to complete
   - Then run configure command

For more information, see environment lifecycle documentation."
            }

            Self::RepositoryAccessFailed { .. } => {
                "Repository Access Failed - Detailed Troubleshooting:

1. Check directory permissions:
   - Read/write access: ls -la data/
   - Fix permissions: chmod -R u+rw data/

2. Verify disk space:
   - Check available space: df -h
   - Free up space if needed

3. Check for file locks:
   - List open files: lsof data/
   - Kill processes if safe to do so

4. Verify directory exists:
   - Create if missing: mkdir -p data/
   - Check parent directory permissions

5. Common issues:
   - Running multiple instances simultaneously
   - File system mounted read-only
   - Insufficient permissions

For persistent issues, check system logs and file system health."
            }

            Self::ConfigureOperationFailed { .. } => {
                "Configure Operation Failed - Detailed Troubleshooting:

1. Review detailed error logs:
   - Run with verbose output: --log-output file-and-stderr
   - Check log files in data/logs/

2. Common failure points:
   - Ansible playbook execution failures
   - Docker installation issues
   - Docker Compose installation problems
   - SSH connectivity issues
   - System package manager failures

3. Infrastructure-specific troubleshooting:
   
   SSH connectivity:
   - Verify SSH keys exist: ls -la ~/.ssh/
   - Check VM/server is running using provider tools
   - Test SSH manually: ssh -i <key> <user>@<ip>
   
   Docker installation:
   - SSH into server and check: docker --version
   - Verify package manager: apt-get update (or equivalent)
   - Check internet connectivity: ping -c 3 google.com
   
   Ansible issues:
   - Check Ansible is installed: ansible --version
   - Verify playbook syntax: ansible-playbook --syntax-check <playbook>
   - Review Ansible logs in data/logs/

4. Recovery steps:
   - Review error messages and logs
   - Fix underlying issues (network, permissions, etc.)
   - Try configure command again
   - If persistent, destroy and recreate environment

For more information, see the configuration troubleshooting guide."
            }

            Self::ProgressReportingFailed { .. } => {
                "Progress Reporting Failed - Critical Internal Error:

This is a critical internal error that should not occur during normal operation.

1. Immediate actions:
   - Save full error output
   - Copy log files from data/logs/
   - Note the exact command that was running

2. Report the issue:
   - Create GitHub issue with full details
   - Include: command, error output, logs, system info
   - Describe steps to reproduce

3. Temporary workarounds:
   - Try running command again
   - Restart application
   - Check for system resource issues (memory, file descriptors)

This error indicates a bug in the progress reporting system.
Please report it so we can fix it."
            }
        }
    }
}
