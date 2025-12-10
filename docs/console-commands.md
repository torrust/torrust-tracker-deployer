# Console Commands - Torrust Tracker Deployer

> **Note**: This document describes the console commands for the Torrust Tracker Deployer tool.

## Current Implementation Status

### ✅ What's Currently Implemented (Available via CLI)

- **Create Template**: Generate environment configuration template (JSON)
- **Create Environment**: Create new deployment environment from configuration file
- **Provision**: VM infrastructure provisioning with OpenTofu (LXD and Hetzner Cloud)
- **Register**: Register existing instances as an alternative to provisioning (for pre-existing VMs, servers, or containers)
- **Configure**: VM configuration with Docker, Docker Compose, and firewall via Ansible
- **Test**: Verification of deployment infrastructure (cloud-init, Docker, Docker Compose)
- **Release**: Deploy application configuration and files (tracker config, docker-compose stack)
- **Run**: Start Torrust Tracker services and validate accessibility
- **Destroy**: Infrastructure cleanup and environment destruction
- Template rendering system (OpenTofu, Ansible, Tracker, Docker Compose templates)
- SSH connectivity validation
- Environment state management and persistence

### ⚠️ What's NOT Yet Implemented

- Porcelain commands (high-level `deploy` command)
- Additional cloud providers (AWS, Azure, GCP)

## Deployment States

The deployment follows a linear state progression:

1. **created** → Environment configuration exists
2. **provisioned** → VM/container infrastructure is running
3. **configured** → Basic system setup complete (Docker, networking, etc.)
4. **released** → Application files and configuration deployed
5. **running** → Torrust Tracker application is active
6. **destroyed** → Infrastructure cleaned up

Each command transitions the deployment to the next state.

## Complete Deployment Workflow

The full deployment workflow with all implemented commands:

```bash
# 1. Generate configuration template
torrust-tracker-deployer create template --provider lxd > my-env.json

# 2. Edit my-env.json with your settings (SSH keys, tracker config, etc.)

# 3. Create environment from configuration
torrust-tracker-deployer create environment --env-file my-env.json

# 4a. Provision NEW VM infrastructure
torrust-tracker-deployer provision my-environment

# 4b. OR Register EXISTING infrastructure (alternative to provision)
torrust-tracker-deployer register my-environment --instance-ip 192.168.1.100

# 5. Configure system (Docker, Docker Compose, firewall)
torrust-tracker-deployer configure my-environment

# 6. Verify deployment infrastructure
torrust-tracker-deployer test my-environment

# 7. Deploy application configuration and files
torrust-tracker-deployer release my-environment

# 8. Start Torrust Tracker services
torrust-tracker-deployer run my-environment

# 9. Destroy environment when done
torrust-tracker-deployer destroy my-environment
```

This workflow deploys a complete Torrust Tracker instance with all configuration and services running.

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
# Utility Commands (Planned)
torrust-tracker-deployer check           # Validate required tools (not yet implemented)
torrust-tracker-deployer list            # List all environments (not yet implemented)

# Environment Management
torrust-tracker-deployer create template [PATH]         # ✅ Generate configuration template
torrust-tracker-deployer create environment -f <file>   # ✅ Create environment from config
# Plumbing Commands (Low-Level)
torrust-tracker-deployer provision <env> # ✅ Create VM infrastructure
torrust-tracker-deployer register <env> --instance-ip <IP>  # ✅ Register existing infrastructure
torrust-tracker-deployer configure <env> # ✅ Setup VM (Docker, Docker Compose, firewall)
torrust-tracker-deployer release <env>   # ✅ Deploy application files and configuration
torrust-tracker-deployer run <env>       # ✅ Start Torrust Tracker services

# Validation
torrust-tracker-deployer test <env>      # ✅ Verify infrastructure (cloud-init, Docker, Docker Compose)
torrust-tracker-deployer configure <env> # ✅ Setup VM (Docker, Docker Compose)
torrust-tracker-deployer release <env>   # Deploy application files (not yet implemented)
torrust-tracker-deployer run <env>       # Start application stack (not yet implemented)

# Validation
torrust-tracker-deployer test <env>      # ✅ Verify infrastructure (cloud-init, Docker, Docker Compose)
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

### `register` - Register Existing Infrastructure

**Status**: ✅ Implemented  
**State Transition**: `created` → `provisioned`  
**Purpose**: Register existing infrastructure as an alternative to provisioning new resources.

```bash
torrust-tracker-deployer register <environment> --instance-ip <IP_ADDRESS>
```

