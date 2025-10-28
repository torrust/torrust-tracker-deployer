//! Create Subcommand Handler
//!
//! This module handles the create subcommand execution at the presentation layer,
//! including configuration file loading, argument processing, user interaction,
//! and command execution.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::application::command_handlers::create::CreateCommandHandler;
use crate::application::command_handlers::create::config::EnvironmentCreationConfig;
use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::presentation::cli::commands::CreateAction;
use crate::presentation::user_output::{UserOutput, VerbosityLevel};
use crate::shared::{Clock, SystemClock};

use super::config_loader::ConfigLoader;
use super::errors::CreateSubcommandError;

/// Handle the create command with its subcommands
///
/// This function routes between different create subcommands (environment or template).
///
/// # Arguments
///
/// * `action` - The create action to perform (environment creation or template generation)
/// * `working_dir` - Root directory for environment data storage
///
/// # Returns
///
/// Returns `Ok(())` on success, or a `CreateSubcommandError` on failure.
///
/// # Errors
///
/// Returns an error if the subcommand execution fails.
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
pub fn handle_create_command(
    action: CreateAction,
    working_dir: &Path,
) -> Result<(), CreateSubcommandError> {
    match action {
        CreateAction::Environment { env_file } => {
            handle_environment_creation(&env_file, working_dir)
        }
        CreateAction::Template { output_path } => {
            let template_path = output_path.unwrap_or_else(CreateAction::default_template_path);
            handle_template_generation(&template_path)
        }
    }
}

/// Handle template generation
///
/// This function generates a configuration template file with placeholder values
/// that users can edit to create their own environment configurations.
///
/// # Arguments
///
/// * `output_path` - Path where the template file should be created
///
/// # Returns
///
/// Returns `Ok(())` on success, or a `CreateSubcommandError` on failure.
///
/// # Errors
///
/// Returns an error if template file creation fails.
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
fn handle_template_generation(output_path: &Path) -> Result<(), CreateSubcommandError> {
    // Create user output for progress messages
    let mut output = UserOutput::new(VerbosityLevel::Normal);

    output.progress("Generating configuration template...");

    // Call existing domain method - template generation implemented in PR #48
    // This is async, so we need to use tokio runtime
    tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            EnvironmentCreationConfig::generate_template_file(output_path)
                .await
                .map_err(CreateSubcommandError::TemplateGenerationFailed)
        })?;

    output.success(&format!(
        "Configuration template generated: {}",
        output_path.display()
    ));
    println!();
    println!("Next steps:");
    println!("1. Edit the template file and replace placeholder values:");
    println!("   - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name (e.g., 'dev', 'staging')");
    println!("   - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key");
    println!("   - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key");
    println!("2. Review default values:");
    println!("   - username: 'torrust' (can be changed if needed)");
    println!("   - port: 22 (standard SSH port)");
    println!("3. Create the environment:");
    println!(
        "   torrust-tracker-deployer create environment --env-file {}",
        output_path.display()
    );

    Ok(())
}

/// Handle environment creation from configuration file
///
/// This function orchestrates the environment creation workflow from the
/// presentation layer by:
/// 1. Loading and parsing the configuration file using Figment
/// 2. Validating the configuration using domain rules
/// 3. Setting up the repository with the working directory
/// 4. Creating the command handler with injected dependencies
/// 5. Executing the create command
/// 6. Providing user-friendly progress updates and error messages
///
/// # Arguments
///
/// * `env_file` - Path to the environment configuration file (JSON format)
/// * `working_dir` - Root directory for environment data storage
///
/// # Returns
///
/// Returns `Ok(())` on success, or a `CreateSubcommandError` if:
/// - Configuration file is not found
/// - Configuration parsing fails
/// - Configuration validation fails
/// - Command execution fails
///
/// # Errors
///
/// This function will return an error if the configuration file cannot be
/// loaded, parsed, validated, or if the create command execution fails.
/// All errors include detailed context and actionable troubleshooting guidance.
#[allow(clippy::result_large_err)] // Error contains detailed context for user guidance
fn handle_environment_creation(
    env_file: &Path,
    working_dir: &Path,
) -> Result<(), CreateSubcommandError> {
    // Create user output with default stdout/stderr channels
    let mut output = UserOutput::new(VerbosityLevel::Normal);

    // Display initial progress (to stderr)
    output.progress(&format!(
        "Loading configuration from '{}'...",
        env_file.display()
    ));

    // Step 1: Load and parse configuration file using Figment
    let loader = ConfigLoader;
    let config: EnvironmentCreationConfig = loader.load_from_file(env_file).inspect_err(|err| {
        output.error(&err.to_string());
    })?;

    output.progress(&format!(
        "Creating environment '{}'...",
        config.environment.name
    ));

    // Step 2: Create repository for environment persistence
    // Use the working directory from CLI args (supports testing and custom locations)
    let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    let repository = repository_factory.create(working_dir.to_path_buf());

    // Step 3: Create clock for timing information
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);

    // Step 4: Create and execute command handler
    let command_handler = CreateCommandHandler::new(repository, clock);

    output.progress("Validating configuration and creating environment...");

    // Step 5: Execute create command
    #[allow(clippy::manual_inspect)]
    let environment = command_handler.execute(config.clone()).map_err(|err| {
        let error = CreateSubcommandError::CommandFailed(err);
        output.error(&error.to_string());
        error
    })?;

    // Step 6: Display success message
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
            CreateSubcommandError::CommandFailed(_) => {
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
}
