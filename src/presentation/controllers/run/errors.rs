//! Error types for the Run Subcommand
//!
//! This module defines error types that can occur during CLI run command execution.
//! All errors follow the project's error handling principles by providing clear,
//! contextual, and actionable error messages with `.help()` methods.

use thiserror::Error;

use crate::application::command_handlers::run::RunCommandHandlerError;
use crate::domain::environment::name::EnvironmentNameError;
use crate::presentation::views::progress::ProgressReporterError;

/// Run command specific errors
///
/// This enum contains all error variants specific to the run command,
/// including environment validation, state validation, and service start failures.
/// Each variant includes relevant context and actionable error messages.
#[derive(Debug, Error)]
pub enum RunSubcommandError {
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

    // ===== State Validation Errors =====
    /// Environment is not in the required state for run
    ///
    /// The run command requires the environment to be in the Released state.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Environment '{name}' is not in the required state for run (current: {current_state}, required: Released)
Tip: Run 'release' command first to deploy the application"
    )]
    InvalidEnvironmentState { name: String, current_state: String },

    // ===== Run Operation Errors =====
    /// Run operation failed
    ///
    /// The run process encountered an error during execution.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to run application stack in environment '{name}': {reason}
Tip: Check logs and try running with --log-output file-and-stderr for more details"
    )]
    RunOperationFailed { name: String, reason: String },

    /// Service start failed
    ///
    /// One or more services failed to start properly.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to start services in environment '{name}': {reason}
Tip: SSH into the VM and check Docker container logs: docker compose logs"
    )]
    ServiceStartFailed { name: String, reason: String },

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

impl From<ProgressReporterError> for RunSubcommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}

impl From<RunCommandHandlerError> for RunSubcommandError {
    fn from(error: RunCommandHandlerError) -> Self {
        match error {
            RunCommandHandlerError::EnvironmentNotFound { name } => {
                Self::EnvironmentNotAccessible {
                    name,
                    data_dir: "data".to_string(),
                }
            }
            RunCommandHandlerError::InvalidState(state_err) => Self::InvalidEnvironmentState {
                name: "environment".to_string(),
                current_state: state_err.to_string(),
            },
            RunCommandHandlerError::MissingInstanceIp { name } => Self::RunOperationFailed {
                name,
                reason: "Instance IP not available - environment may not be fully provisioned"
                    .to_string(),
            },
            RunCommandHandlerError::StartServicesFailed { message, .. } => {
                Self::ServiceStartFailed {
                    name: "environment".to_string(),
                    reason: message,
                }
            }
            RunCommandHandlerError::StatePersistence(err) => Self::RunOperationFailed {
                name: "environment".to_string(),
                reason: format!("Failed to persist state: {err}"),
            },
            RunCommandHandlerError::RunOperationFailed { name, message } => {
                Self::RunOperationFailed {
                    name,
                    reason: message,
                }
            }
        }
    }
}