**Current Implementation**:

- Loads existing environment in `Created` state
- Validates SSH connectivity using environment's SSH credentials
- Sets `runtime_outputs.instance_ip` to the provided IP address
- Marks environment with `provision_method: Registered` metadata
- Renders Ansible templates with runtime variables
- Transitions to `Provisioned` state

**Use Cases**:

- Deploy to spare/existing servers
- Use infrastructure from unsupported cloud providers
- E2E testing with Docker containers (faster than VMs)
- Custom infrastructure configurations

**Arguments**:

- `<environment>` - Name of environment in `Created` state
- `--instance-ip <IP_ADDRESS>` - IP address of existing instance (IPv4 or IPv6)

**Instance Requirements**:

- Ubuntu 24.04 LTS
- SSH connectivity with credentials from `create environment`
- Public SSH key installed for access
- Username with sudo access

**Example**:

```bash
# First, create the environment with SSH credentials
torrust-tracker-deployer create environment -f config.json

# Register existing server instead of provisioning
torrust-tracker-deployer register my-environment --instance-ip 192.168.1.100

# Output:
# ✓ Loading environment...
# ✓ Validating SSH connectivity...
# ✓ Registering instance IP: 192.168.1.100
# ✓ Rendering Ansible templates...
# ✓ Environment registered successfully
```

**Important**: When you destroy a registered environment, the underlying infrastructure is **preserved**. Only the deployer's environment data is removed. This is a key safety feature for registered instances.

**Environment Variables**:

- `RUST_LOG=debug` - Detailed registration logs via tracing

---

### `configure` - System Configuration

**Status**: ✅ Implemented  
**State Transition**: `provisioned` → `configured`  
**Purpose**: Configure the provisioned infrastructure with required software and system settings.

```bash
torrust-tracker-deployer configure <environment>
```

**Current Implementation**:

- Renders Ansible templates with runtime variables
- Waits for cloud-init completion
- Installs Docker engine via Ansible playbook
- Installs Docker Compose via Ansible playbook
- Verifies successful installation
- Updates environment state to `configured`

**Planned Enhancements** (Future):

- System security configuration (UFW firewall, automatic updates) - Partially implemented in app layer
- User account setup
- System monitoring setup
- Log rotation configuration

**Example**:

```bash
# Configure provisioned environment
torrust-tracker-deployer configure my-environment

# Output:
# ✓ Rendering Ansible templates...
# ✓ Waiting for cloud-init completion...
# ✓ Installing Docker...
# ✓ Installing Docker Compose...
# ✓ Environment configured successfully
```

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

### `test` - Infrastructure Validation

**Status**: ✅ Implemented  
**State Transition**: None (validation only, does not change environment state)  
**Purpose**: Verify that the deployment infrastructure is properly configured and ready.

```bash
torrust-tracker-deployer test <environment>
```

**Current Implementation**:

- Verifies cloud-init completion on the provisioned instance
- Validates Docker installation and availability
- Validates Docker Compose installation and version
- Returns success/failure status for the entire infrastructure stack

**Use Cases**:

- Verify infrastructure after provisioning
- Confirm configuration was successful
- Validate environment before application deployment
- Troubleshooting infrastructure issues

**Example**:

```bash
# Test infrastructure readiness
torrust-tracker-deployer test my-environment

# Output:
# ✓ Checking cloud-init status...
# ✓ Validating Docker installation...
# ✓ Validating Docker Compose installation...
# ✓ All infrastructure checks passed
```

**Planned Enhancements** (Future):

- HTTP endpoint health checks (when application is deployed)
- Torrent tracker API validation (when tracker is running)
- Database connectivity tests (when database is deployed)
- Performance baseline checks

---

### `release` - Deploy Application Configuration

**Status**: ✅ Implemented  
**State Transition**: `Configured` → `Released`  
**Purpose**: Deploy application configuration files and prepare the environment for running services.

```bash
torrust-tracker-deployer release <environment>
```

**Current Implementation**:

- Creates storage directory structure on VM (`/opt/torrust/storage/tracker/`)
- Initializes SQLite database for tracker
- Renders tracker configuration from environment settings (`tracker.toml`)
- Generates Docker Compose environment variables (`.env`)
- Deploys all configuration files to VM
- Synchronizes Docker Compose stack files

**What Gets Deployed**:

- Tracker configuration: `/opt/torrust/storage/tracker/etc/tracker.toml`
- Database file: `/opt/torrust/storage/tracker/lib/database/tracker.db`
- Environment variables: `/opt/torrust/.env`
- Docker Compose stack: `/opt/torrust/docker-compose.yml`

