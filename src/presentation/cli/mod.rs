//! CLI Module
//!
//! This module provides the command-line interface structure and functionality
//! for the Torrust Tracker Deployer application. It handles CLI argument parsing
//! and provides the CLI data structures.

use clap::Parser;

// Re-export submodules for convenient access
pub mod args;
pub mod commands;

pub use args::GlobalArgs;
pub use commands::Commands;

/// Command-line interface for Torrust Tracker Deployer
///
/// This struct defines the top-level CLI structure including global arguments
/// and available subcommands. It uses clap for argument parsing and provides
/// comprehensive help documentation.
#[derive(Parser, Debug)]
#[command(name = "torrust-tracker-deployer")]
#[command(about = "Automated deployment infrastructure for Torrust Tracker")]
#[command(version)]
#[allow(clippy::struct_field_names)] // CLI arguments intentionally share 'log_' prefix for clarity
pub struct Cli {
    /// Global arguments (logging configuration)
    #[command(flatten)]
    pub global: GlobalArgs,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_parse_destroy_subcommand() {
        let args = vec!["torrust-tracker-deployer", "destroy", "test-env"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.command.is_some());
        match cli.command.unwrap() {
            Commands::Destroy { environment } => {
                assert_eq!(environment, "test-env");
            }
            Commands::Create { .. } => panic!("Expected Destroy command"),
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
                Commands::Create { .. } => panic!("Expected Destroy command"),
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
            Commands::Create { .. } => panic!("Expected Destroy command"),
        }

        // Log options are set but we don't compare them as they don't implement PartialEq
        assert_eq!(cli.global.log_dir, std::path::PathBuf::from("/tmp/logs"));
    }

    #[test]
    fn it_should_use_default_log_dir_when_not_specified() {
        let args = vec!["torrust-tracker-deployer", "destroy", "test-env"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.global.log_dir, std::path::PathBuf::from("./data/logs"));
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

    #[test]
    fn it_should_parse_create_subcommand() {
        let args = vec![
            "torrust-tracker-deployer",
            "create",
            "--env-file",
            "config.json",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert!(cli.command.is_some());
        match cli.command.unwrap() {
            Commands::Create { env_file } => {
                assert_eq!(env_file, std::path::PathBuf::from("config.json"));
            }
            Commands::Destroy { .. } => panic!("Expected Create command"),
        }
    }

    #[test]
    fn it_should_parse_create_with_short_flag() {
        let args = vec!["torrust-tracker-deployer", "create", "-f", "env.json"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command.unwrap() {
            Commands::Create { env_file } => {
                assert_eq!(env_file, std::path::PathBuf::from("env.json"));
            }
            Commands::Destroy { .. } => panic!("Expected Create command"),
        }
    }

    #[test]
    fn it_should_require_env_file_parameter_for_create() {
        let args = vec!["torrust-tracker-deployer", "create"];
        let result = Cli::try_parse_from(args);

        assert!(result.is_err());
        let error = result.unwrap_err();
        let error_message = error.to_string();
        assert!(
            error_message.contains("required") || error_message.contains("--env-file"),
            "Error message should indicate missing required --env-file: {error_message}"
        );
    }

    #[test]
    fn it_should_parse_working_dir_global_option() {
        let args = vec![
            "torrust-tracker-deployer",
            "--working-dir",
            "/tmp/workspace",
            "create",
            "--env-file",
            "config.json",
        ];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(
            cli.global.working_dir,
            std::path::PathBuf::from("/tmp/workspace")
        );

        match cli.command.unwrap() {
            Commands::Create { env_file } => {
                assert_eq!(env_file, std::path::PathBuf::from("config.json"));
            }
            Commands::Destroy { .. } => panic!("Expected Create command"),
        }
    }

    #[test]
    fn it_should_use_default_working_dir_when_not_specified() {
        let args = vec!["torrust-tracker-deployer", "create", "-f", "config.json"];
        let cli = Cli::try_parse_from(args).unwrap();

        assert_eq!(cli.global.working_dir, std::path::PathBuf::from("."));
    }

    #[test]
    fn it_should_show_create_help() {
        let args = vec!["torrust-tracker-deployer", "create", "--help"];
        let result = Cli::try_parse_from(args);

        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);

        let help_text = error.to_string();
        assert!(
            help_text.contains("env-file") || help_text.contains("configuration"),
            "Help text should mention env-file parameter"
        );
    }
}
