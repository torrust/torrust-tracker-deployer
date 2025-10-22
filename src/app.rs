//! Main application module for the Torrust Tracker Deployer CLI
//!
//! This module contains the CLI structure and main application logic.
//! It initializes logging and handles the application lifecycle.

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::info;

use torrust_tracker_deployer_lib::logging::{LogFormat, LogOutput, LoggingBuilder};

/// Command-line interface for Torrust Tracker Deployer
#[derive(Parser, Debug)]
#[command(name = "torrust-tracker-deployer")]
#[command(about = "Automated deployment infrastructure for Torrust Tracker")]
#[command(version)]
#[allow(clippy::struct_field_names)] // CLI arguments intentionally share 'log_' prefix for clarity
pub struct Cli {
    /// Format for file logging (default: compact, without ANSI codes)
    ///
    /// - pretty: Pretty-printed output for development (no ANSI in files)
    /// - json: JSON output for production environments (no ANSI)
    /// - compact: Compact output for minimal verbosity (no ANSI in files)
    ///
    /// Note: ANSI color codes are automatically disabled for file output
    /// to ensure logs are easily parsed with standard text tools (grep, awk, sed).
    #[arg(long, value_enum, default_value = "compact", global = true)]
    pub log_file_format: LogFormat,

    /// Format for stderr logging (default: pretty, with ANSI codes)
    ///
    /// - pretty: Pretty-printed output with colors for development
    /// - json: JSON output for machine processing
    /// - compact: Compact output with colors for minimal verbosity
    ///
    /// Note: ANSI color codes are automatically enabled for stderr output
    /// to provide colored terminal output for better readability.
    #[arg(long, value_enum, default_value = "pretty", global = true)]
    pub log_stderr_format: LogFormat,

    /// Log output mode (default: file-only for production)
    ///
    /// - file-only: Write logs to file only (production mode)
    /// - file-and-stderr: Write logs to both file and stderr (development/testing mode)
    #[arg(long, value_enum, default_value = "file-only", global = true)]
    pub log_output: LogOutput,

    /// Log directory (default: ./data/logs)
    ///
    /// Directory where log files will be written. The log file will be
    /// named 'log.txt' inside this directory. Parent directories will be
    /// created automatically if they don't exist.
    ///
    /// Note: If the directory cannot be created due to filesystem permissions,
    /// the application will exit with an error. Logging is critical for
    /// observability and the application cannot function without it.
    #[arg(long, default_value = "./data/logs", global = true)]
    pub log_dir: PathBuf,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available CLI commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Destroy an existing deployment environment
    Destroy {
        /// Name of the environment to destroy
        environment: String,
    },
}

