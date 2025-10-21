# Command Reference

This guide provides an overview of all available commands in the Torrust Tracker Deployer. Each command manages a specific phase of the deployment lifecycle.

## Quick Reference

```bash
# Environment lifecycle commands
torrust-tracker-deployer provision <env>    # Create infrastructure
torrust-tracker-deployer configure <env>    # Install software and configure system
torrust-tracker-deployer destroy <env>      # Remove infrastructure and cleanup

# Future commands
torrust-tracker-deployer release <env>      # Deploy application (not yet implemented)
torrust-tracker-deployer run <env>          # Start application services (not yet implemented)
torrust-tracker-deployer status <env>       # Check environment status (not yet implemented)
```

## Available Commands

### Environment Management

#### [`provision`](commands/provision.md)

Create new infrastructure with virtual machines and networking.

**Status**: ✅ Implemented

**Use when**: Starting a new deployment from scratch

```bash
torrust-tracker-deployer provision my-environment
```

Creates:

- LXD virtual machine or container
- Network configuration
- SSH access setup
- Initial system state

---

#### [`configure`](commands/configure.md)

Install software and configure the system after provisioning.

**Status**: 🔄 Partially Implemented

**Use when**: Setting up software on provisioned infrastructure

```bash
torrust-tracker-deployer configure my-environment
```

Configures:

- Docker and Docker Compose
- System packages and updates
- Firewall rules (future)
- Security settings (future)

---

#### [`destroy`](commands/destroy.md)

Remove environment and clean up all resources.

**Status**: ✅ Implemented

**Use when**: Tearing down temporary or failed environments

```bash
torrust-tracker-deployer destroy my-environment
```

Removes:

- All infrastructure resources
- Local state files
- Build artifacts

**⚠️ Warning**: This operation is destructive and irreversible.

---

### Application Management

#### `release` (Future)

Deploy application files and configuration.

**Status**: ❌ Not Yet Implemented

**Use when**: Deploying or updating the Torrust Tracker application

```bash
torrust-tracker-deployer release my-environment
```

Will deploy:

- Docker Compose configurations
- Application configuration files
- Environment variables
- Container images

---

#### `run` (Future)

Start application services and validate deployment.

**Status**: ❌ Not Yet Implemented

**Use when**: Starting the deployed application

```bash
torrust-tracker-deployer run my-environment
```

Will start:

- Docker Compose stack
- Application services
- Health monitoring

---

### Information and Validation

#### `status` (Future)

Display environment status and service health.

**Status**: ❌ Not Yet Implemented

**Use when**: Checking deployment health and status

```bash
torrust-tracker-deployer status my-environment
```

Will show:

- Current deployment state
- Infrastructure status
- Service health
- Resource usage

---

## Command Workflow

### Standard Deployment Flow

```bash
# 1. Create infrastructure
torrust-tracker-deployer provision production

# 2. Configure system
torrust-tracker-deployer configure production

# 3. Deploy application (future)
torrust-tracker-deployer release production

# 4. Start services (future)
torrust-tracker-deployer run production

# 5. Verify deployment (future)
torrust-tracker-deployer status production
```

### Development/Testing Flow

```bash
# Quick setup for testing
torrust-tracker-deployer provision test-env
torrust-tracker-deployer configure test-env

# Run tests...

# Quick teardown
torrust-tracker-deployer destroy test-env
```

### Update Existing Deployment

```bash
# Update configuration only
torrust-tracker-deployer configure production

# Update application (future)
torrust-tracker-deployer release production
torrust-tracker-deployer run production
```

## Common Options

All commands support these common options:

### Logging Options

Control log output and format:

```bash
# Log to file only (default, production-safe)
torrust-tracker-deployer <command> <env>

# Log to file and display on terminal
torrust-tracker-deployer <command> <env> --log-output file-and-stderr

# Change log format
torrust-tracker-deployer <command> <env> --log-format pretty
torrust-tracker-deployer <command> <env> --log-format json
torrust-tracker-deployer <command> <env> --log-format compact
```

**Log Formats**:

- `pretty` - Human-readable with colors (best for development)
- `json` - Structured JSON logs (best for log aggregation)
- `compact` - Minimal space usage (best for production files)

For detailed logging configuration, see the [Logging Guide](logging.md).

### Environment Variables

Control behavior through environment variables:

