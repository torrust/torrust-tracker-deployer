# Quick Start Guide

Get up and running with Torrust Tracker Deployer in minutes.

## Prerequisites

- **LXD** - Local LXD installation configured (for local development)
- **Hetzner Account** - Hetzner Cloud account with API token (for production)
- **OpenTofu** - OpenTofu CLI installed
- **Ansible** - Ansible for configuration management
- **SSH keys** - SSH key pair for VM access

> **Tip**: Run `cargo run --bin dependency-installer check` to verify all prerequisites are met.

## Choose Your Provider

Before starting, decide which provider to use:

| Provider          | Best For                          | Requirements                |
| ----------------- | --------------------------------- | --------------------------- |
| **LXD**           | Local development, CI/CD, testing | Linux with LXD installed    |
| **Hetzner Cloud** | Production deployments            | Hetzner account + API token |

📖 **See [Provider Guides](providers/README.md)** for detailed setup instructions.

## Installation

```bash
# Install dependencies automatically
cargo run --bin dependency-installer install

# Verify installation
cargo run --bin dependency-installer check
```

> **Note**: For manual installation instructions, see the tool-specific documentation in `docs/tech-stack/`.

## Complete Workflow

This example walks through the complete deployment lifecycle from template generation to infrastructure testing.

### Step 1: Generate Environment Template

Create a template configuration file for your chosen provider:

**For LXD (local development)**:

```bash
torrust-tracker-deployer create template --provider lxd my-environment.json
```

**For Hetzner Cloud (production)**:

```bash
torrust-tracker-deployer create template --provider hetzner my-environment.json
```

**Output**:

```text
✓ Template generated: my-environment.json
```

This creates a pre-filled template with provider-specific values that you can customize.

### Step 2: Customize Configuration

Edit the generated template:

```bash
nano my-environment.json
```

**Example LXD configuration**:

```json
{
  "environment": {
    "name": "my-environment"
  },
  "ssh_credentials": {
    "private_key_path": "fixtures/testing_rsa",
    "public_key_path": "fixtures/testing_rsa.pub",
    "username": "torrust",
    "port": 22
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-local"
  }
}
```

**Example Hetzner configuration**:

```json
{
  "environment": {
    "name": "my-production-env"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_ed25519",
    "public_key_path": "~/.ssh/id_ed25519.pub",
    "username": "torrust",
    "port": 22
  },
  "provider": {
    "provider": "hetzner",
    "api_token": "your-hetzner-api-token-here",
    "server_type": "cx22",
    "location": "nbg1"
  }
}
```

> **Note**: For LXD testing, use the test SSH keys from `fixtures/` directory. For production, use your own SSH keys (e.g., `~/.ssh/id_ed25519`).

**Key fields to customize**:

- `environment.name` - Environment identifier (must be unique)
- `ssh_credentials.private_key_path` - Path to your SSH private key file
- `ssh_credentials.public_key_path` - Path to your SSH public key file
- `ssh_credentials.username` - SSH username for VM access (default: torrust)
- `ssh_credentials.port` - SSH port (default: 22)
- `provider` - Provider-specific configuration (see [Provider Guides](providers/README.md))

### Step 3: Create Environment

Generate the deployment environment from your template:

```bash
torrust-tracker-deployer create environment --env-file my-environment.json
```

**Output**:

```text
✓ Validating configuration...
✓ Creating environment structure...
✓ Environment created successfully: my-environment
```

This creates the environment directory structure and validates your configuration.

### Step 4: Provision Infrastructure

Create and configure VM infrastructure:

```bash
torrust-tracker-deployer provision my-environment
```

**Output**:

```text
✓ Rendering OpenTofu templates...
✓ Initializing infrastructure...
✓ Planning infrastructure changes...
✓ Applying infrastructure...
✓ Retrieving instance information...
✓ Instance IP: 10.140.190.42
✓ Rendering Ansible templates...
✓ Waiting for SSH connectivity...
✓ Waiting for cloud-init completion...
✓ Environment provisioned successfully
```

**What happens**:

- Creates LXD VM instance
- Configures network and storage
- Deploys SSH keys
- Waits for VM initialization

**Duration**: ~2-3 minutes (depending on your system)

### Step 5: Configure Software

Install Docker and Docker Compose on the provisioned VM:

```bash
torrust-tracker-deployer configure my-environment
```

