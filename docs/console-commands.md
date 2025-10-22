# Console Commands - Torrust Tracker Deployer

> **Note**: This document is part of the architecture planning and specification phase.
> This document describes the planned console commands for the Torrust Tracker Deployer tool. Most functionality is **not yet implemented** in production code.

## Current Implementation Status

### What's Currently Implemented (E2E Tests Only)

- **Provision**: VM creation with OpenTofu (LXD containers)
- **Configure**: Partial VM setup (Docker and Docker Compose installation via Ansible)
- Template rendering system (OpenTofu and Ansible templates)
- SSH connectivity and validation
- Infrastructure cleanup/destroy

### What's NOT Yet Implemented

- Multiple environment support (only one hardcoded environment)
- Application deployment (Docker Compose stack for Torrust Tracker)
- Environment state management
- Production console application structure
- Most command logic (exists only in E2E tests)

## Deployment States

The deployment follows a linear state progression:

1. **created** → Environment configuration exists
2. **provisioned** → VM/container infrastructure is running
3. **configured** → Basic system setup complete (Docker, networking, etc.)
4. **released** → Application files and configuration deployed
5. **running** → Torrust Tracker application is active
6. **destroyed** → Infrastructure cleaned up

Each command transitions the deployment to the next state.

## Minimum Deployment Workflow

The essential commands for a complete Torrust Tracker deployment:

```bash
# 1. Create environment configuration
torrust-tracker-deployer create myenv

# 2. Provision VM infrastructure
torrust-tracker-deployer provision myenv

# 3. Configure system (Docker, networking)
torrust-tracker-deployer configure myenv

# 4. Deploy Torrust Tracker application
torrust-tracker-deployer release myenv

# At this point, the tracker is deployed and can be started manually
# Later: torrust-tracker-deployer run myenv (when implemented)
```

This workflow takes an environment from non-existent to having a fully deployed Torrust Tracker ready to run.

## Hybrid Command Architecture

The deployer implements a **hybrid approach** offering two levels of command interface:

### Plumbing Commands (Low-Level, Implemented First)

Individual commands for precise control over each deployment step:

- `create` → `provision` → `configure` → `release` → `run`
- Each command performs one specific operation
- Ideal for CI/CD, automation, debugging, and advanced users
- **Implementation Priority**: High (these are implemented first)

### Porcelain Commands (High-Level, Implemented Later)

Simplified commands that orchestrate multiple plumbing commands:

- `deploy` - Intelligent deployment from current state to running
- Automatically determines next steps based on environment state
- If environment is already provisioned, starts from configure
- If environment is already configured, starts from release
- **Implementation Priority**: Medium (after plumbing commands are stable)

### Example Usage Patterns

```bash
# Porcelain: Simple deployment (future)
torrust-tracker-deployer create myenv
torrust-tracker-deployer deploy myenv    # Runs provision→configure→release→run

# Plumbing: Step-by-step control (current focus)
torrust-tracker-deployer create myenv
torrust-tracker-deployer provision myenv
torrust-tracker-deployer configure myenv
torrust-tracker-deployer release myenv
torrust-tracker-deployer run myenv
```

**Note**: Porcelain commands only automate the core deployment workflow (`provision` → `configure` → `release` → `run`). Management commands like `create`, `list`, `check`, `status`, `destroy` remain individual operations.

## Quick Command Reference

```bash
# Utility Commands
torrust-tracker-deployer check           # Validate required tools
torrust-tracker-deployer list            # List all environments

# Environment Management
torrust-tracker-deployer create <env>    # Create new environment configuration
torrust-tracker-deployer status <env>    # Show environment info
torrust-tracker-deployer destroy <env>   # Clean up infrastructure

# Porcelain Commands (High-Level)
torrust-tracker-deployer deploy <env>    # Smart deployment from current state (future)

# Plumbing Commands (Low-Level)
torrust-tracker-deployer provision <env> # Create VM infrastructure
torrust-tracker-deployer configure <env> # Setup VM (Docker, networking)
torrust-tracker-deployer release <env>   # Deploy application files
torrust-tracker-deployer run <env>       # Start application stack

# Validation
torrust-tracker-deployer test <env>      # Run smoke tests
```

## Detailed Command Specifications

### `check` - Tool Validation

**Status**: ❌ Not Implemented  
**State Transition**: None  
**Purpose**: Verify all required third-party tools are installed and properly configured.

```bash
torrust-tracker-deployer check [OPTIONS]
```

**Validates**:

- OpenTofu installation and version
- Ansible installation and version
- LXD setup and permissions
- SSH key availability
- Network connectivity to container registries

**Options**:

- `--fix` - Attempt to auto-install missing tools

**Environment Variables**:

- `RUST_LOG=debug` - Show detailed validation steps via tracing

**Example Output**:

