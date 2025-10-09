# Structured Logging Guide

This guide explains the structured logging implementation in the Torrust Tracker Deploy project, which uses hierarchical structured logging.

## JSON Output Format

When using `logging::init_json()` or `LogFormat::Json`, logs are output in JSON format suitable for log aggregation:

````json
{
  "timestamp": "2024-09-16T17:00:00.000Z",
  "level": "INFO",
  "fields": {
    "command_type": "provision"
  },
  "target": "torrust_tracker_deploy::commands::provision",
  "span": {
    "name": "provision_command"
  },
  "spans": [{ "name": "provision_command", "command_type": "provision" }]
}
```ns to mirror our three-level architecture.

## Architecture Overview

Our structured logging follows a three-level hierarchy that mirrors the application architecture:

```text
Level 1: Commands (Top-level orchestration)
├── Level 2: Steps (Mid-level execution units)
│   └── Level 3: Remote Actions (Leaf-level operations)
````

## Setting Up Logging

### Basic Setup (Recommended)

```rust
use torrust_tracker_deploy::logging;

fn main() {
    // Initialize pretty-printed logging for development
    logging::init();

    // Your application code here...
}
```

### Alternative Formats

```rust
use torrust_tracker_deploy::logging;

fn main() {
    // For production (JSON output)
    logging::init_json();

    // For compact output
    logging::init_compact();

    // Using the format helper
    logging::init_with_format(&LogFormat::Json);
}
```

### CLI Applications with Format Selection

For CLI applications that want to support multiple formats:

```rust
use torrust_tracker_deploy::logging::{self, LogFormat};
use clap::Parser;

#[derive(Parser)]
struct Cli {
    #[arg(long, default_value = "pretty")]
    log_format: LogFormat,
}

fn main() {
    let cli = Cli::parse();
    logging::init_with_format(&cli.log_format);

    // Your application code...
}
```

## Span Hierarchy Examples

When you execute operations, you'll see nested spans in your logs:

### Example: Provision Command

```text
2024-09-16T17:00:00.000Z TRACE provision_command: Starting infrastructure provisioning
2024-09-16T17:00:00.100Z TRACE provision_command:render_opentofu_templates: Rendering OpenTofu templates
2024-09-16T17:00:00.200Z TRACE provision_command:initialize_infrastructure: Initializing infrastructure
2024-09-16T17:00:00.300Z TRACE provision_command:plan_infrastructure: Planning infrastructure deployment
2024-09-16T17:00:00.400Z TRACE provision_command:apply_infrastructure: Applying infrastructure changes
2024-09-16T17:00:00.500Z TRACE provision_command:get_instance_info: Retrieving instance information
2024-09-16T17:00:00.600Z  INFO provision_command: Infrastructure provisioned successfully
```

### Example: Configure Command with Remote Actions

```text
2024-09-16T17:01:00.000Z TRACE configure_command: Starting system configuration
2024-09-16T17:01:00.100Z TRACE configure_command:render_ansible_templates: Rendering Ansible templates
2024-09-16T17:01:00.200Z TRACE configure_command:wait_ssh_connectivity: Waiting for SSH connectivity
2024-09-16T17:01:00.300Z TRACE configure_command:wait_cloud_init: Waiting for cloud-init completion
2024-09-16T17:01:00.400Z TRACE configure_command:wait_cloud_init:cloud_init_validation: Validating cloud-init status
2024-09-16T17:01:00.500Z TRACE configure_command:install_docker: Installing Docker
2024-09-16T17:01:00.600Z TRACE configure_command:validate_docker_installation: Validating Docker
2024-09-16T17:01:00.700Z TRACE configure_command:validate_docker_installation:docker_validation: Checking Docker version
2024-09-16T17:01:00.800Z  INFO configure_command: System configured successfully
```

## Span Fields Reference

### Command Level (Level 1)

- **command_type**: The type of command being executed
  - Values: `"provision"`, `"configure"`, `"test"`

### Step Level (Level 2)

- **step_type**: The category of step being executed
  - Values: `"infrastructure"`, `"rendering"`, `"connectivity"`, `"system"`, `"software"`, `"validation"`
- **operation**: The specific operation being performed
  - Examples: `"init"`, `"plan"`, `"apply"`, `"info"`
- **template_type**: For rendering steps
  - Values: `"opentofu"`, `"ansible"`
- **component**: For software/validation steps
  - Values: `"docker"`, `"docker_compose"`, `"cloud_init"`

