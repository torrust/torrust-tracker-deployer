# Deploying with LXD (Local Development)

This guide explains how to deploy Torrust Tracker infrastructure locally using [LXD](https://canonical.com/lxd), a system container and virtual machine manager.

## Overview

LXD provides lightweight virtual machines that run on your local system. It's ideal for development, testing, and CI/CD pipelines where you want to test deployments without cloud costs.

### Why LXD?

- **Zero cloud costs**: Run everything locally
- **Fast iteration**: Quick VM creation and destruction
- **Consistent environments**: Same workflow as cloud deployments
- **CI/CD friendly**: Works in GitHub Actions and other CI systems

## Prerequisites

Before deploying with LXD, ensure you have:

1. **Linux System**: LXD runs on Linux (Ubuntu, Debian, Fedora, etc.)
2. **LXD Installed**: System container manager
3. **OpenTofu**: Infrastructure as Code tool
4. **Ansible**: Configuration management
5. **SSH Key Pair**: For VM access

### Installing Dependencies

```bash
# Verify all dependencies are installed
cargo run --bin dependency-installer check

# Install missing dependencies (includes LXD)
cargo run --bin dependency-installer install
```

## Step 1: Initialize LXD

If LXD isn't initialized yet:

```bash
# Quick initialization with defaults
sudo lxd init --auto

# Add your user to the lxd group
sudo usermod -a -G lxd $USER

# Apply group membership (or log out and back in)
newgrp lxd

# Verify LXD is working
lxc list
```

### Detailed LXD Configuration (Optional)

For custom configuration:

```bash
sudo lxd init
```

Answer the prompts:

- **Clustering**: No (for single-machine setup)
- **Storage pool**: dir (simplest) or zfs (better performance)
- **Network**: lxdbr0 (default bridge)
- **HTTPS server**: No (unless remote access needed)

## Step 2: Generate SSH Keys (if needed)

If you don't have SSH keys:

```bash
# Generate a new SSH key pair
ssh-keygen -t ed25519 -C "torrust-lxd" -f ~/.ssh/torrust_lxd

# Or use the test keys from fixtures (development only)
ls fixtures/testing_rsa*
```

## Step 3: Create Environment Configuration

Create a configuration file:

```bash
# Create configuration in the envs directory (git-ignored)
nano envs/my-local-env.json
```

**Example configuration**:

```json
{
  "environment": {
    "name": "my-local-env"
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-local"
  },
  "ssh_credentials": {
    "private_key_path": "/home/youruser/.ssh/torrust_lxd",
    "public_key_path": "/home/youruser/.ssh/torrust_lxd.pub",
    "username": "torrust",
    "port": 22
  }
}
```

### Configuration Fields

| Field                              | Description                           | Example                          |
| ---------------------------------- | ------------------------------------- | -------------------------------- |
| `environment.name`                 | Unique identifier for this deployment | `my-local-env`                   |
| `provider.provider`                | Must be `"lxd"`                       | `lxd`                            |
| `provider.profile_name`            | LXD profile name (auto-created)       | `torrust-profile-local`          |
| `ssh_credentials.private_key_path` | Path to SSH private key               | `/home/user/.ssh/id_ed25519`     |
| `ssh_credentials.public_key_path`  | Path to SSH public key                | `/home/user/.ssh/id_ed25519.pub` |
| `ssh_credentials.username`         | SSH user to create in VM              | `torrust`                        |
| `ssh_credentials.port`             | SSH port                              | `22`                             |

## Step 4: Create the Environment

```bash
torrust-tracker-deployer create environment --env-file envs/my-local-env.json
```

**Expected output**:

```text
✓ Validating configuration...
✓ Creating environment structure...
✓ Environment created successfully: my-local-env
```

## Step 5: Provision Infrastructure

Create the LXD virtual machine:

```bash
torrust-tracker-deployer provision my-local-env
```

**Expected output**:

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

1. Creates an LXD profile with VM configuration
2. Provisions a virtual machine instance
3. Cloud-init configures the VM with your SSH key
4. Waits for the VM to be fully ready

**Duration**: ~2-3 minutes

## Step 6: Configure Software

Install Docker and Docker Compose:

```bash
torrust-tracker-deployer configure my-local-env
```

**Expected output**:

```text
✓ Validating prerequisites...
✓ Running Ansible playbooks...
✓ Installing Docker...
✓ Installing Docker Compose...
✓ Configuring permissions...
✓ Verifying installation...
✓ Environment configured successfully
```

**Duration**: ~3-5 minutes

## Step 7: Verify Deployment

Test that everything works:

```bash
torrust-tracker-deployer test my-local-env
```

**Expected output**:

```text
✓ Validating environment state...
✓ Checking VM connectivity...
✓ Testing Docker installation...
✓ Testing Docker Compose...
✓ Verifying user permissions...
✓ Running infrastructure tests...
✓ All tests passed
```

## Step 8: Connect to Your VM

### Option 1: SSH (Recommended)

```bash
# Get the VM IP from the deployment output, or:
lxc list my-local-env

# Connect via SSH
ssh -i ~/.ssh/torrust_lxd torrust@<vm-ip>
```

### Option 2: LXC Console

```bash
# Direct console access
lxc exec torrust-tracker-vm-my-local-env -- bash
```

Once connected, verify Docker:

```bash
docker --version
docker compose version
docker ps
```

## Step 9: Clean Up

Destroy the infrastructure when done:

```bash
torrust-tracker-deployer destroy my-local-env
```

**Expected output**:

```text
✓ Stopping containers...
✓ Destroying infrastructure...
✓ Cleaning up resources...
✓ Environment destroyed successfully
```

## Managing LXD Resources

### List Resources

```bash
# List all instances
lxc list

# List profiles
lxc profile list

# List images
lxc image list
```

### Manual Cleanup

If you need to manually clean up:

```bash
# Delete an instance
lxc delete <instance-name> --force

# Delete a profile
lxc profile delete <profile-name>
```

## Troubleshooting

### LXD Not Running

**Error**: `Failed to connect to LXD`

**Solution**:

```bash
# Check LXD status
sudo systemctl status snap.lxd.daemon

# Restart LXD
sudo systemctl restart snap.lxd.daemon

# Or if installed via apt
sudo systemctl restart lxd
```

### Permission Denied

**Error**: `Permission denied while trying to connect to LXD`

**Solution**:

```bash
# Add user to lxd group
sudo usermod -a -G lxd $USER

# Apply group membership
newgrp lxd

# Or log out and back in
```

### Network Issues

**Error**: `Instance has no network`

**Solution**:

```bash
# Check network bridge
lxc network list

# Recreate default bridge if needed
lxc network delete lxdbr0
lxc network create lxdbr0
```

### VM Won't Start

**Error**: `Failed to start instance`

**Solution**:

```bash
# Check for resource limits
lxc config show <instance-name>

# Check system resources
free -h
df -h

# Check LXD logs
sudo journalctl -u snap.lxd.daemon -n 100
```

### SSH Connection Timeout

**Error**: `Failed to connect via SSH`

**Solution**:

```bash
# Check if VM is running
lxc list

# Wait for cloud-init
lxc exec <instance-name> -- cloud-init status --wait

# Check SSH service
lxc exec <instance-name> -- systemctl status ssh

# Verify SSH key
lxc exec <instance-name> -- cat /home/torrust/.ssh/authorized_keys
```

## Performance Tips

### Use ZFS Storage

ZFS provides better performance than directory storage:

```bash
# Create a ZFS pool (requires ZFS installed)
sudo lxd init
# Choose 'zfs' for storage backend
```

### Allocate More Resources

For better VM performance:

```bash
# Check current limits
lxc config show <instance-name>

# Increase CPU (runtime)
lxc config set <instance-name> limits.cpu 4

# Increase memory (runtime)
lxc config set <instance-name> limits.memory 4GB
```

### Cache Images

Images are cached after first use, speeding up subsequent deployments.

## Resource Requirements

Minimum system requirements for LXD:

| Resource | Minimum                  | Recommended   |
| -------- | ------------------------ | ------------- |
| RAM      | 4 GB                     | 8+ GB         |
| CPU      | 2 cores                  | 4+ cores      |
| Storage  | 20 GB                    | 50+ GB        |
| OS       | Linux (any major distro) | Ubuntu 22.04+ |

## Next Steps

After successful deployment:

1. **Deploy Torrust Tracker**: Follow the Torrust Tracker deployment guide
2. **Test locally**: Develop and test your tracker configuration
3. **Move to production**: Use [Hetzner](hetzner.md) or another cloud provider

## Related Documentation

- [Quick Start Guide](../quick-start.md) - General deployment workflow
- [Command Reference](../commands/README.md) - Detailed command documentation
- [Hetzner Provider](hetzner.md) - Cloud deployment option
- [Template Customization](../template-customization.md) - Customize deployment templates
