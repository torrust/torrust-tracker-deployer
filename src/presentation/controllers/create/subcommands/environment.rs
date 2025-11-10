//! Environment Creation Subcommand
//!
//! This module handles the environment creation subcommand for creating
//! deployment environments from configuration files.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use thiserror::Error;

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::application::command_handlers::create::CreateCommandHandlerError;
use crate::application::command_handlers::CreateCommandHandler;
use crate::domain::Environment;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::dispatch::ExecutionContext;
use crate::presentation::progress::{ProgressReporter, ProgressReporterError};
use crate::presentation::user_output::UserOutput;
use crate::shared::clock::SystemClock;

use super::super::config_loader::ConfigLoader;

/// Handle environment creation from configuration file
///
/// This function orchestrates the environment creation workflow with progress reporting:
///
/// 1. Load configuration from file
/// 2. Initialize dependencies
/// 3. Validate environment
/// 4. Execute create command
/// 5. Display creation results
///
/// Each step is tracked and timed using `ProgressReporter` for clear user feedback.
///
/// # Arguments
///
/// * `env_file` - Path to the environment configuration file (JSON format)
/// * `working_dir` - Root directory for environment data storage
/// * `context` - Execution context providing access to application services
///
/// # Returns
///
/// Returns `Ok(())` on success, or a `CreateSubcommandError` if any step fails.
///
/// # Errors
///
/// This function will return an error if:
/// - Configuration file cannot be loaded or validated
/// - Command execution fails
///
/// All errors include detailed context and actionable troubleshooting guidance.
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle_environment_creation(
    env_file: &Path,
    working_dir: &Path,
    context: &ExecutionContext,
) -> Result<(), CreateSubcommandError> {
    // Create progress reporter for 3 main steps
    let mut progress = ProgressReporter::new(context.user_output().clone(), 3);

    // Step 1: Load configuration
    progress.start_step("Loading configuration")?;
    let config = load_configuration(&mut progress, env_file)?;
    progress.complete_step(Some(&format!(
        "Configuration loaded: {}",
        config.environment.name
    )))?;

    // Step 2: Initialize dependencies
    progress.start_step("Initializing dependencies")?;

    // Create repository and clock services
    // TODO: Once Container is expanded, get these from context.container()
    let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    let repository = repository_factory.create(working_dir.to_path_buf());
    let clock = Arc::new(SystemClock);

    let command_handler = CreateCommandHandler::new(repository, clock);
    progress.complete_step(None)?;

    // Step 3: Execute create command (provision infrastructure)
    progress.start_step("Creating environment")?;
    let environment = execute_create_command(&mut progress, &command_handler, config)?;
    progress.complete_step(Some(&format!(
        "Instance created: {}",
        environment.instance_name().as_str()
    )))?;

    // Complete with summary
    progress.complete(&format!(
        "Environment '{}' created successfully",
        environment.name().as_str()
    ))?;

    // Display final results
    display_creation_results(&context.user_output(), &environment)?;

    Ok(())
}

/// Load and validate configuration from file
///
/// This step handles:
/// - Loading configuration file using Figment
/// - Parsing JSON content
/// - Validating configuration using domain rules
///
/// # Arguments
///
/// * `progress` - Progress reporter for user feedback
/// * `env_file` - Path to the configuration file
///
/// # Returns
///
/// Returns the loaded and validated `EnvironmentCreationConfig`.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration file is not found
/// - JSON parsing fails
/// - Domain validation fails
fn load_configuration(
    progress: &mut ProgressReporter,
    env_file: &Path,
) -> Result<EnvironmentCreationConfig, CreateSubcommandError> {
    let user_output = progress.output();

    user_output
        .lock()
        .map_err(|_| CreateSubcommandError::UserOutputLockFailed)?
        .progress(&format!(
            "Loading configuration from '{}'...",
            env_file.display()
        ));

    let loader = ConfigLoader;

    loader
        .load_from_file(env_file)
        .inspect_err(|err: &CreateSubcommandError| {
            // Attempt to log error, but don't fail if mutex is poisoned
            if let Ok(mut output) = user_output.lock() {
                output.error(&err.to_string());
            }
        })
}