**Use Cases**:

- Deploy application after infrastructure is configured
- Update tracker configuration (re-run after editing environment.json)
- Prepare environment for running services

**Example**:

```bash
# Deploy application configuration
torrust-tracker-deployer release my-environment

# Output:
# ✓ Creating tracker storage directories...
# ✓ Initializing tracker database...
# ✓ Rendering tracker templates...
# ✓ Deploying tracker configuration...
# ✓ Deploying Docker Compose files...
# ✓ Release complete - environment ready to run
```

**Configuration Source**:

The release command uses tracker configuration from your environment JSON:

```json
{
  "tracker": {
    "core": {
      "database_name": "tracker.db",
      "private": false
    },
    "udp_trackers": [{ "bind_address": "0.0.0.0:6868" }],
    "http_trackers": [{ "bind_address": "0.0.0.0:7070" }],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    }
  }
}
```

**Idempotent Operation**:

- Can be re-run safely to update configuration
- Existing database is preserved
- Configuration files are overwritten with new values

---

### `run` - Start Tracker Services

**Status**: ✅ Implemented  
**State Transition**: `Released` → `Running`  
**Purpose**: Start the Torrust Tracker application services and validate they are running.

```bash
torrust-tracker-deployer run <environment>
```

**Current Implementation**:

- Starts Docker Compose services (`docker compose up -d`)
- Validates services are running via Docker status
- Performs external health checks on tracker API
- Verifies firewall allows external access

**Services Started**:

- **Tracker container** (`torrust/tracker:develop`)
  - UDP Tracker endpoints (ports 6868, 6969 by default)
  - HTTP Tracker endpoint (port 7070 by default)
  - HTTP API endpoint (port 1212 by default)

**Health Checks Performed**:

1. **Docker Compose Status** - Verifies containers are running
2. **Tracker API Health** (required) - Tests external accessibility of HTTP API
   - Endpoint: `http://<vm-ip>:1212/api/health_check`
   - Validates service functionality AND firewall configuration
3. **HTTP Tracker Health** (optional) - Tests external accessibility of HTTP tracker
   - Endpoint: `http://<vm-ip>:7070/api/health_check`
   - Warning only if check fails (not all versions have endpoint)

**Use Cases**:

- Start tracker services after release
- Restart services after configuration changes
- Validate tracker is accessible externally

**Example**:

```bash
# Start tracker services
torrust-tracker-deployer run my-environment

# Output:
# ✓ Starting Docker Compose services...
# ✓ Validating services are running...
# ✓ Checking tracker API accessibility...
# ✓ Tracker services running and accessible
```

**Verification**:

After running, you can access the tracker:

```bash
# Get VM IP
VM_IP=$(torrust-tracker-deployer show my-environment | grep 'IP Address' | awk '{print $3}')

# Test tracker API
curl http://$VM_IP:1212/api/health_check

# Get tracker statistics
curl http://$VM_IP:1212/api/v1/stats
```

**Announce URLs**:

- UDP: `udp://<vm-ip>:6868/announce` or `udp://<vm-ip>:6969/announce`
- HTTP: `http://<vm-ip>:7070/announce`

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

### Phase 1: Plumbing Commands (High Priority) - ✅ MOSTLY COMPLETE

Essential low-level commands for the complete deployment workflow:

- ✅ `create template` - Template generation (completed)
- ✅ `create environment` - Environment initialization (completed)
- ✅ `provision` - Infrastructure provisioning (completed)
- ✅ `register` - Register existing infrastructure (completed)
- ✅ `configure` - System configuration (completed)
- ✅ `test` - Infrastructure validation (completed)
- ✅ `destroy` - Infrastructure cleanup (completed)
- ❌ `release` - Application deployment (not yet implemented - critical for deploying Torrust Tracker)
- ❌ `run` - Service management (not yet implemented)

### Phase 2: Operations Commands (Medium Priority)

Management and operational commands:

- ❌ `status` - Environment monitoring (not yet implemented)
- ❌ `list` - Environment listing and overview (not yet implemented)

### Phase 3: Porcelain Commands (Medium Priority)

High-level commands built on top of stable plumbing commands:

- ❌ `deploy` - Smart deployment orchestration (not yet implemented - porcelain command)

### Phase 4: Enhanced Functionality (Low Priority)

Additional features and utilities:

- ❌ `check` - Tool validation (not yet implemented)

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
