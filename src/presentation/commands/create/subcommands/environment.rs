//! Environment Creation Subcommand
//!
//! This module handles the environment creation subcommand for creating
//! deployment environments from configuration files.

use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::domain::Environment;
use crate::presentation::commands::factory::CommandHandlerFactory;
use crate::presentation::progress::ProgressReporter;
use crate::presentation::user_output::UserOutput;

use super::super::config_loader::ConfigLoader;
use super::super::errors::CreateSubcommandError;

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
/// * `user_output` - Shared user output service for consistent output formatting
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
    user_output: &Arc<Mutex<UserOutput>>,
) -> Result<(), CreateSubcommandError> {
    let factory = CommandHandlerFactory::new();
    let ctx = factory.create_context(working_dir.to_path_buf(), user_output.clone());

    // Create progress reporter for 3 main steps
    let mut progress = ProgressReporter::new(ctx.user_output().clone(), 3);

    // Step 1: Load configuration
    progress.start_step("Loading configuration")?;
    let config = load_configuration(progress.output(), env_file)?;
    progress.complete_step(Some(&format!(
        "Configuration loaded: {}",
        config.environment.name
    )))?;

    // Step 2: Initialize dependencies
    progress.start_step("Initializing dependencies")?;
    let command_handler = factory.create_create_handler(&ctx);
    progress.complete_step(None)?;

    // Step 3: Execute create command (provision infrastructure)
    progress.start_step("Creating environment")?;
    let environment = execute_create_command(progress.output(), &command_handler, config)?;
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
    display_creation_results(progress.output(), &environment);

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
/// * `output` - User output for progress messages
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
    user_output: &Arc<Mutex<UserOutput>>,
    env_file: &Path,
) -> Result<EnvironmentCreationConfig, CreateSubcommandError> {
    user_output
        .lock()
        .map_err(|_| CreateSubcommandError::UserOutputLockFailed)?
        .progress(&format!(
            "Loading configuration from '{}'...",
            env_file.display()
        ));

    let loader = ConfigLoader;

    loader.load_from_file(env_file).inspect_err(|err| {
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
/// * `output` - User output for progress messages
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
    user_output: &Arc<Mutex<UserOutput>>,
    command_handler: &crate::application::command_handlers::CreateCommandHandler,
    config: EnvironmentCreationConfig,
) -> Result<Environment, CreateSubcommandError> {
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
/// * `user_output` - Shared user output for result messages
/// * `environment` - The successfully created environment
///
/// # Panics
///
/// This function will panic if the `UserOutput` mutex is poisoned. Since this is
/// called after successful environment creation (when operation is complete),
/// a poisoned mutex indicates an irrecoverable state and panicking is acceptable.
///
/// The panic message provides detailed context matching our error handling principles:
/// clear explanation of what happened, why it's critical, and that it indicates a bug.
fn display_creation_results(user_output: &Arc<Mutex<UserOutput>>, environment: &Environment) {
    let mut output = user_output.lock().expect(
        "CRITICAL: UserOutput mutex poisoned after successful environment creation. \
         This indicates a panic occurred in another thread while holding the output lock. \
         The environment was created successfully, but we cannot display the results. \
         This is a bug - please report it with full logs using --log-output file-and-stderr",
    );

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presentation::user_output::test_support::TestUserOutput;
    use crate::presentation::user_output::VerbosityLevel;
    use std::fs;
    use tempfile::TempDir;

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
        let result = handle_environment_creation(&config_path, working_dir, &user_output);

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

        let result = handle_environment_creation(&config_path, working_dir, &user_output);

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
        let result = handle_environment_creation(&config_path, working_dir, &user_output);

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

        // Create environment first time
        let result1 = handle_environment_creation(&config_path, working_dir, &user_output);
        assert!(result1.is_ok(), "First create should succeed");

        // Try to create same environment again
        let result2 = handle_environment_creation(&config_path, working_dir, &user_output);
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
        let result = handle_environment_creation(&config_path, &custom_working_dir, &user_output);

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
            let result = load_configuration(&user_output, &config_path);

            assert!(result.is_ok(), "Should load valid configuration");
            let config = result.unwrap();
            assert_eq!(config.environment.name, "test-load-config");
        }

        #[test]
        fn it_should_return_error_for_nonexistent_file() {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("missing.json");

            let user_output = TestUserOutput::wrapped(VerbosityLevel::Normal);
            let result = load_configuration(&user_output, &config_path);

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
            let result = load_configuration(&user_output, &config_path);

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
            let loader = ConfigLoader;
            let config = loader.load_from_file(&config_path).unwrap();

            let factory = CommandHandlerFactory::new();
            let ctx = factory.create_context(temp_dir.path().to_path_buf(), user_output.clone());
            let command_handler = factory.create_create_handler(&ctx);
            let result = execute_create_command(&user_output, &command_handler, config);

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
            let loader = ConfigLoader;
            let config = loader.load_from_file(&config_path).unwrap();

            let factory = CommandHandlerFactory::new();
            let ctx = factory.create_context(temp_dir.path().to_path_buf(), user_output.clone());

            // Create environment first time
            let command_handler1 = factory.create_create_handler(&ctx);
            let result1 = execute_create_command(&user_output, &command_handler1, config.clone());
            assert!(result1.is_ok(), "First execution should succeed");

            // Try to create same environment again
            let command_handler2 = factory.create_create_handler(&ctx);
            let result2 = execute_create_command(&user_output, &command_handler2, config);
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
            let factory = CommandHandlerFactory::new();
            let ctx = factory.create_context(temp_dir.path().to_path_buf(), user_output.clone());
            let loader = ConfigLoader;
            let config = loader.load_from_file(&config_path).unwrap();
            let command_handler = factory.create_create_handler(&ctx);
            let environment =
                execute_create_command(&user_output, &command_handler, config).unwrap();

            // Test display function with custom output
            let stderr_buf = Vec::new();
            let stderr_writer = Box::new(Cursor::new(stderr_buf));
            let stdout_buf = Vec::new();
            let stdout_writer = Box::new(Cursor::new(stdout_buf));

            let output =
                UserOutput::with_writers(VerbosityLevel::Normal, stdout_writer, stderr_writer);
            let display_output = Arc::new(Mutex::new(output));

            // This should not panic and should output messages
            display_creation_results(&display_output, &environment);

            // Note: We can't easily verify the exact output without refactoring UserOutput
            // to expose the buffers, but the important thing is it doesn't panic
        }
    }
}