/// Execute the create command with the given configuration
///
/// This step handles:
/// - Executing the create command with the given handler
/// - Handling command execution errors
///
/// # Arguments
///
/// * `progress` - Progress reporter for user feedback
/// * `command_handler` - Pre-created command handler
/// * `config` - Validated environment creation configuration
///
/// # Returns
///
/// Returns the created `Environment` on success.
///
/// # Errors
///
/// Returns an error if command execution fails (e.g., environment already exists).
fn execute_create_command(
    progress: &mut ProgressReporter,
    command_handler: &CreateCommandHandler,
    config: EnvironmentCreationConfig,
) -> Result<Environment, CreateSubcommandError> {
    let user_output = progress.output();

    user_output
        .lock()
        .map_err(|_| CreateSubcommandError::UserOutputLockFailed)?
        .progress(&format!(
            "Creating environment '{}'...",
            config.environment.name
        ));

    user_output
        .lock()
        .map_err(|_| CreateSubcommandError::UserOutputLockFailed)?
        .progress("Validating configuration and creating environment...");

    #[allow(clippy::manual_inspect)]
    command_handler.execute(config).map_err(|source| {
        let error = CreateSubcommandError::CommandFailed { source };
        // Attempt to log error, but don't fail if mutex is poisoned
        if let Ok(mut output) = user_output.lock() {
            output.error(&error.to_string());
        }
        error
    })
}

/// Display the results of successful environment creation
///
/// This step outputs:
/// - Success message with environment name
/// - Instance name
/// - Data directory location
/// - Build directory location
///
/// # Arguments
///
/// * `user_output` - Shared user output for displaying messages
/// * `environment` - The successfully created environment
///
/// # Returns
///
/// Returns `Ok(())` on success, or `CreateSubcommandError::UserOutputLockFailed`
/// if the UserOutput mutex is poisoned.
///
/// # Errors
///
/// This function will return an error if the `UserOutput` mutex is poisoned,
/// which indicates a panic occurred in another thread while holding the output lock.
fn display_creation_results(
    user_output: &Arc<Mutex<UserOutput>>,
    environment: &Environment,
) -> Result<(), CreateSubcommandError> {
    let mut output = user_output
        .lock()
        .map_err(|_| CreateSubcommandError::UserOutputLockFailed)?;

    output.success(&format!(
        "Environment '{}' created successfully",
        environment.name().as_str()
    ));

    output.result(&format!(
        "Instance name: {}",
        environment.instance_name().as_str()
    ));

    output.result(&format!(
        "Data directory: {}",
        environment.data_dir().display()
    ));

    output.result(&format!(
        "Build directory: {}",
        environment.build_dir().display()
    ));

    Ok(())
}

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Format of configuration file
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    /// JSON format
    Json,
}

impl std::fmt::Display for ConfigFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "JSON"),
        }
    }
}

/// Errors that can occur during create subcommand execution
///
/// These errors represent failures in the CLI presentation layer when
/// handling the create command. They provide structured context for
/// troubleshooting and user feedback.
#[derive(Debug, Error)]
pub enum CreateSubcommandError {
    // ===== Configuration File Errors =====
    /// Configuration file not found
    ///
    /// The specified configuration file does not exist or is not accessible.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Configuration file not found: {path}
Tip: Check that the file path is correct: ls -la {path}"
    )]
    ConfigFileNotFound {
        /// Path to the missing configuration file
        path: PathBuf,
    },

    /// Failed to parse configuration file
    ///
    /// The configuration file exists but could not be parsed in the expected format.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Failed to parse configuration file '{path}' as {format}: {source}
Tip: Validate {format} syntax with: jq . < {path}"
    )]
    ConfigParsingFailed {
        /// Path to the configuration file
        path: PathBuf,
        /// Expected format of the file
        format: ConfigFormat,
        /// Underlying parsing error
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// Configuration validation failed
    ///
    /// The configuration file was parsed successfully but contains invalid values.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Configuration validation failed: {source}
Tip: Review the validation error and fix the configuration file"
    )]
    ConfigValidationFailed {
        /// Underlying validation error from domain layer
        #[source]
        source: crate::application::command_handlers::create::config::CreateConfigError,
    },

    // ===== Command Execution Errors =====
    /// Command execution failed
    ///
    /// The create operation failed during execution after validation passed.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Create command execution failed: {source}
