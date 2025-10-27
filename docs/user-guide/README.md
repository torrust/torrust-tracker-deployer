# Torrust Tracker Deployer - User Guide

Welcome to the Torrust Tracker Deployer user guide! This guide will help you get started with deploying and managing Torrust Tracker environments.

## 📋 Table of Contents

- [Overview](#overview)
- [Current Status](#current-status)
- [Quick Start](#quick-start)
- [Available Commands](#available-commands)
- [Basic Workflows](#basic-workflows)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)
- [Additional Resources](#additional-resources)

## Overview

The Torrust Tracker Deployer is a command-line tool for managing deployment environments for Torrust Tracker applications. It provides automated infrastructure provisioning, configuration management, and deployment orchestration.

### What Can You Do?

Currently, you can:

- ✅ **Create environments** - Initialize new deployment environments with configuration
- ✅ **Generate templates** - Create configuration templates for new environments
- ✅ **Destroy environments** - Clean up infrastructure and resources

Coming soon:

- 🔄 **Deploy applications** - Full deployment workflow (provision → configure → release)
- 🔄 **Run applications** - Start deployed applications
- 🔄 **Check status** - View environment health and state

## Current Status

**Implementation Status**: MVP Development Phase

The Torrust Tracker Deployer is currently in active development. The following features are implemented:

- ✅ **Environment Creation** - Create and manage environment configurations
- ✅ **Template Generation** - Generate configuration templates
- ✅ **Environment Destruction** - Clean up environments and infrastructure
- ❌ **Provisioning** - Infrastructure creation (in development)
- ❌ **Configuration** - System setup (planned)
- ❌ **Deployment** - Application deployment (planned)
- ❌ **Runtime Management** - Service control (planned)

**Target Platform**: Currently supports local development using LXD virtual machines. Cloud provider support (Hetzner, AWS, GCP, Azure) is planned for future releases.

## Quick Start

### Prerequisites

Before using the Torrust Tracker Deployer, ensure you have:

- Rust toolchain (for building from source)
- SSH key pair for VM access
- LXD installed and configured (for local deployments)

### Installation

Build from source:

```bash
git clone https://github.com/torrust/torrust-tracker-deployer.git
cd torrust-tracker-deployer
cargo build --release
```

The binary will be available at `./target/release/torrust-tracker-deployer`.

### Your First Environment

Here's how to create your first environment:

#### Step 1: Generate Configuration Template

```bash
torrust-tracker-deployer create template my-config.json
```

This creates a template file with placeholder values.

#### Step 2: Edit Configuration

Edit `my-config.json` and replace the placeholder values:

```json
{
  "environment": {
    "name": "dev-local"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub",
    "username": "torrust",
    "port": 22
  }
}
```

**Required Changes**:

- `name` - Choose a unique environment name (e.g., `dev-local`, `staging`, `prod`)
- `private_key_path` - Path to your SSH private key
- `public_key_path` - Path to your SSH public key

#### Step 3: Create Environment

```bash
torrust-tracker-deployer create environment --env-file my-config.json
```

Success output:

```text
✅ Environment 'dev-local' created successfully
Instance name: torrust-tracker-vm-dev-local
Data directory: data/dev-local
Build directory: build/dev-local
```

#### Step 4: Verify Creation

Check that the environment was created:

```bash
# View environment state file
cat dev-local/environment.json

# Check the data directory structure
ls -la data/
```

#### Step 5: Clean Up (When Finished)

When you're done with the environment:

```bash
torrust-tracker-deployer destroy dev-local
```

## Available Commands

The deployer provides the following commands:

### Environment Management

- **[`create environment`](commands/create-environment.md)** - Create a new deployment environment from configuration
- **`create template`** - Generate configuration template file
- **[`destroy`](commands/destroy.md)** - Remove environment and clean up resources

### Future Commands

These commands are planned for future releases:

- **`deploy`** - Intelligent deployment orchestration (provision → configure → release)
- **`run`** - Start application services
- **`status`** - Check environment status and health
- **`test`** - Run validation tests

See the [Command Reference](commands.md) for complete documentation.

## Basic Workflows

### Development Workflow

Typical workflow for local development:

```bash
# 1. Generate and configure environment
torrust-tracker-deployer create template dev-config.json
# Edit dev-config.json with your values

# 2. Create the environment
torrust-tracker-deployer create environment --env-file dev-config.json

# 3. (Future) Deploy infrastructure
# torrust-tracker-deployer deploy dev-local

# 4. (Future) Start services
# torrust-tracker-deployer run dev-local

# 5. Clean up when done
torrust-tracker-deployer destroy dev-local
```

### Testing Workflow

Workflow for testing environments:

```bash
# Create test environment
torrust-tracker-deployer create template test-config.json
# Configure for test environment
torrust-tracker-deployer create environment --env-file test-config.json

# Run your tests...

# Clean up
torrust-tracker-deployer destroy test-env
```

### Multiple Environments

Managing multiple environments:

```bash
# Create development environment
torrust-tracker-deployer create environment --env-file dev-config.json

# Create staging environment
torrust-tracker-deployer create environment --env-file staging-config.json

# Work with either environment independently

# Clean up specific environment
torrust-tracker-deployer destroy dev-local
```

## Configuration

### Environment Configuration File

The environment configuration file is in JSON format:

```json
{
  "environment": {
    "name": "environment-name"
  },
  "ssh_credentials": {
    "private_key_path": "/path/to/private/key",
    "public_key_path": "/path/to/public/key",
    "username": "ssh-username",
    "port": 22
  }
}
```

#### Configuration Fields

**environment.name** (required):

- Unique identifier for the environment
- Must be lowercase alphanumeric with hyphens
- Used for directory names and resource identification
- Examples: `dev-local`, `staging`, `production-01`

**ssh_credentials.private_key_path** (required):

- Path to SSH private key file
- Supports `~` for home directory
- File must exist and be readable

**ssh_credentials.public_key_path** (required):

- Path to SSH public key file
- Supports `~` for home directory
- File must exist and be readable

**ssh_credentials.username** (required):

- SSH username for VM access
- Default: `torrust`

**ssh_credentials.port** (optional):

- SSH port number
- Default: `22`

### Logging Configuration

Control logging output with command-line options:

```bash
# Development mode - logs to both file and stderr
torrust-tracker-deployer create environment --env-file config.json \
  --log-output file-and-stderr

# Production mode - logs to file only (default)
torrust-tracker-deployer create environment --env-file config.json \
  --log-output file-only

# Change log format
torrust-tracker-deployer create environment --env-file config.json \
  --log-file-format json \
  --log-stderr-format pretty

# Custom log directory
torrust-tracker-deployer create environment --env-file config.json \
  --log-dir ./logs
```

**Logging Options**:

- `--log-output` - Where logs are written (`file-only`, `file-and-stderr`)
- `--log-file-format` - Format for file logs (`pretty`, `json`, `compact`)
- `--log-stderr-format` - Format for stderr logs (`pretty`, `json`, `compact`)
- `--log-dir` - Directory for log files (default: `./data/logs`)

See [Logging Guide](logging.md) for detailed information.

### Working Directory

By default, the deployer uses the current directory for all operations. You can specify a different working directory:

```bash
torrust-tracker-deployer create environment --env-file config.json \
  --working-dir /path/to/workspace
```

This affects:

- Environment state file location
- Data directory location
- Build directory location
- Log directory location (unless overridden with `--log-dir`)

## Troubleshooting

### Common Issues

#### Environment Already Exists

**Error**: `Environment 'name' already exists`

**Solution**: Choose a different name or destroy the existing environment first:

```bash
torrust-tracker-deployer destroy existing-name
torrust-tracker-deployer create environment --env-file config.json
```

#### SSH Key Not Found

**Error**: `SSH key not found at path`

**Solution**: Verify the key paths in your configuration file:

```bash
# Check if keys exist
ls -la ~/.ssh/id_rsa ~/.ssh/id_rsa.pub

# Generate new keys if needed
ssh-keygen -t rsa -b 4096 -f ~/.ssh/id_rsa
```

#### Invalid Environment Name

**Error**: `Invalid environment name`

**Solution**: Environment names must:

- Be lowercase
- Use alphanumeric characters and hyphens only
- Not start or end with a hyphen

Valid examples: `dev-local`, `staging-01`, `prod`

Invalid examples: `Dev-Local`, `staging_01`, `-dev`

#### Permission Denied

**Error**: `Permission denied when accessing directory`

**Solution**: Ensure you have write permissions for the working directory:

```bash
# Check permissions
ls -la .

# Fix permissions if needed
chmod 755 .
```

### Getting Help

For additional help:

1. **Check Command Documentation**: See the [Command Reference](commands.md)
2. **View Logs**: Check logs in `./data/logs/` for detailed error information
3. **Enable Verbose Logging**: Use `--log-output file-and-stderr` to see real-time logs
4. **Report Issues**: [GitHub Issues](https://github.com/torrust/torrust-tracker-deployer/issues)

### Verbose Logging

Enable verbose logging for troubleshooting:

```bash
torrust-tracker-deployer create environment --env-file config.json \
  --log-output file-and-stderr \
  --log-stderr-format pretty
```

This shows real-time progress and detailed error information.

## Additional Resources

### Documentation

- **[Command Reference](commands.md)** - Complete command documentation
- **[Logging Guide](logging.md)** - Logging configuration and best practices
- **[Template Customization](template-customization.md)** - Advanced configuration options

### Development Documentation

For contributors and developers:

- **[Architecture Overview](../codebase-architecture.md)** - System design and architecture
- **[Contributing Guidelines](../contributing/README.md)** - How to contribute
- **[Development Principles](../development-principles.md)** - Code quality standards

### Project Resources

- **[GitHub Repository](https://github.com/torrust/torrust-tracker-deployer)** - Source code and issues
- **[Roadmap](../roadmap.md)** - Future plans and features
- **[Changelog](../../CHANGELOG.md)** - Version history and changes

## Next Steps

Now that you understand the basics:

1. **Try the Quick Start** - Create your first environment
2. **Explore Commands** - Read the [Command Reference](commands.md)
3. **Configure Logging** - Set up logging that fits your workflow
4. **Report Feedback** - Share your experience on GitHub

Happy deploying! 🚀