### Remote Action Level (Level 3)

- **action_type**: The type of remote action
  - Values: `"validation"`
- **component**: The component being acted upon
  - Values: `"cloud_init"`, `"docker"`, `"docker_compose"`
- **server_ip**: The target server IP address

## Environment Field Usage

The application supports multi-environment deployments (e.g., `e2e-full`, `e2e-config`, `e2e-provision`). The `environment` field helps identify which environment a log entry belongs to, especially important when debugging multiple environments concurrently.

### When to Include Environment Field

#### ✅ Always Include in Command Spans

All commands that operate on environments **must** include the environment field in their `#[instrument]` macro:

```rust
#[instrument(
    name = "provision_command",
    skip_all,
    fields(
        command_type = "provision",
        environment = %environment.name()  // ✅ Required
    )
)]
pub async fn execute(
    &self,
    environment: Environment<Created>,
) -> Result<Environment<Provisioned>, ProvisionCommandError> {
    // Command implementation...
}
```

**Commands that require environment field:**

- `ProvisionCommand` ✅
- `ConfigureCommand` ✅
- `TestCommand` ✅
- `DestroyCommand` (when implemented)
- `CreateCommand` (when implemented - use the name being created)

**Commands that do NOT require environment field:**

- `CheckCommand` - Generic system checks, no specific environment
- Internal tools - Linters, formatters, etc.

#### ✅ Include in High-Value Application Layer Logs

Add environment field to important logs in the **application layer** where environment context is available and adds value:

```rust
// Command-level logs
info!(
    command = "provision",
    environment = %environment.name(),  // ✅ Include for visibility
    "Starting complete infrastructure provisioning workflow"
);

// Step-level logs with significant operations
info!(
    step = "install_docker",
    environment = %environment.name(),  // ✅ Useful for debugging
    "Installing Docker via Ansible"
);
```

**Good candidates for environment field:**

- Command start/completion messages
- Step-level operations where environment provides context
- Error logs where environment helps identify the issue
- State transition logs

#### ❌ Do NOT Include in Infrastructure Layer

Infrastructure layer components should remain **environment-agnostic** to maintain proper abstraction:

```rust
// Infrastructure adapter - NO environment field
impl TofuClient {
    pub fn apply(&self, working_dir: &Path) -> Result<Output> {
        info!(
            working_dir = %working_dir.display(),
            // ❌ NO environment field - adapter is generic
            "Applying infrastructure changes"
        );
    }
}

// SSH client - NO environment field
impl SshClient {
    pub fn execute(&self, host: &str, command: &str) -> Result<Output> {
        info!(
            host = %host,
            command = %command,
            // ❌ NO environment field - client is generic
            "Executing SSH command"
        );
    }
}
```

**Never include environment in:**

- External tool adapters (`TofuClient`, `AnsibleClient`, `SshClient`)
- Infrastructure clients and wrappers
- Shared utilities (SSH, file operations, etc.)
- Generic helpers that don't operate on environments

### Abstraction Layers

```text
┌─────────────────────────────────────────────────────────┐
│ Application Layer (Environment-Aware)                   │
│ - Commands: provision, configure, test                  │
│ - Steps: infrastructure setup, software installation    │
│ ✅ Include environment field in spans and key logs      │
├─────────────────────────────────────────────────────────┤
│ Domain Layer (Business Logic)                           │
│ - Environment, State, Repository abstractions           │
│ ✅ Include environment field where it makes sense       │
├─────────────────────────────────────────────────────────┤
│ Infrastructure Layer (Environment-Agnostic)             │
│ - Adapters: TofuClient, AnsibleClient, SshClient        │
│ - External tool wrappers                                │
│ ❌ NEVER include environment field                      │
└─────────────────────────────────────────────────────────┘
```

### Examples from Codebase

#### ✅ Good: Command Span with Environment

```rust
// src/application/commands/provision.rs
#[instrument(
    name = "provision_command",
    skip_all,
    fields(
        command_type = "provision",
        environment = %environment.name()  // ✅ Correct
    )
)]
pub async fn execute(
    &self,
    environment: Environment<Created>,
) -> Result<Environment<Provisioned>, ProvisionCommandError> {
    info!(
        command = "provision",
        environment = %environment.name(),  // ✅ Explicit for visibility
        "Starting complete infrastructure provisioning workflow"
    );
    // ...
}
```

#### ✅ Good: Infrastructure Layer Without Environment