Tip: Check logs with --log-output file-and-stderr for detailed error information"
    )]
    CommandFailed {
        /// Underlying command handler error
        #[source]
        source: CreateCommandHandlerError,
    },

    // ===== Template Generation Errors =====
    /// Template generation failed
    ///
    /// Failed to generate template configuration or files.
    /// Use `.help()` for detailed troubleshooting steps.
    #[error(
        "Template generation failed: {source}
Tip: Check that you have write permissions in the target directory"
    )]
    TemplateGenerationFailed {
        /// Underlying template generation error from domain layer
        #[source]
        source: crate::application::command_handlers::create::config::CreateConfigError,
    },

    // ===== User Output Lock Errors =====
    /// User output lock acquisition failed
    ///
    /// Failed to acquire the mutex lock for user output. This indicates a panic
    /// occurred in another thread while holding the lock.
    #[error("Failed to acquire user output lock - a panic occurred in another thread")]
    UserOutputLockFailed,

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

impl From<ProgressReporterError> for CreateSubcommandError {
    fn from(source: ProgressReporterError) -> Self {
        Self::ProgressReportingFailed { source }
    }
}

impl CreateSubcommandError {
    /// Provides detailed troubleshooting guidance for this error
    ///
    /// Returns context-specific help text that guides users toward resolving
    /// the issue. This implements the project's tiered help system pattern
    /// for actionable error messages.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use torrust_tracker_deployer_lib::presentation::controllers::create::CreateSubcommandError;
    /// use std::path::PathBuf;
    ///
    /// let error = CreateSubcommandError::ConfigFileNotFound {
    ///     path: PathBuf::from("config.json"),
    /// };
    ///
    /// let help = error.help();
    /// assert!(help.contains("File Not Found"));
    /// assert!(help.contains("Check that the file path"));
    /// ```
    #[must_use]
    pub fn help(&self) -> &'static str {
        match self {
            Self::ConfigFileNotFound { .. } => {
                "Configuration File Not Found - Troubleshooting:

1. Check that the file path is correct in your --env-file argument
2. Verify the file exists: ls -la <path>
3. Ensure you have read permissions on the file
4. Use absolute paths or paths relative to current directory

Example:
  torrust-tracker-deployer create environment --env-file ./config/environment.json

For more information about configuration format, see the documentation."
            }
            Self::ConfigParsingFailed { format, .. } => match format {
                ConfigFormat::Json => {
                    "JSON Configuration Parsing Failed - Troubleshooting:

1. Validate JSON syntax using a JSON validator:
   - Online: jsonlint.com
   - Command line: jq . < your-config.json

2. Common JSON syntax errors:
   - Missing or extra commas
   - Missing quotes around strings
   - Unclosed braces or brackets
   - Invalid escape sequences

3. Verify required fields are present:
   - environment.name
   - ssh_credentials.private_key_path
   - ssh_credentials.public_key_path

4. Check field types match expectations:
   - Strings must be in quotes
   - Numbers should not have quotes
   - Booleans are true/false (lowercase)

Example valid configuration:
{
  \"environment\": {
    \"name\": \"dev\"
  },
  \"ssh_credentials\": {
    \"private_key_path\": \"fixtures/testing_rsa\",
    \"public_key_path\": \"fixtures/testing_rsa.pub\"
  }
}

For more information, see the configuration documentation."
                }
            },
            Self::ConfigValidationFailed { source } | Self::TemplateGenerationFailed { source } => {
                source.help()
            }
            Self::CommandFailed { source } => source.help(),
            Self::UserOutputLockFailed => {
                "User Output Lock Failed - Troubleshooting:

This error indicates that a panic occurred in another thread while it was using
the user output system, leaving the mutex in a \"poisoned\" state.

1. Check for any error messages that appeared before this one
   - The original panic message should appear earlier in the output
   - This will indicate what caused the initial failure

2. This is typically caused by:
   - A bug in the application code that caused a panic
   - An unhandled error condition that triggered a panic
   - Resource exhaustion (memory, file handles, etc.)

3. If you can reproduce this issue:
   - Run with --verbose to see more detailed logging
   - Report the issue with the full error output and steps to reproduce

This is a serious application error that indicates a bug. Please report it to the developers."
            }
            Self::ProgressReportingFailed { .. } => {
                "Progress Reporting Failed - Critical Internal Error:

This error indicates that the progress reporting system encountered a critical
internal error while trying to update the user interface. This is a BUG in the
application and should NOT occur under normal circumstances.

Immediate Actions:
1. Save any logs using: --log-output file-and-stderr
2. Note the operation that was in progress when this occurred
3. Record any error messages that appeared before this one
4. Document the current state of your environment

Report the Issue:
1. Include the full log output (--log-output file-and-stderr)
2. Provide steps to reproduce the error
3. Include your environment configuration file
4. Note your operating system and version
5. Report to: https://github.com/torrust/torrust-tracker-deployer/issues

Workaround:
1. Restart the application and retry the operation
2. Try the operation again with --verbose for more details
3. Check system resources (memory, disk space)
4. Check file system permissions

This error means the operation may have PARTIALLY completed or FAILED.
Verify the state of your environment before retrying."
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap::Container;
    use crate::presentation::controllers::create::config_loader::ConfigLoader;
    use crate::presentation::dispatch::ExecutionContext;
    use crate::presentation::user_output::test_support::TestUserOutput;
    use crate::presentation::user_output::{UserOutput, VerbosityLevel};
    use std::fs;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;

    fn create_test_context(
        _working_dir: &Path,
        _user_output: Arc<Mutex<UserOutput>>,
    ) -> ExecutionContext {
        let container = Container::new(VerbosityLevel::Silent);
        ExecutionContext::new(Arc::new(container))
    }

    #[test]
    fn it_should_create_environment_from_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        // Use absolute paths to SSH keys to ensure they work regardless of current directory
        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        // Write a valid configuration file
        let config_json = format!(
            r#"{{
            "environment": {{
                "name": "test-create-env"
            }},
            "ssh_credentials": {{
                "private_key_path": "{private_key_path}",
                "public_key_path": "{public_key_path}"
            }}
        }}"#
        );
        fs::write(&config_path, config_json).unwrap();

        let working_dir = temp_dir.path();
        let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        let context = create_test_context(working_dir, user_output);
        let result = handle_environment_creation(&config_path, working_dir, &context);

        assert!(
            result.is_ok(),
            "Should successfully create environment: {:?}",
            result.err()
        );

        // Verify environment state file was created by repository
        // Repository creates: <base_dir>/{env-name}/environment.json
        let env_state_file = working_dir.join("test-create-env/environment.json");
        assert!(
            env_state_file.exists(),
            "Environment state file should be created at: {}",
            env_state_file.display()
        );
    }

    #[test]
    fn it_should_return_error_for_missing_config_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.json");
        let working_dir = temp_dir.path();
        let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        let context = create_test_context(working_dir, user_output);

        let result = handle_environment_creation(&config_path, working_dir, &context);

        assert!(result.is_err());
        match result.unwrap_err() {
            CreateSubcommandError::ConfigFileNotFound { path } => {
                assert_eq!(path, config_path);
            }
            other => panic!("Expected ConfigFileNotFound, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_for_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("invalid.json");

        // Write invalid JSON
        fs::write(&config_path, r#"{"invalid json"#).unwrap();

        let working_dir = temp_dir.path();
        let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        let context = create_test_context(working_dir, user_output);
        let result = handle_environment_creation(&config_path, working_dir, &context);

        assert!(result.is_err());
        match result.unwrap_err() {
            CreateSubcommandError::ConfigParsingFailed { .. } => {
                // Expected
            }
            other => panic!("Expected ConfigParsingFailed, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_return_error_for_duplicate_environment() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");

        // Use absolute paths to SSH keys to ensure they work regardless of current directory
        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config_json = format!(
            r#"{{
            "environment": {{
                "name": "duplicate-env"
            }},
            "ssh_credentials": {{
                "private_key_path": "{private_key_path}",
                "public_key_path": "{public_key_path}"
            }}
        }}"#
        );
        fs::write(&config_path, config_json).unwrap();

        let working_dir = temp_dir.path();
        let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        let context = create_test_context(working_dir, user_output);

        // Create environment first time
        let result1 = handle_environment_creation(&config_path, working_dir, &context);
        assert!(result1.is_ok(), "First create should succeed");

        // Try to create same environment again (use new context to avoid any state issues)
        let user_output2 = TestUserOutput::wrapped(VerbosityLevel::Normal);
        let context2 = create_test_context(working_dir, user_output2);
        let result2 = handle_environment_creation(&config_path, working_dir, &context2);
        assert!(result2.is_err(), "Second create should fail");

        match result2.unwrap_err() {
            CreateSubcommandError::CommandFailed { .. } => {
                // Expected - environment already exists
            }
            other => panic!("Expected CommandFailed, got: {other:?}"),
        }
    }

    #[test]
    fn it_should_create_environment_in_custom_working_dir() {
        let temp_dir = TempDir::new().unwrap();
        let custom_working_dir = temp_dir.path().join("custom");
        fs::create_dir(&custom_working_dir).unwrap();

        let config_path = temp_dir.path().join("config.json");

        // Use absolute paths to SSH keys to ensure they work regardless of current directory
        let project_root = env!("CARGO_MANIFEST_DIR");
        let private_key_path = format!("{project_root}/fixtures/testing_rsa");
        let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

        let config_json = format!(
            r#"{{
            "environment": {{
                "name": "custom-location-env"
            }},
            "ssh_credentials": {{
                "private_key_path": "{private_key_path}",
                "public_key_path": "{public_key_path}"
            }}
        }}"#
        );
        fs::write(&config_path, config_json).unwrap();

        let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
        let context = create_test_context(&custom_working_dir, user_output);
        let result = handle_environment_creation(&config_path, &custom_working_dir, &context);

        assert!(result.is_ok(), "Should create in custom working dir");

        // Verify environment was created in custom location
        // Repository creates: <base_dir>/{env-name}/environment.json
        let env_state_file = custom_working_dir.join("custom-location-env/environment.json");
        assert!(
            env_state_file.exists(),
            "Environment state should be in custom working directory at: {}",
            env_state_file.display()
        );
    }

    // ============================================================================
    // UNIT TESTS - Helper Functions
    // ============================================================================

    mod load_configuration_tests {
        use super::*;

        #[test]
        fn it_should_load_valid_configuration() {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("config.json");

            let project_root = env!("CARGO_MANIFEST_DIR");
            let private_key_path = format!("{project_root}/fixtures/testing_rsa");
            let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

            let config_json = format!(
                r#"{{
                "environment": {{
                    "name": "test-load-config"
                }},
                "ssh_credentials": {{
                    "private_key_path": "{private_key_path}",
                    "public_key_path": "{public_key_path}"
                }}
            }}"#
            );
            fs::write(&config_path, config_json).unwrap();

            let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
            let mut progress = ProgressReporter::new(user_output, 5);
            let result = load_configuration(&mut progress, &config_path);

            assert!(result.is_ok(), "Should load valid configuration");
            let config = result.unwrap();
            assert_eq!(config.environment.name, "test-load-config");
        }

        #[test]
        fn it_should_return_error_for_nonexistent_file() {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("missing.json");

            let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
            let mut progress = ProgressReporter::new(user_output, 5);
            let result = load_configuration(&mut progress, &config_path);

            assert!(result.is_err());
            match result.unwrap_err() {
                CreateSubcommandError::ConfigFileNotFound { path } => {
                    assert_eq!(path, config_path);
                }
                other => panic!("Expected ConfigFileNotFound, got: {other:?}"),
            }
        }

        #[test]
        fn it_should_return_error_for_invalid_json() {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("invalid.json");
            fs::write(&config_path, r#"{"broken json"#).unwrap();

            let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
            let mut progress = ProgressReporter::new(user_output, 5);
            let result = load_configuration(&mut progress, &config_path);

            assert!(result.is_err());
            match result.unwrap_err() {
                CreateSubcommandError::ConfigParsingFailed { .. } => {
                    // Expected
                }
                other => panic!("Expected ConfigParsingFailed, got: {other:?}"),
            }
        }
    }

    mod execute_create_command_tests {
        use super::*;

        #[test]
        fn it_should_execute_command_successfully() {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("config.json");

            let project_root = env!("CARGO_MANIFEST_DIR");
            let private_key_path = format!("{project_root}/fixtures/testing_rsa");
            let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

            let config_json = format!(
                r#"{{
                "environment": {{
                    "name": "test-execute"
                }},
                "ssh_credentials": {{
                    "private_key_path": "{private_key_path}",
                    "public_key_path": "{public_key_path}"
                }}
            }}"#
            );
            fs::write(&config_path, config_json).unwrap();

            let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
            let _context = create_test_context(temp_dir.path(), user_output.clone());
            let loader = ConfigLoader;
            let config = loader.load_from_file(&config_path).unwrap();

            // Create command handler using manual dependency creation
            // TODO: Once Container is expanded, get these from context.container()
            let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
            let repository = repository_factory.create(temp_dir.path().to_path_buf());
            let clock = Arc::new(SystemClock);
            let command_handler = CreateCommandHandler::new(repository, clock);

            // Create ProgressReporter for the function call (use 5 total steps)
            let mut progress = ProgressReporter::new(user_output, 5);
            let result = execute_create_command(&mut progress, &command_handler, config);

            assert!(result.is_ok(), "Should execute command successfully");
            let environment = result.unwrap();
            assert_eq!(environment.name().as_str(), "test-execute");
        }

        #[test]
        fn it_should_return_error_for_duplicate_environment() {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("config.json");

            let project_root = env!("CARGO_MANIFEST_DIR");
            let private_key_path = format!("{project_root}/fixtures/testing_rsa");
            let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

            let config_json = format!(
                r#"{{
                "environment": {{
                    "name": "test-duplicate"
                }},
                "ssh_credentials": {{
                    "private_key_path": "{private_key_path}",
                    "public_key_path": "{public_key_path}"
                }}
            }}"#
            );
            fs::write(&config_path, config_json).unwrap();

            let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
            let _context = create_test_context(temp_dir.path(), user_output.clone());
            let loader = ConfigLoader;
            let config = loader.load_from_file(&config_path).unwrap();

            // Create command handler using manual dependency creation
            // TODO: Once Container is expanded, get these from context.container()
            let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
            let repository = repository_factory.create(temp_dir.path().to_path_buf());
            let clock = Arc::new(SystemClock);
            let command_handler = CreateCommandHandler::new(repository, clock);

            // Create environment first time
            let mut progress1 = ProgressReporter::new(user_output.clone(), 5);
            let result1 = execute_create_command(&mut progress1, &command_handler, config.clone());
            assert!(result1.is_ok(), "First execution should succeed");

            // Try to create same environment again
            let mut progress2 = ProgressReporter::new(user_output, 5);
            let result2 = execute_create_command(&mut progress2, &command_handler, config);
            assert!(result2.is_err(), "Second execution should fail");

            match result2.unwrap_err() {
                CreateSubcommandError::CommandFailed { .. } => {
                    // Expected
                }
                other => panic!("Expected CommandFailed, got: {other:?}"),
            }
        }
    }

    mod display_creation_results_tests {
        use super::*;
        use crate::presentation::user_output::{UserOutput, VerbosityLevel};
        use std::io::Cursor;

        #[test]
        fn it_should_display_environment_details() {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("config.json");

            let project_root = env!("CARGO_MANIFEST_DIR");
            let private_key_path = format!("{project_root}/fixtures/testing_rsa");
            let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

            let config_json = format!(
                r#"{{
                "environment": {{
                    "name": "test-display"
                }},
                "ssh_credentials": {{
                    "private_key_path": "{private_key_path}",
                    "public_key_path": "{public_key_path}"
                }}
            }}"#
            );
            fs::write(&config_path, config_json).unwrap();

            // Create environment
            let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
            let _context = create_test_context(temp_dir.path(), user_output.clone());
            let loader = ConfigLoader;
            let config = loader.load_from_file(&config_path).unwrap();

            // Create command handler using manual dependency creation
            // TODO: Once Container is expanded, get these from context.container()
            let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
            let repository = repository_factory.create(temp_dir.path().to_path_buf());
            let clock = Arc::new(SystemClock);
            let command_handler = CreateCommandHandler::new(repository, clock);

            let mut progress = ProgressReporter::new(user_output, 5);
            let environment =
                execute_create_command(&mut progress, &command_handler, config).unwrap();

            // Test display function with custom output
            let stderr_buf = Vec::new();
            let stderr_writer = Box::new(Cursor::new(stderr_buf));
            let stdout_buf = Vec::new();
            let stdout_writer = Box::new(Cursor::new(stdout_buf));

            let output =
                UserOutput::with_writers(VerbosityLevel::Normal, stdout_writer, stderr_writer);
            let display_output = Arc::new(Mutex::new(output));

            // Test display function with the user output directly
            let result = display_creation_results(&display_output, &environment);
            assert!(result.is_ok(), "display_creation_results should succeed");

            // Note: We can't easily verify the exact output without refactoring UserOutput
            // to expose the buffers, but the important thing is it succeeds
        }
    }

    // ============================================================================
    // ERROR TESTS
    // ============================================================================

    mod error_tests {
        use super::*;

        #[test]
        fn it_should_provide_help_for_config_file_not_found() {
            let error = CreateSubcommandError::ConfigFileNotFound {
                path: PathBuf::from("missing.json"),
            };

            let help = error.help();
            assert!(help.contains("File Not Found"));
            assert!(help.contains("Check that the file path"));
            assert!(help.contains("ls -la"));
        }

        #[test]
        fn it_should_provide_help_for_json_parsing_failed() {
            let source = std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid json");
            let error = CreateSubcommandError::ConfigParsingFailed {
                path: PathBuf::from("config.json"),
                format: ConfigFormat::Json,
                source: Box::new(source),
            };

            let help = error.help();
            assert!(help.contains("JSON"));
            assert!(help.contains("syntax"));
            assert!(help.contains("jq"));
        }

        #[test]
        fn it_should_display_config_file_path_in_error() {
            let error = CreateSubcommandError::ConfigFileNotFound {
                path: PathBuf::from("/path/to/config.json"),
            };

            let message = error.to_string();
            assert!(message.contains("/path/to/config.json"));
            assert!(message.contains("not found"));
        }

        #[test]
        fn it_should_display_format_in_parsing_error() {
            let source = std::io::Error::new(std::io::ErrorKind::InvalidData, "test");
            let error = CreateSubcommandError::ConfigParsingFailed {
                path: PathBuf::from("config.json"),
                format: ConfigFormat::Json,
                source: Box::new(source),
            };

            let message = error.to_string();
            assert!(message.contains("JSON"));
            assert!(message.contains("config.json"));
        }

        #[test]
        fn it_should_have_help_for_all_error_variants() {
            use crate::application::command_handlers::create::config::CreateConfigError;
            use crate::domain::EnvironmentNameError;
            use crate::presentation::progress::ProgressReporterError;

            let errors: Vec<CreateSubcommandError> = vec![
                CreateSubcommandError::ConfigFileNotFound {
                    path: PathBuf::from("test.json"),
                },
                CreateSubcommandError::ConfigParsingFailed {
                    path: PathBuf::from("test.json"),
                    format: ConfigFormat::Json,
                    source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "test")),
                },
                CreateSubcommandError::ConfigValidationFailed {
                    source: CreateConfigError::InvalidEnvironmentName(
                        EnvironmentNameError::InvalidFormat {
                            attempted_name: "test".to_string(),
                            reason: "invalid".to_string(),
                            valid_examples: vec!["dev".to_string()],
                        },
                    ),
                },
                CreateSubcommandError::UserOutputLockFailed,
                CreateSubcommandError::ProgressReportingFailed {
                    source: ProgressReporterError::UserOutputMutexPoisoned,
                },
            ];

            for error in errors {
                let help = error.help();
                assert!(!help.is_empty(), "Help text should not be empty");
                assert!(
                    help.contains("Troubleshooting") || help.contains("Fix") || help.len() > 50,
                    "Help should contain actionable guidance"
                );
            }
        }
    }
}
