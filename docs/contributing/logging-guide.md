# Structured Logging Guide

This guide explains the structured logging implementation in the Torrust Tracker Deploy project, which uses hierar### JSON Output Format

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
