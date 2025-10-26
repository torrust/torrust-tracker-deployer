//! CLI Command Definitions
//!
//! This module defines the command-line interface structure and available commands
//! for the Torrust Tracker Deployer CLI application.

use clap::Subcommand;

use std::path::PathBuf;

/// Available CLI commands
///
/// This enum defines all the subcommands available in the CLI application.
/// Each variant represents a specific operation that can be performed.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create a new deployment environment
    ///
    /// This command creates a new environment based on a configuration file.
    /// The configuration file specifies the environment name, SSH credentials,
    /// and other settings required for environment creation.
    Create {
        /// Path to the environment configuration file
        ///
        /// The configuration file must be in JSON format and contain all
        /// required fields for environment creation. Use --help for more
        /// information about the configuration format.
        #[arg(long, short = 'f', value_name = "FILE")]
        env_file: PathBuf,
    },

    /// Destroy an existing deployment environment
    ///
    /// This command will tear down all infrastructure associated with the
    /// specified environment, including virtual machines, networks, and
    /// persistent data. This operation is irreversible.
    Destroy {
        /// Name of the environment to destroy
        ///
        /// The environment name must be a valid identifier that was previously
        /// created through the provision command.
        environment: String,
    },
    // Future commands will be added here:
    //
    // /// Provision a new deployment environment
    // Provision {
    //     /// Name of the environment to create
    //     environment: String,
    //     /// Infrastructure provider to use (lxd, multipass, etc.)
    //     #[arg(long, default_value = "lxd")]
    //     provider: String,
    // },
    //
    // /// Configure an existing deployment environment
    // Configure {
    //     /// Name of the environment to configure
    //     environment: String,
    // },
    //
    // /// Create a new release of the deployed application
    // Release {
    //     /// Name of the environment for the release
    //     environment: String,
    //     /// Version tag for the release
    //     version: String,
    // },
}