/// Main application entry point
///
/// This function initializes logging, displays information to the user,
/// and executes the requested command if provided.
///
/// # Panics
///
/// This function will panic if:
/// - Log directory cannot be created (filesystem permissions issue)
/// - Logging initialization fails (usually means it was already initialized)
///
/// Both panics are intentional as logging is critical for observability.
pub fn run() {
    let cli = Cli::parse();

    // Clone values for logging before moving them
    let log_file_format = cli.log_file_format.clone();
    let log_stderr_format = cli.log_stderr_format.clone();
    let log_output = cli.log_output;
    let log_dir = cli.log_dir.clone();

    // Initialize logging FIRST before any other logic
    LoggingBuilder::new(&cli.log_dir)
        .with_file_format(cli.log_file_format)
        .with_stderr_format(cli.log_stderr_format)
        .with_output(cli.log_output)
        .init();

    // Log startup event with configuration details
    info!(
        app = "torrust-tracker-deployer",
        version = env!("CARGO_PKG_VERSION"),
        log_dir = %log_dir.display(),
        log_file_format = ?log_file_format,
        log_stderr_format = ?log_stderr_format,
        log_output = ?log_output,
        "Application started"
    );

    // Execute command if provided, otherwise display help
    match cli.command {
        Some(Commands::Destroy { environment }) => {
            if let Err(e) = handle_destroy_command(&environment) {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        }
        None => {
            display_help_info();
        }
    }

    info!("Application finished");
}

/// Display helpful information to the user when no command is provided
fn display_help_info() {
    println!("🏗️  Torrust Tracker Deployer");
    println!("=========================");
    println!();
    println!("This repository provides automated deployment infrastructure for Torrust tracker projects.");
    println!("The infrastructure includes VM provisioning with OpenTofu and configuration");
    println!("management with Ansible playbooks.");
    println!();
    println!("📋 Getting Started:");
    println!("   Please follow the instructions in the README.md file to:");
    println!("   1. Set up the required dependencies (OpenTofu, Ansible, LXD)");
    println!("   2. Provision the deployment infrastructure");
    println!("   3. Deploy and configure the services");
    println!();
    println!("🧪 Running E2E Tests:");
    println!("   Use the e2e tests binaries to run end-to-end tests:");
    println!("   cargo e2e-provision && cargo e2e-config");
    println!();
    println!("📖 For detailed instructions, see: README.md");
    println!();
    println!("💡 To see available commands, run: torrust-tracker-deployer --help");
}

/// Handle the destroy command
///
/// This function orchestrates the environment destruction workflow by:
/// 1. Validating the environment name
/// 2. Loading the environment from persistent storage
/// 3. Executing the destroy command handler
/// 4. Providing user-friendly progress updates
///
/// # Arguments
///
/// * `environment_name` - The name of the environment to destroy
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if:
/// - Environment name is invalid
/// - Environment cannot be loaded
/// - Destruction fails
///
/// # Errors
///
/// This function will return an error if the environment name is invalid,
/// the environment cannot be loaded, or the destruction process fails.
fn handle_destroy_command(environment_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::sync::Arc;
    use std::time::Duration;
    use torrust_tracker_deployer_lib::adapters::tofu::client::OpenTofuClient;
    use torrust_tracker_deployer_lib::application::command_handlers::DestroyCommandHandler;
    use torrust_tracker_deployer_lib::domain::environment::name::EnvironmentName;
    use torrust_tracker_deployer_lib::domain::environment::state::AnyEnvironmentState;
    use torrust_tracker_deployer_lib::infrastructure::persistence::repository_factory::RepositoryFactory;
    use torrust_tracker_deployer_lib::shared::user_output::{UserOutput, VerbosityLevel};

    // Create user output with default stdout/stderr channels
    let mut output = UserOutput::new(VerbosityLevel::Normal);

    // Display initial progress (to stderr)
    output.progress(&format!("Destroying environment '{environment_name}'..."));

    // Validate environment name
    let env_name = EnvironmentName::new(environment_name.to_string()).map_err(|e| {
        output.error(&format!(
            "Invalid environment name '{environment_name}': {e}"
        ));
        format!("Invalid environment name: {e}")
    })?;

    // Build path for the environment
    let build_dir = PathBuf::from("build").join(env_name.as_str());

    // Create OpenTofu client
    let opentofu_client = Arc::new(OpenTofuClient::new(build_dir.join("opentofu")));

    // Create repository for loading environment state
    let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
    let repository = repository_factory.create(PathBuf::from("data"));

    // Load the environment from storage
    let environment = repository.load(&env_name).map_err(|e| {
        output.error(&format!(
            "Failed to load environment '{environment_name}': {e}"
        ));
        format!("Failed to load environment: {e}")
    })?;

    // Check if environment exists
    let environment = environment.ok_or_else(|| {
        output.error(&format!(
            "Environment '{environment_name}' not found. Has it been provisioned?"
        ));
        format!("Environment '{environment_name}' not found")
    })?;

    // Create and execute destroy command handler
    output.progress("Tearing down infrastructure...");

    let command_handler = DestroyCommandHandler::new(opentofu_client, repository);

    // Execute destroy based on the environment's current state
    let _destroyed_env = match environment {
        AnyEnvironmentState::Destroyed(env) => {
            output.warn("Environment is already destroyed");
            Ok(env)
        }
        AnyEnvironmentState::Created(env) => command_handler.execute(env),
        AnyEnvironmentState::Provisioning(env) => command_handler.execute(env),
        AnyEnvironmentState::Provisioned(env) => command_handler.execute(env),
        AnyEnvironmentState::Configuring(env) => command_handler.execute(env),
        AnyEnvironmentState::Configured(env) => command_handler.execute(env),
        AnyEnvironmentState::Releasing(env) => command_handler.execute(env),
        AnyEnvironmentState::Released(env) => command_handler.execute(env),
        AnyEnvironmentState::Running(env) => command_handler.execute(env),
        AnyEnvironmentState::ProvisionFailed(env) => command_handler.execute(env),
        AnyEnvironmentState::ConfigureFailed(env) => command_handler.execute(env),
        AnyEnvironmentState::ReleaseFailed(env) => command_handler.execute(env),
        AnyEnvironmentState::RunFailed(env) => command_handler.execute(env),
    }
    .map_err(|e| {
        output.error(&format!(
            "Failed to destroy environment '{environment_name}': {e}"
        ));
        format!("Destroy command failed: {e}")
    })?;

    output.progress("Cleaning up resources...");
    output.success(&format!(
        "Environment '{environment_name}' destroyed successfully"
    ));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn it_should_parse_destroy_subcommand() {
        let args = vec!["torrust-tracker-deployer", "destroy", "test-env"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.command.is_some());
        match cli.command.unwrap() {
            Commands::Destroy { environment } => {
                assert_eq!(environment, "test-env");
            }
        }
    }

    #[test]
    fn it_should_parse_destroy_with_different_environment_names() {
        let test_cases = vec!["e2e-provision", "production", "test-123", "dev-environment"];

        for env_name in test_cases {
            let args = vec!["torrust-tracker-deployer", "destroy", env_name];
            let cli = Cli::try_parse_from(args).unwrap();

            match cli.command.unwrap() {
                Commands::Destroy { environment } => {
                    assert_eq!(environment, env_name);
                }
            }
        }
    }

    #[test]
    fn it_should_require_environment_parameter() {
        let args = vec!["torrust-tracker-deployer", "destroy"];
        let result = Cli::try_parse_from(args);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_message = error.to_string();
        assert!(
            error_message.contains("required") || error_message.contains("argument"),
            "Error message should indicate missing required argument: {error_message}"
        );
    }

    #[test]
    fn it_should_parse_global_log_options_with_destroy_command() {
        let args = vec![
            "torrust-tracker-deployer",
            "--log-file-format",
            "json",
            "--log-stderr-format",
            "compact",
            "--log-output",
            "file-and-stderr",
            "--log-dir",
            "/tmp/logs",
            "destroy",
            "test-env",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        // Verify the destroy command was parsed correctly
        match cli.command.unwrap() {
            Commands::Destroy { environment } => {
                assert_eq!(environment, "test-env");
            }
        }

        // Log options are set but we don't compare them as they don't implement PartialEq
        assert_eq!(cli.log_dir, PathBuf::from("/tmp/logs"));
    }

    #[test]
    fn it_should_use_default_log_dir_when_not_specified() {
        let args = vec!["torrust-tracker-deployer", "destroy", "test-env"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.log_dir, PathBuf::from("./data/logs"));
    }

    #[test]
    fn it_should_handle_no_command() {
        let args = vec!["torrust-tracker-deployer"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.command.is_none());
    }

    #[test]
    fn it_should_show_help_with_help_flag() {
        let args = vec!["torrust-tracker-deployer", "--help"];
        let result = Cli::try_parse_from(args);

        // Help flag causes a "display help" error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn it_should_show_version_with_version_flag() {
        let args = vec!["torrust-tracker-deployer", "--version"];
        let result = Cli::try_parse_from(args);

        // Version flag causes a "display version" error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    #[test]
    fn it_should_show_destroy_help() {
        let args = vec!["torrust-tracker-deployer", "destroy", "--help"];
        let result = Cli::try_parse_from(args);

        // Help flag causes a "display help" error
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);

        // Verify the help text mentions the environment parameter
        let help_text = error.to_string();
        assert!(
            help_text.contains("environment") || help_text.contains("<ENVIRONMENT>"),
            "Help text should mention environment parameter"
        );
    }
}
