# Command Reference

This guide provides an overview of the available commands for the Torrust Tracker Deployer.

## ⚠️ Implementation Status

**Current State**: Core deployment commands are fully implemented with CLI interface.

- ✅ **Implemented**: `create`, `provision`, `configure`, `test`, `destroy` commands
- ✅ **Multi-Provider Support**: LXD (local) and Hetzner Cloud (production)
- ❌ **CLI Interface**: `deploy`, `run`, `status` commands not yet implemented

## Provider Selection

All commands work with both supported providers. The provider is specified in your environment configuration file:

| Provider    | Configuration           | Use Case                 |
| ----------- | ----------------------- | ------------------------ |
| **LXD**     | `"provider": "lxd"`     | Local development, CI/CD |
| **Hetzner** | `"provider": "hetzner"` | Production deployments   |

📖 **See [Provider Guides](providers/README.md)** for detailed provider configuration.

## Available Commands

```bash
# Available commands
torrust-tracker-deployer create template --provider <lxd|hetzner> [path]  # Generate configuration template
torrust-tracker-deployer create environment --env-file <file>             # Create new deployment environment
torrust-tracker-deployer provision <env>                                   # Provision VM infrastructure
torrust-tracker-deployer configure <env>                                   # Configure infrastructure (Docker, etc.)
torrust-tracker-deployer test <env>                                        # Verify deployment
torrust-tracker-deployer destroy <env>                                     # Destroy infrastructure and clean up

# Future commands
torrust-tracker-deployer deploy <env>      # Future - Full deployment (provision → configure → release)
torrust-tracker-deployer run <env>         # Future - Start application services
torrust-tracker-deployer status <env>      # Future - Check environment status
```

**Note**: The `deploy` command will internally orchestrate the complete deployment workflow: provision → configure → release. Individual commands for these phases may be added later for advanced users.

## Command Details

### Environment Management

#### [`create environment`](commands/create.md)

Create a new deployment environment from a configuration file.

**Status**: ✅ Fully Implemented

**Use when**: Initializing new environments for development, testing, or production

```bash
torrust-tracker-deployer create environment --env-file ./config/environment.json
```

Creates:

- Environment data directory
- Initial environment state (Created)
- Persistent configuration

---

#### `create template`

Generate a configuration template file for environment creation.

**Status**: ✅ Fully Implemented

**Use when**: Starting a new environment configuration from scratch

```bash
# Generate template with default name
torrust-tracker-deployer create template

# Generate template with custom name
torrust-tracker-deployer create template my-config.json
```

Creates: JSON configuration template with placeholder values

---

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
# 1. Generate configuration template
torrust-tracker-deployer create template production-config.json

# 2. Edit the configuration file
nano production-config.json

# 3. Create environment configuration
torrust-tracker-deployer create environment --env-file production-config.json

# 4. Deploy complete stack (provision → configure → release)
torrust-tracker-deployer deploy production

# 5. Start services
torrust-tracker-deployer run production

# 6. Verify deployment
torrust-tracker-deployer status production
torrust-tracker-deployer test production
```

### Development/Testing Workflow

Current available commands for development and testing:

```bash
# 1. Create environment from configuration
torrust-tracker-deployer create environment --env-file dev-config.json

# 2. When done, clean up
torrust-tracker-deployer destroy dev-env
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

The create and destroy commands support these common options:

- `--help` - Display command help- `--output-format <FORMAT>`, `-o` - Output format for command results (`text` or `json`, default: `text`)- `--log-output <OUTPUT>` - Logging destination (`file-only` or `file-and-stderr`)
- `--log-file-format <FORMAT>` - File log format (`pretty`, `json`, or `compact`)
- `--log-stderr-format <FORMAT>` - Stderr log format (`pretty`, `json`, or `compact`)
- `--log-dir <DIR>` - Log directory (default: `./data/logs`)
- `--working-dir <DIR>` - Working directory for environment data (default: `.`)

### Environment Variables

- `RUST_LOG` - Control log verbosity (e.g., `RUST_LOG=debug`)

## Getting Started

New users should:

1. Start with the [`create environment` command documentation](commands/create-environment.md) to set up your first environment
2. Use the [`create template` command](#create-template) to generate a configuration file
3. Review the [`destroy` command documentation](commands/destroy.md) to understand cleanup workflow
4. Consult individual command documentation as more commands become available

## Related Documentation

- [Create Environment Command](commands/create-environment.md) - Detailed create environment command documentation
- [Destroy Command](commands/destroy.md) - Detailed destroy command documentation
- [Logging Guide](../logging.md) - Configure logging output
- [E2E Testing](../../e2e-testing.md) - Testing infrastructure and commands
