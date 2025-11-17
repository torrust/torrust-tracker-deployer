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
    /// Create operations (environment creation or template generation)
    ///
    /// This command provides subcommands for creating environments and generating
    /// configuration templates.
    Create {
        #[command(subcommand)]
        action: CreateAction,
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

    /// Provision a new deployment environment infrastructure
    ///
    /// This command provisions the virtual machine infrastructure for a deployment
    /// environment that was previously created. It will:
    /// - Render and apply `OpenTofu` templates
    /// - Create LXD VM instances
    /// - Configure networking
    /// - Wait for SSH connectivity
    /// - Wait for cloud-init completion
    ///
    /// The environment must be in "Created" state (use 'create environment' first).
    Provision {
        /// Name of the environment to provision
        ///
        /// The environment name must match an existing environment that was
        /// previously created and is in "Created" state.
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

/// Actions available for the create command
#[derive(Debug, Subcommand)]
pub enum CreateAction {
    /// Create environment from configuration file
    ///
    /// This subcommand creates a new deployment environment based on a
    /// configuration file. The configuration file specifies the environment
    /// name, SSH credentials, and other settings required for creation.
    Environment {
        /// Path to the environment configuration file
        ///
        /// The configuration file must be in JSON format and contain all
        /// required fields for environment creation.
        #[arg(long, short = 'f', value_name = "FILE")]
        env_file: PathBuf,
    },

    /// Generate template configuration file
    ///
    /// This subcommand generates a JSON configuration template file with
    /// placeholder values. Edit the template to provide your actual
    /// configuration values, then use 'create environment' to create
    /// the environment.
    Template {
        /// Output path for the template file (optional)
        ///
        /// If not provided, creates environment-template.json in the
        /// current working directory. Parent directories will be created
        /// automatically if they don't exist.
        #[arg(value_name = "PATH")]
        output_path: Option<PathBuf>,
    },
}

impl CreateAction {
    /// Get the default template output path
    #[must_use]
    pub fn default_template_path() -> PathBuf {
        PathBuf::from("environment-template.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_use_default_template_path() {
        let default_path = CreateAction::default_template_path();
        assert_eq!(default_path, PathBuf::from("environment-template.json"));
    }
}