impl RunSubcommandError {
    /// Get detailed troubleshooting guidance for this error
    ///
    /// This method provides comprehensive troubleshooting steps that can be
    /// displayed to users when they need more help resolving the error.
    #[must_use]
    #[allow(clippy::too_many_lines)] // Help text is comprehensive for user guidance
    pub fn help(&self) -> &'static str {
        match self {
            Self::InvalidEnvironmentName { .. } => {
                "Invalid Environment Name - Detailed Troubleshooting:

1. Check environment name format:
   - Length: Must be 1-63 characters
   - Start: Must begin with letter (a-z, A-Z) or digit (0-9)
   - Characters: Only letters, digits, and hyphens allowed
   - End: Must not end with a hyphen

2. Common valid examples:
   - 'production'
   - 'test-env'
   - 'e2e-provision'
   - 'dev123'

3. Common invalid examples:
   - 'test_env' (underscores not allowed)
   - '-test' (starts with hyphen)
   - 'test-' (ends with hyphen)
   - '' (empty string)

For more information, see the environment naming conventions in the documentation."
            }

            Self::EnvironmentNotAccessible { .. } => {
                "Environment Not Accessible - Detailed Troubleshooting:

1. Check if environment exists:
   - List environments: ls -la data/
   - Look for directory with your environment name

2. Verify file permissions:
   - Check directory permissions: ls -ld data/
   - Ensure read/write access: chmod 755 data/

3. Check if environment was created:
   - Look for environment.json file: ls -la data/{env_name}/
   - Verify it's a valid deployment environment

4. Common causes:
   - Environment was never created (run create first)
   - Wrong data directory path
   - Permission issues
   - Corrupted environment state

If the environment should exist, check the logs for more details."
            }

            Self::InvalidEnvironmentState { .. } => {
                "Invalid Environment State - Detailed Troubleshooting:

1. Check current environment state:
   - The run command requires the environment to be 'Released'
   - Run the workflow in order: create → provision → configure → release → run

2. Required workflow:
   - First: torrust-tracker-deployer create environment -f env.json
   - Then: torrust-tracker-deployer provision <env-name>
   - Then: torrust-tracker-deployer configure <env-name>
   - Then: torrust-tracker-deployer release <env-name>
   - Finally: torrust-tracker-deployer run <env-name>

3. Check environment state:
   - Look at data/<env-name>/environment.json
   - Verify the 'state' field shows 'Released'

4. Common issues:
   - Skipped the 'release' step
   - Previous release command failed
   - Environment was reset or recreated

Run 'release' command first, then retry 'run'."
            }

            Self::RunOperationFailed { .. } => {
                "Run Operation Failed - Detailed Troubleshooting:

1. Check system resources:
   - Ensure sufficient disk space on target VM
   - Check network connectivity to the VM
   - Verify SSH access is working

2. Review the operation logs:
   - Run with verbose logging: --log-output file-and-stderr
   - Check log files in data/logs/
   - Look for specific error details

3. Check Docker status on VM:
   - SSH into the VM
   - Run: docker compose ps
   - Check container health: docker ps -a

4. Common issues:
   - Docker daemon not running
   - Port conflicts with existing services
   - Missing Docker images
   - Resource constraints (memory, CPU)

5. Recovery options:
   - Retry the run operation
   - Run 'release' again to reset deployment
   - Manually stop conflicting services on VM

For persistent issues, check the deployment documentation."
            }

            Self::ServiceStartFailed { .. } => {
                "Service Start Failed - Detailed Troubleshooting:

1. Check service logs:
   - SSH into the VM
   - Run: docker compose logs -f
   - Look for startup errors

2. Verify service configuration:
   - Check docker-compose.yml syntax
   - Verify environment variables are set
   - Ensure required secrets are available

3. Check resource availability:
   - Sufficient memory for all services
   - Available ports (not already in use)
   - Required volumes and mounts exist

4. Common service issues:
   - Database connection failures
   - Missing configuration files
   - Network connectivity between containers
   - Permission issues on mounted volumes

5. Manual debugging:
   - SSH into the VM
   - Run: docker compose up (without -d) to see logs
   - Check individual container logs

For persistent issues, check individual service documentation."
            }

            Self::ProgressReportingFailed { .. } => {
                "Progress Reporting Failed - Critical Internal Error:

This is a critical bug that indicates progress reporting to the user failed.
This should never happen in normal operation.

1. Immediate actions:
   - Save all relevant logs and error messages
   - Note what operation was being performed
   - Record the environment state

2. Report the issue:
   - This is a bug that needs to be reported
   - Include full logs: --log-output file-and-stderr
   - Provide steps to reproduce if possible
   - Include system information (OS, versions)

3. Workaround:
   - Restart the application
   - Try the operation again
   - Check for resource exhaustion (memory, threads)

This error indicates a serious bug in the application's progress reporting system.
Please report it to the development team with full details."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_provide_help_for_invalid_environment_name() {
        let error = RunSubcommandError::InvalidEnvironmentName {
            name: "invalid_name".to_string(),
            source: EnvironmentNameError::InvalidFormat {
                attempted_name: "invalid_name".to_string(),
                reason: "contains underscore".to_string(),
                valid_examples: vec!["dev".to_string(), "staging".to_string()],
            },
        };

        let help = error.help();
        assert!(help.contains("Invalid Environment Name"));
        assert!(help.contains("Check environment name format"));
    }

    #[test]
    fn it_should_provide_help_for_environment_not_accessible() {
        let error = RunSubcommandError::EnvironmentNotAccessible {
            name: "test-env".to_string(),
            data_dir: "/tmp/data".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Environment Not Accessible"));
        assert!(help.contains("Check if environment exists"));
    }

    #[test]
    fn it_should_provide_help_for_invalid_environment_state() {
        let error = RunSubcommandError::InvalidEnvironmentState {
            name: "test-env".to_string(),
            current_state: "Configured".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Invalid Environment State"));
        assert!(help.contains("release"));
    }

    #[test]
    fn it_should_provide_help_for_service_start_failed() {
        let error = RunSubcommandError::ServiceStartFailed {
            name: "test-env".to_string(),
            reason: "container exited".to_string(),
        };

        let help = error.help();
        assert!(help.contains("Service Start Failed"));
        assert!(help.contains("docker compose logs"));
    }

    #[test]
    fn it_should_have_help_for_all_error_variants() {
        let errors: Vec<RunSubcommandError> = vec![
            RunSubcommandError::InvalidEnvironmentName {
                name: "test".to_string(),
                source: EnvironmentNameError::InvalidFormat {
                    attempted_name: "test".to_string(),
                    reason: "invalid".to_string(),
                    valid_examples: vec!["dev".to_string()],
                },
            },
            RunSubcommandError::EnvironmentNotAccessible {
                name: "test".to_string(),
                data_dir: "/tmp".to_string(),
            },
            RunSubcommandError::InvalidEnvironmentState {
                name: "test".to_string(),
                current_state: "Created".to_string(),
            },
            RunSubcommandError::RunOperationFailed {
                name: "test".to_string(),
                reason: "connection failed".to_string(),
            },
            RunSubcommandError::ServiceStartFailed {
                name: "test".to_string(),
                reason: "timeout".to_string(),
            },
        ];

        for error in errors {
            let help = error.help();
            assert!(!help.is_empty(), "Help text should not be empty");
            assert!(
                help.contains("Troubleshooting") || help.len() > 50,
                "Help should contain actionable guidance"
            );
        }
    }

    #[test]
    fn it_should_display_error_with_context() {
        let error = RunSubcommandError::InvalidEnvironmentName {
            name: "invalid_env".to_string(),
            source: EnvironmentNameError::InvalidFormat {
                attempted_name: "invalid_env".to_string(),
                reason: "contains underscore".to_string(),
                valid_examples: vec!["dev".to_string()],
            },
        };

        let message = error.to_string();
        assert!(message.contains("invalid_env"));
        assert!(message.contains("Invalid environment name"));
    }
}