```bash
# Set log level
RUST_LOG=debug torrust-tracker-deployer provision my-env
RUST_LOG=info torrust-tracker-deployer configure my-env

# Detailed component logging
RUST_LOG=torrust_tracker_deployer::application::commands=trace \
    torrust-tracker-deployer provision my-env
```

**Log Levels**:

- `error` - Only errors
- `warn` - Warnings and errors
- `info` - General information (default)
- `debug` - Detailed debugging information
- `trace` - Very detailed trace information

### Help Information

Get help for any command:

```bash
torrust-tracker-deployer --help
torrust-tracker-deployer provision --help
torrust-tracker-deployer destroy --help
```

## State Transitions

Commands transition environments through defined states:

```text
┌─────────┐
│ Created │ (Initial state)
└────┬────┘
     │
     │ provision
     ▼
┌──────────────┐
│ Provisioned  │ (Infrastructure ready)
└──────┬───────┘
       │
       │ configure
       ▼
┌──────────────┐
│ Configured   │ (System configured)
└──────┬───────┘
       │
       │ release (future)
       ▼
┌──────────────┐
│  Released    │ (Application deployed)
└──────┬───────┘
       │
       │ run (future)
       ▼
┌──────────────┐
│   Running    │ (Services active)
└──────┬───────┘
       │
       │ destroy
       ▼
┌──────────────┐
│  Destroyed   │ (Cleaned up)
└──────────────┘
```

Each state represents a checkpoint in the deployment lifecycle.

## Error Handling

All commands provide:

- Clear error messages with context
- Actionable troubleshooting steps
- Detailed logs in `data/logs/log.txt`
- Trace information for complex errors

When a command fails:

1. **Check the error message** for immediate context
2. **Review logs** in `data/logs/log.txt` for details
3. **Follow troubleshooting steps** in error output
4. **Consult command documentation** for specific issues

## Getting Started

### First-Time Users

1. **Start with provision**: Create your first environment

   ```bash
   torrust-tracker-deployer provision my-first-env
   ```

2. **Configure the system**: Install required software

   ```bash
   torrust-tracker-deployer configure my-first-env
   ```

3. **Explore logs**: Check what happened

   ```bash
   cat data/logs/log.txt
   ```

4. **Clean up when done**: Remove the test environment

   ```bash
   torrust-tracker-deployer destroy my-first-env
   ```

### Prerequisites

Before using any command, ensure you have:

- **LXD** installed and configured (for provision/destroy)
- **OpenTofu** installed (for infrastructure management)
- **Ansible** installed (for configuration management)
- **Docker** installed (for E2E testing and configuration)

See the main README for installation instructions.

## Best Practices

### Production Deployments

1. **Always backup** before running destroy
2. **Test in non-production** first
3. **Use file-only logging** for production (default)
4. **Enable audit logs** for critical environments
5. **Document your workflows** in runbooks

### Development Workflows

1. **Use file-and-stderr** for real-time feedback
2. **Set appropriate log levels** with `RUST_LOG`
3. **Clean up test environments** regularly
4. **Version control configurations** for reproducibility

### Automation

1. **Handle failures gracefully** in scripts
2. **Use idempotent commands** (like destroy) safely
3. **Implement confirmation** for destructive operations
4. **Log all automation** actions for auditing

## Troubleshooting

### Common Issues

**Command hangs or takes too long**:

- Check `data/logs/log.txt` for progress
- Verify LXD/OpenTofu is responsive
- Use `--log-output file-and-stderr` to see real-time updates

**Permission errors**:

- Ensure user is in `lxd` group: `sudo usermod -aG lxd $USER`
- Check file permissions in `data/` and `build/` directories
- Verify OpenTofu has write access to state files

**State inconsistencies**:

- Review environment state in `data/state.json`
- Check OpenTofu state: `cd build/tofu/lxd && tofu show`
- Use destroy command to clean up and start fresh

## Related Documentation

- [Logging Guide](logging.md) - Configure logging and understand log formats
- [Contributing Guide](../contributing/commands.md) - Developer documentation for commands
- [E2E Testing Guide](../e2e-testing.md) - How commands are tested

## Next Steps

- Read the [Provision Command](commands/provision.md) guide to create your first environment
- Learn about [Logging](logging.md) to understand command output
- Explore [Template Customization](template-customization.md) to customize deployments