```text
✅ OpenTofu v1.6.0 - OK
✅ Ansible v2.15.0 - OK
✅ LXD v5.0 - OK
❌ SSH key not found at ~/.ssh/id_rsa
⚠️  Docker registry connectivity - Slow response
```

---

### `list` - Environment Listing

**Status**: ❌ Not Implemented  
**State Transition**: None (read-only)  
**Purpose**: Display a summary of all available deployment environments.

```bash
torrust-tracker-deployer list [OPTIONS]
```

**Should Display**:

- Environment name
- Current deployment state (created, provisioned, configured, etc.)
- Creation timestamp
- Last modified timestamp
- Infrastructure status (if provisioned)
- Brief resource summary (IP address, instance type)

**Options**:

- `--format <table|json|yaml>` - Output format
- `--state <state>` - Filter by deployment state
- `--sort <name|created|modified|state>` - Sort criteria

**Environment Variables**:

- `RUST_LOG=debug` - Show detailed listing information via tracing

**Example Output**:

```text
NAME          STATE        CREATED      MODIFIED     IP ADDRESS
e2e-test      provisioned  2 hours ago  30 min ago   10.140.190.14
production    running      5 days ago   1 hour ago   10.140.192.45
staging       configured   1 day ago    6 hours ago  10.140.191.23
development   destroyed    3 days ago   2 days ago   -
```

---

### `deploy` - Smart Deployment (Porcelain Command)

**Status**: ❌ Not Implemented (future porcelain command)  
**State Transition**: Current state → `running` (intelligent progression)  
**Purpose**: Orchestrate the deployment workflow from the current environment state to running.

```bash
torrust-tracker-deployer deploy <environment> [OPTIONS]
```

**Intelligent Behavior**:

- **From `created`**: Runs `provision` → `configure` → `release` → `run`
- **From `provisioned`**: Runs `configure` → `release` → `run`
- **From `configured`**: Runs `release` → `run`
- **From `released`**: Runs `run`
- **From `running`**: Reports already running (no-op)

**Benefits**:

- Simplified user experience for common deployment workflow
- Automatic state detection and appropriate action selection
- Reduces need to remember command sequences
- Ideal for development and testing workflows

**Options**:

- `--skip-run` - Deploy but don't start services
- `--dry-run` - Show what steps would be executed
- `--verbose` - Show detailed progress information

**Environment Variables**:

- `RUST_LOG=debug` - Show detailed deployment orchestration via tracing

**Implementation Note**: This is a **porcelain command** that will be implemented after the underlying plumbing commands (`provision`, `configure`, `release`, `run`) are stable.

---

### `create` - Environment Creation

**Status**: ❌ Not Implemented (essential for deployment)  
**State Transition**: → `created`  
**Purpose**: Initialize a new deployment environment configuration.

```bash
torrust-tracker-deployer create <environment-name> [OPTIONS]
```

**Current Limitation**: Only one hardcoded environment is supported. This command is essential for creating environments and must be implemented for basic deployment functionality.

**Essential Functionality**:

- Generate environment configuration files
- Set up directory structure for the environment
- Create SSH key pairs if needed
- Initialize template customizations

**Options**:

- `--provider <lxd|multipass>` - Infrastructure provider
- `--template <path>` - Custom template directory
- `--ssh-key <path>` - Specific SSH key to use

---

### `provision` - Infrastructure Provisioning

**Status**: ✅ Implemented (in E2E tests)  
**State Transition**: `created` → `provisioned`  
**Purpose**: Create and initialize VM/container infrastructure.

```bash
torrust-tracker-deployer provision <environment> [OPTIONS]
```

**Current Implementation**:

- Renders OpenTofu templates to `build/tofu/`
- Runs `tofu init` and `tofu apply`
- Creates LXD container with cloud-init
- Waits for network connectivity
- Returns instance IP address

**What It Does**:

1. Template rendering (OpenTofu configurations)
2. Infrastructure initialization (`tofu init`)
3. Resource creation (`tofu apply`)
4. Network and SSH connectivity validation

**Options**:

- `--auto-approve` - Skip confirmation prompts
- `--keep-on-failure` - Don't cleanup on provision failure

**Environment Variables**:

- `RUST_LOG=debug` - Detailed provisioning logs via tracing

---

### `configure` - System Configuration

**Status**: 🔄 Partially Implemented (Docker & Docker Compose only)  
**State Transition**: `provisioned` → `configured`  
**Purpose**: Set up the basic system environment and dependencies.

```bash
torrust-tracker-deployer configure <environment> [OPTIONS]
```

**Current Implementation**:

- Renders Ansible templates with runtime variables
- Waits for cloud-init completion
- Installs Docker via Ansible playbook
- Installs Docker Compose via Ansible playbook

**Complete Configuration Should Include**:

- System updates and security patches
- Docker and Docker Compose installation ✅
- Firewall configuration ❌
- User account setup ❌
- System monitoring setup ❌
- Log rotation configuration ❌

**Options**:

