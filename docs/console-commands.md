# Console Commands - Torrust Tracker Deploy

> **⚠️ DRAFT DOCUMENTATION**  
> This document describes the planned console commands for the Torrust Tracker Deploy tool. Most functionality is **not yet implemented** in production code.

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

## Quick Command Reference

```bash
torrust-deploy check           # Validate required tools
torrust-deploy create <env>    # Create new environment configuration
torrust-deploy provision <env> # Create VM infrastructure
torrust-deploy configure <env> # Setup VM (Docker, networking)
torrust-deploy release <env>   # Deploy application files
torrust-deploy run <env>       # Start application stack
torrust-deploy test <env>      # Run smoke tests
torrust-deploy status <env>    # Show environment info
torrust-deploy destroy <env>   # Clean up infrastructure
```

## Detailed Command Specifications

### `check` - Tool Validation

**Status**: ❌ Not Implemented  
**State Transition**: None  
**Purpose**: Verify all required third-party tools are installed and properly configured.

```bash
torrust-deploy check [OPTIONS]
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

### `create` - Environment Creation

**Status**: ❌ Not Implemented (concept not supported yet)  
**State Transition**: → `created`  
**Purpose**: Initialize a new deployment environment configuration.

```bash
torrust-deploy create <environment-name> [OPTIONS]
```

**Current Limitation**: Only one hardcoded environment is supported. This command will be implemented when multi-environment support is added.

**Future Functionality**:

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
torrust-deploy provision <environment> [OPTIONS]
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
torrust-deploy configure <environment> [OPTIONS]
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

**Status**: ❌ Not Implemented  
**State Transition**: `configured` → `released`  
**Purpose**: Deploy Torrust Tracker application files and configuration.

```bash
torrust-deploy release <environment> [OPTIONS]
```

**Should Include**:

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
torrust-deploy run <environment> [OPTIONS]
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
torrust-deploy test <environment> [OPTIONS]
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
torrust-deploy status <environment> [OPTIONS]
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

**Status**: ✅ Implemented (in E2E tests)  
**State Transition**: Any state → `destroyed`  
**Purpose**: Clean up all infrastructure and resources for an environment.

```bash
torrust-deploy destroy <environment> [OPTIONS]
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

1. **High Priority** (Core functionality)

   - `provision` - Move from E2E to production ✅ Partially done
   - `configure` - Complete system setup 🔄 In progress
   - `destroy` - Production cleanup logic ✅ Partially done

2. **Medium Priority** (Application deployment)

   - `release` - Application deployment pipeline
   - `run` - Service management
   - `status` - Environment monitoring

3. **Low Priority** (Enhanced functionality)

   - `create` - Multi-environment support
   - `test` - Automated validation
   - `check` - Tool validation

## Notes

- All commands use the tracing crate for logging (control verbosity with `RUST_LOG` environment variable)
- Set `RUST_LOG=debug` for detailed output, `RUST_LOG=info` for standard output
- Configuration should be environment-aware once multi-env support is added
- Error handling should be consistent across all commands
- Each command should validate prerequisites before execution
- State transitions should be atomic where possible
