# Command Reference

This guide provides an overview of the available commands for the Torrust Tracker Deployer.

## ⚠️ Implementation Status

**Current State**: The `destroy` command is fully implemented with CLI interface.

- ✅ **Implemented**: `destroy` command
- ❌ **CLI Interface**: Other commands not yet implemented

The CLI commands documented here represent the planned MVP implementation.

## Planned Commands (MVP)

```bash
# Available command
torrust-tracker-deployer destroy <env>     # Destroy infrastructure and clean up resources

# Future commands
torrust-tracker-deployer create <env>      # Future - Create environment configuration
torrust-tracker-deployer deploy <env>      # Future - Full deployment (provision → configure → release)
torrust-tracker-deployer run <env>         # Future - Start application services
torrust-tracker-deployer status <env>      # Future - Check environment status
torrust-tracker-deployer test <env>        # Future - Run validation tests
```

**Note**: The `deploy` command will internally orchestrate the complete deployment workflow: provision → configure → release. Individual commands for these phases may be added later for advanced users.

## Available Commands

### Environment Management

#### [`destroy`](commands/destroy.md)

Remove environment and clean up all resources.

**Status**: ✅ Fully Implemented

**Use when**: Tearing down temporary or failed environments

```bash
torrust-tracker-deployer destroy my-environment
```

Removes:

- All infrastructure resources (VMs, networks)
- Local state files
- Build artifacts

**⚠️ Warning**: This operation is destructive and irreversible.

---

### Future Commands

#### `create` (Planned)

Create a new environment configuration.

**Status**: ❌ Not Yet Implemented

**Use when**: Initializing a new deployment environment

```bash
# Planned CLI usage
torrust-tracker-deployer create my-environment
```

---

#### `deploy` (Planned)

Complete deployment workflow that orchestrates provision → configure → release.

**Status**: ❌ Not Yet Implemented

**Use when**: Deploying a new environment from start to finish

```bash
# Planned CLI usage
torrust-tracker-deployer deploy my-environment
```

This command will internally execute:

1. **Provision**: Create infrastructure (VMs, networks)
2. **Configure**: Install software and configure system
3. **Release**: Deploy application files and configuration

---

#### `run` (Planned)

Start application services.

**Status**: ❌ Not Yet Implemented

**Use when**: Starting the Torrust Tracker application

```bash
# Planned CLI usage
torrust-tracker-deployer run my-environment
```

---

#### `status` (Planned)

Display environment status and service health.

**Status**: ❌ Not Yet Implemented

**Use when**: Checking deployment health and status

```bash
# Planned CLI usage
torrust-tracker-deployer status my-environment
```

---

#### `test` (Planned)

Run validation tests on deployed environment.

**Status**: ❌ Not Yet Implemented

**Use when**: Validating deployment functionality

```bash
# Planned CLI usage
torrust-tracker-deployer test my-environment
```

---

## Deployment Workflow

### Standard Deployment (Planned)

The recommended workflow once all commands are implemented:

```bash
# 1. Create environment configuration
torrust-tracker-deployer create production

# 2. Deploy complete stack (provision → configure → release)
torrust-tracker-deployer deploy production

# 3. Start services
torrust-tracker-deployer run production

# 4. Verify deployment
torrust-tracker-deployer status production
torrust-tracker-deployer test production
```

### Development/Testing Workflow

```bash
# Quick teardown (implemented)
torrust-tracker-deployer destroy test-env

# Note: create and deploy commands are not yet implemented
```

## Environment States

Environments transition through the following states:

```text
Created → Provisioned → Configured → Released → Running → Destroyed
```

- **Created**: Environment configuration exists
- **Provisioned**: Infrastructure (VMs, networks) is running
- **Configured**: System software installed and configured
- **Released**: Application deployed
- **Running**: Services are active
- **Destroyed**: All resources cleaned up

## Common Options

The destroy command supports these options:

- `--help` - Display command help
- `--log-output <OUTPUT>` - Logging destination (`file-only` or `file-and-stderr`)
- `--log-file-format <FORMAT>` - File log format (`pretty`, `json`, or `compact`)
- `--log-stderr-format <FORMAT>` - Stderr log format (`pretty`, `json`, or `compact`)
- `--log-dir <DIR>` - Log directory (default: `./data/logs`)

### Environment Variables

- `RUST_LOG` - Control log verbosity (e.g., `RUST_LOG=debug`)

## Getting Started

New users should:

1. Start with the [`destroy` command documentation](commands/destroy.md) to understand the workflow
2. Review the deployment states and standard workflow
3. Consult individual command documentation as more commands become available

## Related Documentation

- [Destroy Command](commands/destroy.md) - Detailed destroy command documentation
- [Logging Guide](../logging.md) - Configure logging output
- [E2E Testing](../../e2e-testing.md) - Testing infrastructure and commands