- `--skip-updates` - Skip system package updates
- `--configure-firewall` - Set up firewall rules
- `--install-monitoring` - Install monitoring agents

---

### `release` - Application Deployment

**Status**: ❌ Not Implemented (critical for deployment)  
**State Transition**: `configured` → `released`  
**Purpose**: Deploy Torrust Tracker application files and configuration.

```bash
torrust-tracker-deployer release <environment> [OPTIONS]
```

**Critical Functionality**:

- Generate Docker Compose configuration for Torrust Tracker
- Create environment variable files
- Copy application configuration to VM
- Pull required Docker images
- Validate configuration files

**Options**:

- `--config <path>` - Custom tracker configuration
- `--image-tag <tag>` - Specific Torrust Tracker image version
- `--dry-run` - Validate without deploying

---

### `run` - Application Startup

**Status**: ❌ Not Implemented  
**State Transition**: `released` → `running`  
**Purpose**: Start the Torrust Tracker Docker Compose stack.

```bash
torrust-tracker-deployer run <environment> [OPTIONS]
```

**Should Include**:

- Execute `docker-compose up -d`
- Validate service startup
- Check container health status
- Display service URLs and ports

**Options**:

- `--wait` - Wait for all services to be healthy
- `--logs` - Show service logs during startup
- `--timeout <seconds>` - Startup timeout

---

### `test` - Deployment Validation

**Status**: ❌ Not Implemented  
**State Transition**: None (validation only)  
**Purpose**: Run smoke tests against a deployed environment.

```bash
torrust-tracker-deployer test <environment> [OPTIONS]
```

**Should Include**:

- HTTP endpoint health checks
- Torrent tracker API validation
- Database connectivity tests
- Performance baseline checks

**Options**:

- `--suite <basic|full|performance>` - Test suite selection
- `--timeout <seconds>` - Test timeout
- `--report <format>` - Test report format (json, xml, text)

---

### `status` - Environment Information

**Status**: ❌ Not Implemented  
**State Transition**: None (read-only)  
**Purpose**: Display comprehensive environment status and information.

```bash
torrust-tracker-deployer status <environment> [OPTIONS]
```

**Should Display**:

- Current deployment state
- Infrastructure status (VM running, IP address)
- Service health (containers, ports, endpoints)
- Resource usage (CPU, memory, disk)
- Recent deployment history

**Options**:

- `--format <table|json|yaml>` - Output format
- `--watch` - Continuous monitoring mode
- `--services-only` - Show only service status

---

### `destroy` - Infrastructure Cleanup

**Status**: 🔄 Being Implemented  
**State Transition**: Any state → `destroyed`  
**Purpose**: Clean up all infrastructure and resources for an environment.

```bash
torrust-tracker-deployer destroy <environment> [OPTIONS]
```

**Current Implementation**:

- Runs `tofu destroy --auto-approve`
- Removes LXD containers and networks
- Cleans up temporary files

**Should Include**:

- Stop running services gracefully
- Backup data if requested
- Remove all infrastructure resources
- Clean up local configuration files
- Confirm destruction completion

**Options**:

- `--force` - Skip confirmation prompts
- `--backup` - Create backup before destruction
- `--keep-data` - Preserve persistent data volumes

## Implementation Priority

### Phase 1: Plumbing Commands (High Priority)

Essential low-level commands for the complete deployment workflow:

- `create` - Environment initialization (required to create environments)
- `provision` - Move from E2E to production ✅ Partially done
- `configure` - Complete system setup 🔄 In progress
- `release` - Application deployment (critical for getting Torrust Tracker on VM)
- `destroy` - Infrastructure cleanup 🔄 Being implemented

### Phase 2: Operations Commands (Medium Priority)

Management and operational commands:

- `run` - Service management (users can start manually after release)
- `status` - Environment monitoring
- `list` - Environment listing and overview

### Phase 3: Porcelain Commands (Medium Priority)

High-level commands built on top of stable plumbing commands:

- `deploy` - Smart deployment orchestration (porcelain command)

### Phase 4: Enhanced Functionality (Low Priority)

Additional features and utilities:

- `test` - Automated validation
- `check` - Tool validation

## Notes

### Command Architecture

- **Hybrid approach**: Combines low-level plumbing commands with high-level porcelain commands
- **Plumbing first**: Individual commands (`provision`, `configure`, etc.) implemented before orchestration commands
- **Porcelain commands scope**: Only automate the deployment workflow (`provision` → `configure` → `release` → `run`)
- **Management commands**: `create`, `list`, `check`, `status`, `destroy` remain individual operations

### Technical Implementation

- All commands use the tracing crate for logging (control verbosity with `RUST_LOG` environment variable)
- Set `RUST_LOG=debug` for detailed output, `RUST_LOG=info` for standard output
- Configuration should be environment-aware once multi-env support is added
- Error handling should be consistent across all commands
- Each command should validate prerequisites before execution
- State transitions should be atomic where possible