**Output**:

```text
✓ Validating prerequisites...
✓ Running Ansible playbooks...
✓ Installing Docker...
✓ Installing Docker Compose...
✓ Configuring permissions...
✓ Verifying installation...
✓ Environment configured successfully
```

**What happens**:

- Installs Docker Engine
- Installs Docker Compose plugin
- Adds SSH user to docker group
- Verifies installation

**Duration**: ~3-5 minutes (depending on network speed)

### Step 6: Verify Infrastructure

Test that everything is working correctly:

```bash
torrust-tracker-deployer test my-environment
```

**Output**:

```text
✓ Validating environment state...
✓ Checking VM connectivity...
✓ Testing Docker installation...
✓ Testing Docker Compose...
✓ Verifying user permissions...
✓ Running infrastructure tests...
✓ All tests passed
```

**What is tested**:

- SSH connectivity
- Docker daemon running
- Docker CLI accessible
- Docker Compose available
- Non-root Docker access

### Step 7: Clean Up

When you're done, destroy the environment:

```bash
torrust-tracker-deployer destroy my-environment
```

**Output**:

```text
✓ Stopping containers...
✓ Destroying infrastructure...
✓ Cleaning up resources...
✓ Environment destroyed successfully
```

**What happens**:

- Stops all running containers
- Destroys LXD VM instance
- Removes LXD profile
- Cleans up OpenTofu state

## Quick Reference

### One-line Setup

```bash
# Create template, edit it, then provision, configure, and test
torrust-tracker-deployer create template dev.json && \
  # Edit dev.json with your SSH keys and settings, then:
  torrust-tracker-deployer create environment --env-file dev.json && \
  torrust-tracker-deployer provision dev && \
  torrust-tracker-deployer configure dev && \
  torrust-tracker-deployer test dev
```

### Common Commands

```bash
# Check dependencies
cargo run --bin dependency-installer check

# Create template
torrust-tracker-deployer create template <output-path>

# Create environment
torrust-tracker-deployer create environment --env-file <config-file>

# Provision infrastructure
torrust-tracker-deployer provision <environment>

# Configure software
torrust-tracker-deployer configure <environment>

# Verify infrastructure
torrust-tracker-deployer test <environment>

# Clean up
torrust-tracker-deployer destroy <environment>
```

## Troubleshooting

### LXD not initialized

**Error**: `Failed to connect to LXD`

**Solution**:

```bash
sudo lxd init --auto
sudo usermod -a -G lxd $USER
newgrp lxd
```

### SSH connection timeout

**Error**: `Failed to connect via SSH`

**Solution**:

```bash
# Check VM status
lxc list

# Verify cloud-init completed
lxc exec <instance-name> -- cloud-init status --wait

# Check SSH key permissions
chmod 600 /path/to/private/key
```

### Docker not found after configuration

**Error**: `Docker command not found`

**Solution**:

```bash
# SSH into VM
ssh -i <private-key> torrust@<vm-ip>

# Check Docker status
sudo systemctl status docker

# Restart Docker if needed
sudo systemctl restart docker

# Re-run configuration
torrust-tracker-deployer configure my-environment
```

### Port already in use

**Error**: `Port 22 already in use`

**Solution**:

```bash
# List existing containers
lxc list

# Remove old instance
lxc delete <instance-name> --force

# Try again
torrust-tracker-deployer provision my-environment
```

## What's Next?

After completing this quick start:

1. **Read the user guides** - Learn more about each command in `docs/user-guide/commands/`
2. **Explore advanced configuration** - Customize your deployments
3. **Integrate with CI/CD** - Automate your deployment pipeline
4. **Review troubleshooting** - Understand common issues and solutions

## Additional Resources

- [Command Reference](commands/README.md) - Detailed documentation for all commands
- [Architecture Guide](../codebase-architecture.md) - Understanding the codebase
- [Contributing Guide](../contributing/README.md) - Contributing to the project
- [Console Commands](../console-commands.md) - Technical command reference

## Getting Help

If you encounter issues:

1. Check the troubleshooting section above
2. Review the command-specific guides in `docs/user-guide/commands/`
3. Check the [known issues](../contributing/known-issues.md) documentation
4. Open an issue on GitHub with:
   - Steps to reproduce
   - Error messages
   - Environment details (OS, LXD version, etc.)
