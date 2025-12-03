# `provision` - Provision VM Infrastructure

Provision virtual machine infrastructure for a deployment environment.

## Purpose

Creates and configures VM infrastructure using OpenTofu (Terraform). This command takes an environment from the "Created" state to the "Provisioned" state with running VM instances.

The provision command works with all supported providers:

- **LXD** - Creates local VMs for development and testing
- **Hetzner Cloud** - Creates cloud servers for production deployments

## Command Syntax

```bash
torrust-tracker-deployer provision <ENVIRONMENT>
```

## Arguments

- `<ENVIRONMENT>` (required) - Name of the environment to provision

## Prerequisites

1. **Environment created** - Must run `create environment` first
2. **Provider-specific requirements**:
   - **LXD**: Local LXD installation configured
   - **Hetzner**: Valid API token in environment configuration
3. **OpenTofu installed** - OpenTofu CLI available in PATH
4. **SSH keys** - SSH key pair referenced in environment configuration

📖 **See [Provider Guides](../providers/README.md)** for provider-specific setup.

## State Transition

```text
[Created] --provision--> [Provisioned]
```

## What Happens

When you provision an environment:

1. **Renders OpenTofu templates** - Generates provider-specific infrastructure-as-code files
2. **Initializes OpenTofu** - Sets up backend and providers (`tofu init`)
3. **Creates execution plan** - Validates configuration (`tofu plan`)
4. **Applies infrastructure** - Creates VM resources (`tofu apply`)
5. **Retrieves instance info** - Gets IP address and instance details
6. **Renders Ansible templates** - Generates configuration management files
7. **Waits for SSH** - Verifies network connectivity
8. **Waits for cloud-init** - Ensures VM initialization is complete
9. **Updates environment state** - Transitions to "Provisioned"

## Examples

### Basic provisioning

```bash
# Provision the environment
torrust-tracker-deployer provision my-environment

# Output:
# ✓ Rendering OpenTofu templates...
# ✓ Initializing infrastructure...
# ✓ Planning infrastructure changes...
# ✓ Applying infrastructure...
# ✓ Retrieving instance information...
# ✓ Instance IP: 10.140.190.42
# ✓ Rendering Ansible templates...
# ✓ Waiting for SSH connectivity...
# ✓ Waiting for cloud-init completion...
# ✓ Environment provisioned successfully
```

### Provision multiple environments

```bash
# Development
torrust-tracker-deployer provision dev-local

# Staging
torrust-tracker-deployer provision staging

# Production (Hetzner)
torrust-tracker-deployer provision production
```

## Output

The provision command creates provider-specific resources:

### LXD Provider

- **VM instance** - LXD virtual machine (`torrust-tracker-vm-<env-name>`)
- **LXD profile** - Custom profile with cloud-init configuration
- **Network configuration** - Bridged network with IP assignment
- **OpenTofu state** - Infrastructure state in `build/<env>/tofu/lxd/`

### Hetzner Provider

- **Cloud server** - Hetzner Cloud server instance
- **Firewall** - Hetzner firewall with SSH access
- **SSH key** - Uploaded SSH public key
- **OpenTofu state** - Infrastructure state in `build/<env>/tofu/hetzner/`

### Common Outputs (All Providers)

- **Ansible inventory** - Generated inventory in `build/<env>/ansible/`
- **Environment state update** - State file updated to "Provisioned"

## Next Steps

After provisioning:

```bash
# 1. Configure the infrastructure (install Docker, Docker Compose)
torrust-tracker-deployer configure my-environment

# 2. Verify infrastructure readiness
torrust-tracker-deployer test my-environment
```

## Troubleshooting

### Environment not found

**Problem**: Cannot find environment with the specified name

**Solution**: Verify the environment was created

```bash
# Check environment data directory exists
ls -la data/my-environment/

# If not, create the environment first
torrust-tracker-deployer create environment -f config.json
```

### LXD not initialized (LXD provider only)

**Problem**: LXD is not properly initialized

**Solution**: Initialize LXD

```bash
# Initialize LXD with default settings
sudo lxd init --auto

# Add your user to lxd group
sudo usermod -a -G lxd $USER
newgrp lxd
```

### OpenTofu not found

**Problem**: OpenTofu CLI is not installed or not in PATH

**Solution**: Install OpenTofu

```bash
# Install OpenTofu
curl -fsSL https://get.opentofu.org/install-opentofu.sh | sudo bash

# Verify installation
tofu version
```

### SSH connection timeout

**Problem**: Cannot establish SSH connection to provisioned VM

**Solution**: Check network connectivity and cloud-init status

```bash
# Get VM IP address
lxc list

# Try to connect manually
ssh -i <path-to-private-key> torrust@<vm-ip>

# Check cloud-init status
lxc exec <instance-name> -- cloud-init status
```

### Port already in use

**Problem**: LXD profile or instance name already exists

**Solution**: Clean up existing resources

```bash
# List existing instances
lxc list

# Delete old instance if needed
lxc delete <instance-name> --force

# List profiles
lxc profile list

# Delete old profile if needed
lxc profile delete <profile-name>
```

## Common Use Cases

### Quick local development

```bash
# Create, provision, and configure in sequence
torrust-tracker-deployer create environment -f dev.json
torrust-tracker-deployer provision dev-local
torrust-tracker-deployer configure dev-local
```

### CI/CD pipeline

```bash
#!/bin/bash
set -e

ENV_NAME="ci-${CI_JOB_ID}"

# Create environment
torrust-tracker-deployer create environment -f ci-config.json

# Provision infrastructure
torrust-tracker-deployer provision ${ENV_NAME}

# Run tests...
# Cleanup is handled by destroy command
```

### Reprovisioning

If you need to reprovision (destroy and create again):

```bash
# Destroy existing environment
torrust-tracker-deployer destroy my-environment

# Create fresh environment
torrust-tracker-deployer create environment -f config.json

# Provision again
torrust-tracker-deployer provision my-environment
```

## Technical Details

### Generated Resources

**LXD Resources**:

- Instance: `torrust-tracker-vm-<environment-name>`
- Profile: `torrust-profile-<environment-name>`
- Network: Bridged network with DHCP

**File Artifacts**:

- OpenTofu files: `build/<env>/tofu/lxd/`
- Ansible inventory: `build/<env>/ansible/inventory.yml`
- Instance info: Stored in environment state

### Cloud-Init Configuration

The provisioned VM includes cloud-init configuration for:

- User account creation (SSH username from config)
- SSH key deployment (public key from config)
- Network configuration
- Initial system setup

## See Also

- [create](create.md) - Create environment (prerequisite)
- [configure](configure.md) - Configure infrastructure (next step)
- [test](test.md) - Verify infrastructure
- [destroy](destroy.md) - Clean up infrastructure
