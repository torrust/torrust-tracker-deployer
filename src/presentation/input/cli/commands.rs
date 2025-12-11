//! CLI Command Definitions
//!
//! This module defines the command-line interface structure and available commands
//! for the Torrust Tracker Deployer CLI application.

use clap::Subcommand;

use std::path::PathBuf;

use crate::domain::provider::Provider;

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

    /// Configure a provisioned deployment environment
    ///
    /// This command configures the infrastructure of a provisioned deployment
    /// environment. It will:
    /// - Install Docker engine
    /// - Install Docker Compose
    /// - Configure system services
    ///
    /// The environment must be in "Provisioned" state (use 'provision' command first).
    Configure {
        /// Name of the environment to configure
        ///
        /// The environment name must match an existing environment that was
        /// previously provisioned and is in "Provisioned" state.
        environment: String,
    },

    /// Verify deployment infrastructure
    ///
    /// This command validates that a deployed environment's infrastructure is
    /// properly configured and ready. It will:
    /// - Verify cloud-init completion
    /// - Verify Docker installation
    /// - Verify Docker Compose installation
    ///
    /// The environment must have an instance IP set (use 'provision' command first).
    Test {
        /// Name of the environment to test
        ///
        /// The environment name must match an existing environment that was
        /// previously provisioned and has an instance IP assigned.
        environment: String,
    },

    /// Register an existing instance as an alternative to provisioning
    ///
    /// This command registers an existing VM, physical server, or container
    /// with an environment that was previously created. Instead of provisioning
    /// new infrastructure, it uses the provided IP address to connect to
    /// existing infrastructure.
    ///
    /// The environment must be in "Created" state (use 'create environment' first).
    /// After registration, the environment transitions to "Provisioned" state
    /// and can continue with 'configure', 'release', and 'run' commands.
    ///
    /// Instance Requirements:
    /// - Ubuntu 24.04 LTS
    /// - SSH connectivity with credentials from 'create environment'
    /// - Public SSH key installed for access
    /// - Username with sudo access
    Register {
        /// Name of the environment to register the instance with
        ///
        /// The environment name must match an existing environment that was
        /// previously created and is in "Created" state.
        environment: String,

        /// IP address of the existing instance
        ///
        /// The IP address (IPv4 or IPv6) of the instance to register.
        /// The instance must be reachable via SSH using the credentials
        /// configured in the environment.
        #[arg(long, value_name = "IP_ADDRESS")]
        instance_ip: String,

        /// SSH port for the instance (optional - overrides environment config)
        ///
        /// If not provided, uses the SSH port from the environment configuration.
        /// This is useful when the instance uses a non-standard SSH port,
        /// such as in Docker bridge networking where ports are dynamically mapped.
        #[arg(long, value_name = "PORT")]
        ssh_port: Option<u16>,
    },

    /// Release application files to a configured environment
    ///
    /// This command prepares and transfers application files (docker-compose.yml,
    /// configuration files, etc.) to a configured VM. The environment must be
    /// in the "Configured" state.
    ///
    /// After successful release:
    /// - Docker compose files are copied to /opt/torrust/ on the VM
    /// - Environment transitions to "Released" state
    /// - You can then run `run <environment>` to start the services
    ///
    /// # Examples
    ///
    /// ```text
    /// torrust-tracker-deployer release my-env
    /// torrust-tracker-deployer release production
    /// ```
    Release {
        /// Name of the environment to release to
        ///
        /// The environment name must match an existing environment that was
        /// previously configured and is in "Configured" state.
        environment: String,
    },

    /// Run the application stack on a released environment
    ///
    /// This command starts the docker compose services on a released VM.
    /// The environment must be in the "Released" state.
    ///
    /// After successful run:
    /// - Docker containers are started via 'docker compose up -d'
    /// - Environment transitions to "Running" state
    /// - Services are accessible on the VM
    ///
    /// # Examples
    ///
    /// ```text
    /// torrust-tracker-deployer run my-env
    /// torrust-tracker-deployer run production
    /// ```
    Run {
        /// Name of the environment to run
        ///
        /// The environment name must match an existing environment that was
        /// previously released and is in "Released" state.
        environment: String,
    },
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

        /// Provider to generate template for (required)
        ///
        /// Available providers:
        /// - lxd: Local LXD provider for development and testing
        /// - hetzner: Hetzner Cloud provider for production deployments
        #[arg(long, short = 'p', value_enum)]
        provider: Provider,
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
