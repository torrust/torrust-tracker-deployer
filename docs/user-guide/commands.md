# Command Reference

This guide provides an overview of the planned commands for the Torrust Tracker Deployer.

## ⚠️ Implementation Status

**Current State**: Only the `DestroyCommand` handler exists in the Application layer. The CLI interface is not yet implemented.

- ✅ **Application Layer**: `DestroyCommand` handler implemented
- ❌ **CLI Interface**: Not yet implemented (coming in [issue #10](https://github.com/torrust/torrust-tracker-deployer/issues/10))

The CLI commands documented here represent the planned MVP implementation.

## Planned Commands (MVP)

```bash
# Planned CLI commands
torrust-tracker-deployer create <env>     # Future - Create environment configuration
torrust-tracker-deployer deploy <env>     # Future - Full deployment (provision → configure → release)
torrust-tracker-deployer run <env>        # Future - Start application services
torrust-tracker-deployer status <env>     # Future - Check environment status
torrust-tracker-deployer test <env>       # Future - Run validation tests
torrust-tracker-deployer destroy <env>    # Destroy infrastructure (App layer exists, CLI coming)
```

**Note**: The `deploy` command will internally orchestrate the complete deployment workflow: provision → configure → release. Individual commands for these phases may be added later for advanced users.

## Available Commands

### Environment Management

#### [`destroy`](commands/destroy.md)

Remove environment and clean up all resources.

**Status**:

- ✅ Application Layer: `DestroyCommand` implemented
- ❌ CLI: Not yet implemented (see [issue #10](https://github.com/torrust/torrust-tracker-deployer/issues/10))

**Use when**: Tearing down temporary or failed environments

```bash
# Planned CLI usage (not yet available)
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
# Quick setup for testing (once implemented)
torrust-tracker-deployer create test-env
torrust-tracker-deployer deploy test-env

# Run tests...

# Quick teardown (App layer exists, CLI coming)
torrust-tracker-deployer destroy test-env
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

All commands (when implemented) will support these options:

- `--help` - Display command help
- `--log-output <OUTPUT>` - Logging destination (`file-only` or `file-and-stderr`)
- `--log-format <FORMAT>` - Log format (`pretty`, `json`, or `compact`)

### Environment Variables

- `RUST_LOG` - Control log verbosity (e.g., `RUST_LOG=debug`)

## Getting Started

Once the CLI is implemented, new users should:

1. Start with the [`destroy` command documentation](commands/destroy.md) to understand the workflow
2. Review the deployment states and standard workflow
3. Consult individual command documentation as commands become available

## Related Documentation

- [Destroy Command](commands/destroy.md) - Detailed destroy command documentation
- [Logging Guide](../logging.md) - Configure logging output
- [E2E Testing](../../e2e-testing.md) - Testing infrastructure and commands
- [Issue #10 - UI Layer Destroy Command](../../issues/10-epic-ui-layer-destroy-command.md) - CLI implementation plan
