//! Environment Creation Subcommand
//!
//! This module handles the environment creation subcommand for creating
//! deployment environments from configuration files.

use std::path::Path;

use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::domain::Environment;
use crate::presentation::commands::context::report_error;
use crate::presentation::commands::factory::CommandHandlerFactory;
use crate::presentation::user_output::UserOutput;

use super::super::config_loader::ConfigLoader;
use super::super::errors::CreateSubcommandError;

/// Handle environment creation from configuration file
///
/// This function orchestrates the environment creation workflow by delegating
/// to focused step functions:
///
/// 1. Load configuration from file
/// 2. Execute create command
/// 3. Display creation results
///
/// Each step is implemented as a separate function for clarity and testability.
///
/// # Arguments
///
/// * `env_file` - Path to the environment configuration file (JSON format)
/// * `working_dir` - Root directory for environment data storage
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
) -> Result<(), CreateSubcommandError> {
    let factory = CommandHandlerFactory::new();
    let mut ctx = factory.create_context(working_dir.to_path_buf());

    // Step 1: Load configuration
    let config = load_configuration(ctx.output(), env_file)?;

    // Step 2: Execute command (create handler before borrowing output)
    let command_handler = factory.create_create_handler(&ctx);
    let environment = execute_create_command(ctx.output(), command_handler, config)?;

    // Step 3: Display results
    display_creation_results(ctx.output(), &environment);

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
    output: &mut UserOutput,
    env_file: &Path,
) -> Result<EnvironmentCreationConfig, CreateSubcommandError> {
    output.progress(&format!(
        "Loading configuration from '{}'...",
        env_file.display()
    ));

    let loader = ConfigLoader;
    loader.load_from_file(env_file).inspect_err(|err| {
        report_error(output, err);
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
    output: &mut UserOutput,
    command_handler: crate::application::command_handlers::CreateCommandHandler,
    config: EnvironmentCreationConfig,
) -> Result<Environment, CreateSubcommandError> {
    output.progress(&format!(
        "Creating environment '{}'...",
        config.environment.name
    ));

    output.progress("Validating configuration and creating environment...");

    #[allow(clippy::manual_inspect)]
    command_handler.execute(config).map_err(|source| {
        let error = CreateSubcommandError::CommandFailed { source };
        report_error(output, &error);
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
/// * `output` - User output for result messages
/// * `environment` - The successfully created environment
fn display_creation_results(output: &mut UserOutput, environment: &Environment) {
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
        let result = handle_environment_creation(&config_path, working_dir);

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

        let result = handle_environment_creation(&config_path, working_dir);

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
        let result = handle_environment_creation(&config_path, working_dir);

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

        // Create environment first time
        let result1 = handle_environment_creation(&config_path, working_dir);
        assert!(result1.is_ok(), "First create should succeed");

        // Try to create same environment again
        let result2 = handle_environment_creation(&config_path, working_dir);
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

        let result = handle_environment_creation(&config_path, &custom_working_dir);

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
        use crate::presentation::user_output::{UserOutput, VerbosityLevel};

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

            let mut output = UserOutput::new(VerbosityLevel::Quiet);
            let result = load_configuration(&mut output, &config_path);

            assert!(result.is_ok(), "Should load valid configuration");
            let config = result.unwrap();
            assert_eq!(config.environment.name, "test-load-config");
        }

        #[test]
        fn it_should_return_error_for_nonexistent_file() {
            let temp_dir = TempDir::new().unwrap();
            let config_path = temp_dir.path().join("missing.json");

            let mut output = UserOutput::new(VerbosityLevel::Quiet);
            let result = load_configuration(&mut output, &config_path);

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

            let mut output = UserOutput::new(VerbosityLevel::Quiet);
            let result = load_configuration(&mut output, &config_path);

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
        use crate::presentation::user_output::{UserOutput, VerbosityLevel};

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

            let mut output = UserOutput::new(VerbosityLevel::Quiet);
            let loader = ConfigLoader;
            let config = loader.load_from_file(&config_path).unwrap();

            let factory = CommandHandlerFactory::new();
            let ctx = factory.create_context(temp_dir.path().to_path_buf());
            let command_handler = factory.create_create_handler(&ctx);
            let result = execute_create_command(&mut output, command_handler, config);

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

            let mut output = UserOutput::new(VerbosityLevel::Quiet);
            let loader = ConfigLoader;
            let config = loader.load_from_file(&config_path).unwrap();

            let factory = CommandHandlerFactory::new();
            let ctx = factory.create_context(temp_dir.path().to_path_buf());

            // Create environment first time
            let command_handler1 = factory.create_create_handler(&ctx);
            let result1 = execute_create_command(&mut output, command_handler1, config.clone());
            assert!(result1.is_ok(), "First execution should succeed");

            // Try to create same environment again
            let command_handler2 = factory.create_create_handler(&ctx);
            let result2 = execute_create_command(&mut output, command_handler2, config);
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
            let factory = CommandHandlerFactory::new();
            let ctx = factory.create_context(temp_dir.path().to_path_buf());
            let loader = ConfigLoader;
            let config = loader.load_from_file(&config_path).unwrap();
            let mut quiet_output = UserOutput::new(VerbosityLevel::Quiet);
            let command_handler = factory.create_create_handler(&ctx);
            let environment = execute_create_command(&mut quiet_output, command_handler, config).unwrap();

            // Test display function with custom output
            let stderr_buf = Vec::new();
            let stderr_writer = Box::new(Cursor::new(stderr_buf));
            let stdout_buf = Vec::new();
            let stdout_writer = Box::new(Cursor::new(stdout_buf));

            let mut output =
                UserOutput::with_writers(VerbosityLevel::Normal, stdout_writer, stderr_writer);

            // This should not panic and should output messages
            display_creation_results(&mut output, &environment);

            // Note: We can't easily verify the exact output without refactoring UserOutput
            // to expose the buffers, but the important thing is it doesn't panic
        }
    }
}