```rust
// src/infrastructure/external_tools/tofu/adapter/client.rs
impl OpenTofuClient {
    pub fn apply(&self, working_dir: &Path, auto_approve: bool) -> Result<Output> {
        info!(
            working_dir = %working_dir.display(),
            auto_approve = %auto_approve,
            // ✅ No environment - stays generic
            "Applying infrastructure changes"
        );
        // ...
    }
}
```

#### ❌ Bad: Environment in Infrastructure Layer

```rust
// src/infrastructure/external_tools/tofu/adapter/client.rs
impl OpenTofuClient {
    pub fn apply(
        &self,
        working_dir: &Path,
        environment: &str  // ❌ Wrong - breaks abstraction
    ) -> Result<Output> {
        info!(
            working_dir = %working_dir.display(),
            environment = %environment,  // ❌ Wrong - adapter should be generic
            "Applying infrastructure changes"
        );
        // ...
    }
}
```

### Visibility Through Span Hierarchy

Remember: Logs within command spans automatically inherit environment context. You don't need to add environment field to every log if the span hierarchy provides it:

```text
2025-10-08T09:35:40.731158Z  INFO torrust_tracker_deploy::application::steps::software::docker: Installing Docker via Ansible
  at src/application/steps/software/docker.rs:62
  in torrust_tracker_deploy::application::steps::software::docker::install_docker with step_type: "software", component: "docker"
  in torrust_tracker_deploy::application::commands::configure::configure_command with command_type: "configure", environment: e2e-full
```

**When to be explicit:**

- High-level command logs (start/completion)
- Error logs where environment is critical for diagnosis
- Logs that might be viewed outside span context (JSON aggregation)

**When to rely on span inheritance:**

- Nested step logs within command execution
- Infrastructure layer operations (no environment at all)
- Debug/trace logs where span context is sufficient

## Environment Variables

Control logging behavior with environment variables:

```bash
# Show all trace-level logs for development
export RUST_LOG=torrust_tracker_deploy=trace

# Production logging (info and above)
export RUST_LOG=torrust_tracker_deploy=info

# Only errors and warnings
export RUST_LOG=torrust_tracker_deploy=warn

# Detailed logging for specific modules
export RUST_LOG=torrust_tracker_deploy::commands=trace,torrust_tracker_deploy::steps=debug
```

## JSON Output Format

When using `logging_simple::init_json()`, logs are output in JSON format suitable for log aggregation:

```json
{
  "timestamp": "2024-09-16T17:00:00.000Z",
  "level": "INFO",
  "fields": {
    "command_type": "provision"
  },
  "target": "torrust_tracker_deploy::commands::provision",
  "span": {
    "name": "provision_command"
  },
  "spans": [{ "name": "provision_command", "command_type": "provision" }]
}
```

## Best Practices

### 1. Use Appropriate Log Levels

- **TRACE**: Detailed flow information (span entry/exit)
- **DEBUG**: Detailed information for debugging
- **INFO**: General information about application progress
- **WARN**: Warning messages about potential issues
- **ERROR**: Error messages about failures

### 2. Leverage Span Context

The hierarchical spans automatically provide context. You don't need to repeat information that's already captured in the span fields.

```rust
// Good: Span fields provide context
#[instrument(name = "docker_validation", fields(component = "docker"))]
pub async fn validate_docker() {
    info!("Starting validation");  // Context is implicit from span
}

// Avoid: Redundant context information
#[instrument(name = "docker_validation", fields(component = "docker"))]
pub async fn validate_docker() {
    info!(component = "docker", "Starting Docker validation");  // Redundant
}
```

### 3. Filter in Production

Use environment-specific filtering to reduce noise in production:

```bash
# Development: Show everything
export RUST_LOG=trace

# Production: Focus on important events
export RUST_LOG=torrust_tracker_deploy=info,warn,error
```

## Debugging

### Viewing Span Relationships

To see the full span hierarchy, use trace level logging:

```bash
RUST_LOG=torrust_tracker_deploy=trace cargo run --bin your-binary
```

### Finding Specific Operations

Filter logs for specific operations:

```bash
# Only infrastructure operations
RUST_LOG=torrust_tracker_deploy=info cargo run | grep infrastructure

# Only validation operations
RUST_LOG=torrust_tracker_deploy=trace cargo run | grep validation
```

### Performance Analysis

The span timings help identify slow operations:

```text
2024-09-16T17:00:00.000Z TRACE provision_command: entered
2024-09-16T17:00:05.234Z TRACE provision_command: exited  // 5.234 seconds total
```
