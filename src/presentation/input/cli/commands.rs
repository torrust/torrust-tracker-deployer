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
    ///
    /// WHAT GETS DESTROYED:
    ///   • VM instance (deleted permanently)
    ///   • Virtual networks (removed)
    ///   • Remote data (tracker database, logs, configs on VM)
    ///   • Running services (stopped and removed)
    ///
    /// WHAT GETS PRESERVED:
    ///   • Local data directory: data/{env-name}/ (use 'purge' to remove)
    ///   • Build artifacts: build/{env-name}/ (use 'purge' to remove)
    ///   • Environment state file (allows reusing the name after purge)
    ///
    /// SAFETY WARNINGS:
    ///   • Operation is IRREVERSIBLE - infrastructure cannot be recovered
    ///   • Remote data (tracker database) will be permanently lost
    ///   • Always backup important data before destroying
    ///
    /// NEXT STEPS:
    ///   After destroying, you can:
    ///   • Purge local data to reuse environment name: purge {env-name}
    ///   • Keep local data for reference or audit trail
    ///
    /// EXECUTION TIME:
    ///   Typical duration: 1-3 minutes
    ///   Factors: provider API response, resource cleanup timing
    Destroy {
        /// Name of the environment to destroy
        ///
        /// The environment name must be a valid identifier that was previously
        /// created through the provision command.
        environment: String,
    },

    /// Purge local data for an environment
    ///
    /// This command removes all local data directories for an environment,
    /// including the data/{env-name}/ and build/{env-name}/ directories.
    ///
    /// WORKFLOW POSITION (Step 9 - After destroy, optional):
    ///   ... → destroy → \[PURGE\] (optional cleanup to reuse environment name)
    ///
    /// COMPARISON WITH DESTROY:
    ///   • destroy: Removes REMOTE infrastructure (VMs, cloud resources)
    ///   • purge:   Removes LOCAL data (state files, build artifacts)
    ///   Typical sequence: destroy (step 8) → purge (step 9, optional)
    ///
    /// WHEN TO USE:
    ///   • After destroying an environment to reuse the environment name
    ///   • To free up disk space from environments no longer needed
    ///   • To clean up state when infrastructure was destroyed independently
    ///
    /// WHAT GETS REMOVED:
    ///   • data/{env-name}/environment.json (state file)
    ///   • data/{env-name}/logs/ (execution logs)
    ///   • build/{env-name}/ (rendered templates, Terraform state)
    ///   After purge, the environment name becomes available for reuse
    ///
    /// SAFETY WARNINGS:
    ///   • Always prompts for confirmation unless --force is provided
    ///   • Operation is IRREVERSIBLE - local data permanently deleted
    ///   • For running environments: only removes LOCAL data, does NOT destroy infrastructure
    ///   • Best practice: only purge after destroy completes successfully
    ///
    /// EXAMPLES:
    ///   After destroying an environment:
    ///     torrust-tracker-deployer purge my-env
    ///
    ///   Skip confirmation (for automation/scripts):
    ///     torrust-tracker-deployer purge my-env --force
    Purge {
        /// Name of the environment to purge
        ///
        /// The environment name must match an existing environment in the
        /// local data directory.
        environment: String,

        /// Skip confirmation prompt
        ///
        /// When provided, the purge operation proceeds without asking for
        /// user confirmation. Use with caution, especially for non-destroyed
        /// environments.
        #[arg(short, long)]
        force: bool,
    },

    /// Provision a new deployment environment infrastructure
    ///
    /// This command provisions the virtual machine infrastructure for a deployment
    /// environment that was previously created. It will:
    /// - Render and apply `OpenTofu` templates
    /// - Create VM instances
    /// - Configure networking
    /// - Wait for SSH connectivity
    /// - Wait for cloud-init completion
    ///
    /// The environment must be in "Created" state (use 'create environment' first).
    ///
    /// STATE TRANSITION:
    ///   • Prerequisites: Environment must be in Created state
    ///     - Use 'create environment' first
    ///     - Check state: show {env-name}
    ///   • After Success: Environment transitions to Provisioned state
    ///   • Infrastructure Created: VM instance, networking, SSH access
    ///   • On Failure: Remains in Created state
    ///
    /// WORKFLOW POSITION (Step 2 of 8):
    ///   create environment → \[PROVISION\] → configure → release → run
    ///                             ↓
    ///                      (alternative: register)
    ///
    /// NEXT STEPS:
    ///   After provisioning:
    ///   1. Verify infrastructure (optional): test {env-name}
    ///   2. Configure the instance: configure {env-name}
    ///
    /// ALTERNATIVE: Register Existing Infrastructure
    ///   If you already have a server/VM, use 'register' instead:
    ///   register {env-name} --instance-ip \<IP\>
    ///   This skips infrastructure provisioning.
    ///
    /// COMMON ERRORS:
    ///   • "Environment not in Created state": Run 'create environment' first
    ///   • "Provider credentials missing": Check environment config file
    ///   • "SSH connection failed": Verify network connectivity
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
    ///
    /// STATE TRANSITION:
    ///   • Prerequisites: Environment must be in Provisioned state
    ///     - Use 'provision' or 'register' first
    ///   • After Success: Environment transitions to Configured state
    ///   • Configuration Applied: Docker, Docker Compose, system packages, firewall
    ///   • On Failure: Remains in Provisioned state
    ///
    /// WORKFLOW POSITION (Step 3 of 8):
    ///   provision/register → \[CONFIGURE\] → release → run
    ///
    /// WHAT THIS COMMAND DOES NOT DO:
    ///   • Does not deploy application files (use 'release')
    ///   • Does not start services (use 'run')
    ///   • Does not provision infrastructure (use 'provision'/'register' first)
    ///
    /// EXECUTION TIME:
    ///   Typical duration: 2-5 minutes
    ///   Factors: network speed, package downloads, instance specifications
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
    ///
    /// WHAT GETS VALIDATED:
    ///   • Cloud-init completion (system initialization finished)
    ///   • Docker engine installed and running
    ///   • Docker Compose installed and accessible
    ///   • SSH connectivity to instance
    ///
    /// WHEN TO RUN (Recommended Checkpoints):
    ///   • After 'provision' - verify infrastructure is ready
    ///   • After 'register' - verify existing instance meets requirements
    ///   • Before 'configure' - confirm base system is operational
    ///   • Troubleshooting - diagnose infrastructure issues
    ///
    /// EXIT CODES:
    ///   • 0: All checks passed - infrastructure ready
    ///   • Non-zero: One or more checks failed - see output for details
    ///
    /// WHAT THIS DOES NOT VALIDATE:
    ///   • Application deployment (use after 'release' to check that)
    ///   • Running services (use 'show' after 'run' to check that)
    ///   • Configuration file syntax (use 'validate' for that)
    Test {
        /// Name of the environment to test
        ///
        /// The environment name must match an existing environment that was
        /// previously provisioned and has an instance IP assigned.
        environment: String,
    },

    /// Validate environment configuration without deployment
    ///
    /// This command validates an environment configuration file without
    /// executing any deployment operations. It performs comprehensive
    /// validation including:
    /// - JSON schema compliance
    /// - Environment name format
    /// - Provider configuration validity
    /// - SSH key file existence
    /// - Domain name format (if configured)
    /// - Port number ranges
    ///
    /// This is a dry-run command useful for:
    /// - Verifying configuration before creating environments
    /// - AI agents validating user inputs
    /// - CI/CD pipelines checking configurations
    /// - Troubleshooting configuration issues
    ///
    /// PRE-DEPLOYMENT USAGE:
    ///   Always validate BEFORE 'create environment' to catch errors early
    ///   Saves time by detecting issues before resource provisioning
    ///
    /// WHAT GETS VALIDATED:
    ///   • JSON syntax and schema compliance
    ///   • Environment name (format, length, characters)
    ///   • Provider configuration (type, credentials structure)
    ///   • SSH credentials (key file paths exist, format valid)
    ///   • Network settings (port ranges, IP formats)
    ///   • Tracker configuration (modes, database type)
    ///
    /// BENEFITS OF EARLY VALIDATION:
    ///   • Fast feedback (no network calls or resource creation)
    ///   • Clear error messages with specific field issues
    ///   • Prevents partial deployments from invalid configs
    ///   • Saves time and costs (catch before provisioning)
    ///
    /// EXAMPLE WORKFLOW INTEGRATION:
    ///   1. Create template: create template --provider \<type\>
    ///   2. Edit configuration: vim environment-template.json
    ///   3. Validate config: validate --env-file environment-template.json
    ///   4. Create environment: create environment --env-file environment-template.json
    ///
    /// EXAMPLES:
    ///   torrust-tracker-deployer validate --env-file envs/my-config.json
    ///   torrust-tracker-deployer validate -f production.json
    Validate {
        /// Path to the environment configuration file
        ///
        /// The configuration file must be in JSON format. The file will be
        /// validated against the environment configuration schema.
        #[arg(long, short = 'f', value_name = "FILE")]
        env_file: PathBuf,
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
    /// STATE TRANSITION:
    ///   • Prerequisites: Environment must be in Created state
    ///   • After Success: Environment transitions to Provisioned state
    ///   • Infrastructure: Existing instance registered (not created)
    ///   • On Failure: Remains in Created state
    ///
    /// WORKFLOW POSITION (Alternative to Step 2):
    ///   create environment → \[REGISTER\] → configure → release → run
    ///                    ↓
    ///             (alternative: provision)
    ///
    /// WHEN TO USE REGISTER VS PROVISION:
    ///   Use REGISTER when:
    ///   • You have an existing server/VM
    ///   • You want to use bare-metal hardware
    ///   • You already provisioned infrastructure externally
    ///   • You're deploying to a Docker container (for testing)
    ///
    ///   Use PROVISION when:
    ///   • You want automated infrastructure creation
    ///   • You're using supported cloud providers
    ///   • You want reproducible infrastructure
    ///
    /// INSTANCE REQUIREMENTS:
    ///   • Ubuntu 24.04 LTS
    ///   • SSH connectivity with credentials from 'create environment'
    ///   • Public SSH key installed for access
    ///   • Username with sudo access
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
    /// STATE TRANSITION:
    ///   • Prerequisites: Environment must be in Configured state
    ///   • After Success: Environment transitions to Released state
    ///   •  Files Deployed:
    ///     - docker-compose.yml to /opt/torrust/
    ///     - Tracker configuration to /opt/torrust/storage/tracker/etc/
    ///     - Environment variables to /opt/torrust/.env
    ///     - Monitoring configs (if enabled)
    ///   • On Failure: Remains in Configured state
    ///
    /// WORKFLOW POSITION (Step 4 of 8):
    ///   configure → \[RELEASE\] → run
    ///
    /// WHAT THIS DOES NOT DO:
    ///   • Does not start containers (use 'run')
    ///   • Does not install Docker (done in 'configure')
    ///   • Does not provision infrastructure (done in 'provision')
    ///
    /// EXAMPLES:
    ///   torrust-tracker-deployer release my-env
    ///   torrust-tracker-deployer release production
    Release {
        /// Name of the environment to release to
        ///
        /// The environment name must match an existing environment that was
        /// previously configured and is in "Configured" state.
        environment: String,
    },

    /// Generate deployment artifacts without executing deployment
    ///
    /// This command generates all deployment artifacts (docker-compose files,
    /// tracker configuration, Ansible playbooks, etc.) to the build directory
    /// without executing any deployment operations.
    ///
    /// NOT PART OF NORMAL DEPLOYMENT WORKFLOW:
    ///   This is a preview/debugging command. Normal workflow automatically
    ///   generates artifacts during provision/configure/release commands.
    ///
    /// COMPARISON WITH NORMAL WORKFLOW:
    ///   Normal: create → provision (generates artifacts + creates infrastructure)
    ///   Render: create → render (generates artifacts only, no infrastructure)
    ///   Use render when you want to inspect what will be deployed before
    ///   committing to infrastructure provisioning.
    ///
    /// TWO MODES:
    ///   1. From existing environment (--env-name):
    ///      Read-only preview of what would be deployed
    ///      Works at any state, does not modify environment
    ///
    ///   2. From configuration file (--env-file):
    ///      Generate artifacts without creating environment
    ///      Useful for validating configurations or generating examples
    ///
    /// USE CASES:
    ///   • Preview artifacts before provisioning infrastructure
    ///   • Inspect what will be deployed before committing to provision
    ///   • Generate artifacts for documentation or examples
    ///   • Test configuration changes without affecting environment
    ///
    /// OUTPUT DIRECTORY:
    ///   Must be different from standard build/{env}/ directory to prevent
    ///   conflicts with real deployments. Use custom path like ./preview/
    ///
    /// EXAMPLES:
    ///   Generate from existing environment:
    ///     torrust-tracker-deployer render --env-name my-env --instance-ip 10.0.0.1 --output-dir ./preview
    ///
    ///   Generate from config file (no environment creation):
    ///     torrust-tracker-deployer render --env-file envs/my-config.json --instance-ip 10.0.0.1 --output-dir /tmp/artifacts
    ///
    ///   Overwrite existing output directory:
    ///     torrust-tracker-deployer render --env-name my-env --instance-ip 10.0.0.1 --output-dir ./preview --force
    Render {
        /// Name of existing environment (mutually exclusive with --env-file)
        ///
        /// Generate artifacts from an existing environment at any state.
        /// This is a read-only operation that does not modify environment state.
        #[arg(long, group = "input", conflicts_with = "env_file")]
        env_name: Option<String>,

        /// Path to environment configuration file (mutually exclusive with --env-name)
        ///
        /// Generate artifacts directly from a configuration file without
        /// creating an environment.
        #[arg(long, short = 'f', group = "input", conflicts_with = "env_name")]
        env_file: Option<PathBuf>,

        /// Target instance IP address (REQUIRED)
        ///
        /// IP address of the target server where artifacts will be deployed.
        /// The IP will be used in generated Ansible inventory and configuration files.
        ///
        /// This allows previewing artifacts for different target IPs before
        /// committing to infrastructure provisioning.
        #[arg(long, value_name = "IP_ADDRESS", required = true)]
        instance_ip: String,

        /// Output directory for generated artifacts (REQUIRED)
        ///
        /// Directory where all deployment artifacts will be written.
        /// Must be different from the standard build/{env}/ directory used by
        /// provision to prevent artifact conflicts and data loss.
        ///
        /// The directory must not exist unless --force is provided.
        #[arg(long, short = 'o', value_name = "PATH", required = true)]
        output_dir: PathBuf,

        /// Overwrite existing output directory
        ///
        /// If the output directory already exists, this flag allows overwriting
        /// its contents. Without this flag, the command will fail if the
        /// directory exists.
        #[arg(long, default_value_t = false)]
        force: bool,
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
    /// STATE TRANSITION:
    ///   • Prerequisites: Environment must be in Released state
    ///   • After Success: Environment transitions to Running state
    ///   • Services Started: Tracker (UDP/HTTP), Database, Prometheus, Grafana
    ///   • On Failure: Remains in Released state
    ///
    /// WORKFLOW POSITION (Step 5 of 8 - Final deployment step):
    ///   release → \[RUN\] → (services running)
    ///
    /// SERVICE ACCESS:
    ///   Once running, services are accessible at:
    ///   • Tracker UDP: <udp://{instance-ip}:6969> (if enabled)
    ///   • Tracker HTTP: <http://{instance-ip}:7070> (if enabled)
    ///   • Tracker API: <http://{instance-ip}:1212> (if enabled)
    ///   • Grafana: <http://{instance-ip}:3000> (if enabled)
    ///   • Prometheus: <http://{instance-ip}:9090> (if enabled)
    ///
    /// VERIFYING SERVICES:
    ///   Check services after running:
    ///   - View environment status: show {env-name}
    ///   - SSH to check containers: ssh -i ~/.ssh/key user@instance-ip
    ///   - Check container status: docker compose ps
    ///   - View logs: docker compose logs tracker
    ///
    /// EXAMPLES:
    ///   torrust-tracker-deployer run my-env
    ///   torrust-tracker-deployer run production
    Run {
        /// Name of the environment to run
        ///
        /// The environment name must match an existing environment that was
        /// previously released and is in "Released" state.
        environment: String,
    },

    /// Show environment information with state-aware details
    ///
    /// This command displays a read-only view of stored environment data
    /// without remote verification, making it fast and reliable.
    ///
    /// The output includes:
    /// - Environment name and current state
    /// - Provider information
    /// - Infrastructure details (IP, SSH credentials) when provisioned
    /// - Service URLs when running
    /// - Next-step guidance based on current state
    ///
    /// STATE-DEPENDENT INFORMATION:
    ///   Created state: Shows environment name, provider, config
    ///   Provisioned state: Adds instance IP, SSH details
    ///   Configured state: Adds system configuration status
    ///   Released state: Adds deployment file locations
    ///   Running state: Adds service URLs and access information
    ///
    /// COMMON USAGE SCENARIOS:
    ///   • Check current state before running next command
    ///   • Get instance IP for SSH access
    ///   • Get service URLs after deployment
    ///   • Verify environment exists before operations
    ///   • Quick status check without network calls
    ///
    /// OUTPUT FORMAT OPTIONS:
    ///   Use --output-format json for machine-readable output
    ///   Default: Human-readable text with tables
    ///
    /// PERFORMANCE NOTE:
    ///   Fast operation - reads local state file only (no network calls)
    ///
    /// EXAMPLES:
    ///   torrust-tracker-deployer show my-env
    ///   torrust-tracker-deployer show production
    Show {
        /// Name of the environment to show
        ///
        /// The environment name must match an existing environment.
        environment: String,
    },

    /// List all environments in the deployment workspace
    ///
    /// This command provides a quick overview of all environments with their
    /// names, states, and providers. It scans the local data directory and
    /// does not make any network calls.
    ///
    /// NOT PART OF DEPLOYMENT WORKFLOW:
    ///   This is an informational command that can be run at any time to
    ///   see what environments exist and their current states.
    ///
    /// OUTPUT INFORMATION:
    ///   For each environment, displays:
    ///   • Environment name
    ///   • Current state (Created, Provisioned, Configured, Released, Running, Destroyed)
    ///   • Provider type (generic, e.g., LXD, Hetzner)
    ///   • Creation timestamp
    ///
    /// WHEN TO USE:
    ///   • Check which environments exist before creating a new one
    ///   • Verify environment states before running commands
    ///   • Quick audit of all deployment environments
    ///   • See what can be purged to free up space
    ///
    /// PERFORMANCE:
    ///   Fast operation - only reads local JSON files, no network calls
    ///
    /// EXAMPLE:
    ///   torrust-tracker-deployer list
    List,

    /// Generate CLI documentation in JSON format
    ///
    /// This command generates machine-readable documentation for all CLI
    /// commands, arguments, and their descriptions. The output is a structured
    /// JSON document suitable for AI agents, documentation generators, and
    /// IDE integrations.
    ///
    /// NOT PART OF DEPLOYMENT WORKFLOW:
    ///   This is a meta-command for generating documentation. It's used by
    ///   maintainers and AI agents, not part of normal deployment operations.
    ///
    /// OUTPUT FORMAT OPTIONS:
    ///   • No path: Outputs JSON to stdout (pipeable)
    ///   • With path: Writes JSON to file
    ///
    /// WHEN TO REGENERATE:
    ///   After modifying command documentation in source code:
    ///   • Added/changed command descriptions
    ///   • Updated argument help text
    ///   • Modified command structure
    ///   Regeneration ensures JSON docs stay in sync with CLI
    ///
    /// USAGE SCENARIOS:
    ///   • AI agents: Read command descriptions and argument details
    ///   • Documentation sites: Generate command reference automatically
    ///   • IDE plugins: Provide autocomplete and inline help
    ///   • Shell completion: Generate dynamic completion scripts
    ///
    /// FORMAT DETAILS:
    ///   JSON includes for each command:
    ///   • Name and description
    ///   • All arguments with types and help text
    ///   • Subcommands (if applicable)
    ///   • Default values and constraints
    ///
    /// EXAMPLES:
    ///   Generate to stdout:
    ///     torrust-tracker-deployer docs
    ///
    ///   Save to file:
    ///     torrust-tracker-deployer docs docs/cli/commands.json
    ///
    ///   Pipe to other tools:
    ///     torrust-tracker-deployer docs | jq '.cli.commands[] | .name'
    Docs {
        /// Output path for CLI documentation file (optional)
        ///
        /// If not provided, documentation is written to stdout.
        ///
        /// Recommended location: `docs/cli/commands.json`
        #[arg(value_name = "PATH")]
        output_path: Option<PathBuf>,
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
    ///
    /// STATE TRANSITION:
    ///   • Prerequisites: None (first command in workflow)
    ///   • After Success: Environment transitions to Created state
    ///   • Creates: data/{env-name}/ and build/{env-name}/ directories
    ///
    /// WORKFLOW POSITION (Step 1 of 8):
    ///   [CREATE ENVIRONMENT] → provision/register → configure → release → run
    ///
    /// NEXT STEPS:
    ///   After creating an environment, choose one:
    ///   1. Provision new infrastructure: provision {env-name}
    ///   2. Register existing infrastructure: register {env-name} --instance-ip \<IP\>
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
    ///
    /// WORKFLOW POSITION (Step 0 - Before everything):
    ///   [CREATE TEMPLATE] → edit → validate → create environment → provision → ...
    ///
    /// AVAILABLE PROVIDERS:
    ///   Templates are provider-specific and include appropriate defaults:
    ///   • Local VM providers (e.g., LXD) - for development/testing
    ///   • Cloud providers (e.g., Hetzner) - for production deployments
    ///   Each provider template includes provider-specific configuration fields
    ///
    /// CUSTOMIZATION REQUIRED:
    ///   Generated templates have placeholder values that MUST be edited:
    ///   • Environment name (must be unique)
    ///   • SSH credentials (username, key paths)
    ///   • Provider settings (varies by provider)
    ///   • Tracker configuration (UDP/HTTP ports, database type)
    ///   • Optional: monitoring, backup, HTTPS settings
    ///
    /// NEXT STEPS:
    ///   1. Generate template: create template --provider \<type\>
    ///   2. Edit template: vim environment-template.json
    ///   3. Validate config: validate --env-file environment-template.json
    ///   4. Create environment: create environment --env-file environment-template.json
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

    /// Generate JSON Schema for environment configuration
    ///
    /// This subcommand generates a JSON Schema that describes the structure
    /// and validation rules for environment configuration files. The schema
    /// can be used by IDEs, editors, and AI assistants for autocomplete,
    /// validation, and inline documentation.
    ///
    /// NOT PART OF DEPLOYMENT WORKFLOW:
    ///   This is a meta-command for IDE integration. Generate once and
    ///   configure your editor to use it for environment JSON files.
    ///
    /// IDE INTEGRATION BENEFITS:
    ///   • Autocomplete: Suggestions for configuration fields as you type
    ///   • Validation: Real-time error checking for invalid values
    ///   • Documentation: Inline help text for each configuration option
    ///   • Type checking: Ensures correct data types (strings, numbers, booleans)
    ///
    /// SETUP INSTRUCTIONS:
    ///   1. Generate schema:
    ///      torrust-tracker-deployer create schema schemas/environment-config.json
    ///
    ///   2. Configure your IDE:
    ///      VS Code: Add to settings.json:
    ///      "json.schemas": [{
    ///      "fileMatch": ["envs/*.json"],
    ///      "url": "./schemas/environment-config.json"
    ///      }]
    ///
    ///      `JetBrains` IDEs: File → Settings → Languages & Frameworks →
    ///      Schemas and DTDs → JSON Schema Mappings
    ///
    /// WHEN TO REGENERATE:
    ///   After modifying configuration structure in source code:
    ///   • Added new configuration fields
    ///   • Changed validation rules or constraints
    ///   • Updated field descriptions
    ///
    /// USAGE SCENARIOS:
    ///   • First-time setup: Enable IDE autocomplete for config files
    ///   • After updates: Regenerate to sync with code changes
    ///   • CI/CD: Validate configs against schema in automated tests
    ///   • AI agents: Provide schema for better config generation
    Schema {
        /// Output path for the schema file (optional)
        ///
        /// If not provided, outputs the schema to stdout.
        /// Parent directories will be created automatically if they don't exist.
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
