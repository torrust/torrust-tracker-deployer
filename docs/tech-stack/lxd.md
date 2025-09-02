# LXD System Containers

LXD is a next generation system container and virtual machine manager that provides a unified user experience around full Linux systems running inside containers or virtual machines.

## Overview

LXD system containers bridge the gap between traditional containers and VMs:

- **Like VMs**: Full init system (systemd), cloud-init support, multi-user environment
- **Like Containers**: Fast startup, efficient resource usage, no nested virtualization
- **Perfect for**: CI/CD testing of system-level configurations and Docker workloads

This approach gives you the VM-like experience you need for cloud-init testing while maintaining compatibility with CI environments.

## Installation

### Check if Already Installed

```bash
# Check if LXD is installed
lxd version
```

If this command returns version information, you can skip the installation steps below.

### Install LXD

Install via snap (recommended):

```bash
# On Ubuntu/Debian
sudo snap install lxd

# Initialize LXD (run once after installation)
sudo lxd init --auto

# Add your user to the lxd group
sudo usermod -a -G lxd $USER

# IMPORTANT: For group membership to take effect, you MUST do ONE of:
# Option 1 (Temporary): Use 'newgrp lxd' to start a new shell with the group active
# Option 2: Log out and log back in completely
# Option 3: Close terminal and open a new one
# Option 4 (Most Reliable): Reboot your system if other methods don't work

# Verify you're in the lxd group (should show your username)
getent group lxd | grep "$USER"

# Verify the group is active in your session (lxd should be in the list)
id -nG
```

## Proper LXD Group Setup

**The Problem**: LXD's client (`lxc`) connects to the daemon via a Unix socket that only members of the `lxd` group can access. Even if your user is in the `lxd` group, your current shell session might not have picked up that membership yet, resulting in "permission denied" errors.

**The Solution**: Based on the
[official LXD documentation](https://documentation.ubuntu.com/lxd/en/latest/tutorial/first_steps/#add-the-current-user-to-the-lxd-group),
follow these steps:

1. **Check if you're already in the lxd group**:

   ```bash
   getent group lxd | grep "$USER"
   ```

   If this returns your username, you're already in the group.

2. **Verify your current session has the group active**:

   ```bash
   id -nG
   ```

   If `lxd` is not in the list, your session hasn't picked up the group membership.

3. **If not in the group, add yourself**:

   ```bash
   sudo usermod -aG lxd "$USER"
   ```

4. **Activate the group membership** (choose ONE method):

   **Method 1 (Temporary)**: Start a subshell with the group active:

   ```bash
   newgrp lxd
   ```

   **Method 2 (Permanent)**: Log out and log back in completely

   **Method 3 (Permanent)**: Close your terminal and open a new one

   **Method 4 (Most Reliable)**: Reboot your system - this ensures all processes pick up the new group membership

   **Note**: In some cases, logging out/in or closing the terminal may not be sufficient, and a full system reboot may be required to properly activate the group membership.

5. **Verify the setup works**:

   ```bash
   lxc version
   ```

   If this works without `sudo` or permission errors, you're all set!

**Important**: The official LXD documentation states that if `lxd init --minimal` results in an error, "your group
membership might not have taken effect. In this case, close and re-open your terminal, then try again."

## Basic LXD Operations

### Container Management

```bash
# List all containers
lxc list

# Create a new container
lxc launch ubuntu:22.04 container-name

# Access a container
lxc exec container-name -- /bin/bash

# Stop a container
lxc stop container-name

# Start a container
lxc start container-name

# Delete a container
lxc delete container-name

# Get container information
lxc info container-name

# View container logs
lxc info container-name --show-log
```

### File Operations

```bash
# Copy files to container
lxc file push /local/path container-name/remote/path

# Copy files from container
lxc file pull container-name/remote/path /local/path
```

### Execute Commands Remotely

```bash
# Execute a single command
lxc exec container-name -- command

# Check system information
lxc exec container-name -- lsb_release -a

# Check systemd services (works in LXD system containers!)
lxc exec container-name -- systemctl status ssh

# Install packages
lxc exec container-name -- sudo apt update
lxc exec container-name -- sudo apt install -y package-name

# Check available disk space
lxc exec container-name -- df -h

# Check running processes
lxc exec container-name -- ps aux

# Check cloud-init status (if cloud-init is configured)
lxc exec container-name -- cloud-init status
```

## Docker Support

LXD system containers fully support Docker and Docker Compose:

```bash
# Docker can be installed and run inside LXD containers
lxc exec container-name -- sudo apt install -y docker.io
lxc exec container-name -- sudo systemctl enable docker
lxc exec container-name -- sudo systemctl start docker

# Test Docker functionality
lxc exec container-name -- sudo docker run hello-world
```

This makes LXD containers perfect for testing Docker Compose workflows without nested virtualization complexity.

## Troubleshooting

### Common Issues

1. **LXD not found**: Ensure LXD is installed via snap
2. **Permission errors**: Make sure your user is in the `lxd` group
3. **Socket permissions**: In CI environments, you may need to adjust socket permissions

### LXD Permission Issues

If you get `Error: LXD unix socket "/var/snap/lxd/common/lxd/unix.socket" not accessible: permission denied`, follow the [Proper LXD Group Setup](#proper-lxd-group-setup) section above. Quick fixes:

- **Most Reliable**: Reboot your system after adding yourself to the `lxd` group
- **Alternative**: Log out and log back in after adding yourself to the `lxd` group
- **Temporary**: Use `sg lxd -c "lxc command"` to run LXD commands with proper group access
- **Alternative**: Use `newgrp lxd` to activate the group in your current session
- **CI environments only**: As a temporary workaround (not recommended for regular use), you can use `sudo chmod 666 /var/snap/lxd/common/lxd/unix.socket` but this creates security risks
- **Last resort**: Use `sudo` with LXD commands, though this is not recommended for regular use

### Useful Commands

```bash
# Check LXD version
lxd version

# Troubleshoot LXD permissions
# Check if you're in the lxd group
groups $USER

# Check current active groups (lxd should be in this list)
id -nG

# If lxd is missing from id -nG, your session hasn't picked up group membership
# Fix with: newgrp lxd (temporary), log out/in, or reboot (most reliable)

# Check socket permissions
ls -la /var/snap/lxd/common/lxd/unix.socket

# If permission denied error occurs, try activating lxd group
newgrp lxd

# Or use sg (switch group) to run commands with proper group access
sg lxd -c "lxc list"
sg lxd -c "lxc info container-name"

# Or as a workaround, use sudo (not recommended for regular use)
sudo lxc list
```

## Why Use LXD System Containers?

### Advantages

- **No nested virtualization required**: Perfect for CI/CD environments
- **Faster startup than full VMs**: Containers start much quicker
- **Lower resource usage**: More efficient than traditional VMs
- **Full systemd support**: Unlike regular containers, system containers run a full init system
- **Complete cloud-init compatibility**: Full support for cloud-init configurations
- **Docker support**: Can run Docker and Docker Compose inside containers

### Use Cases

- **CI/CD testing**: Perfect for GitHub Actions and other CI environments
- **Development environments**: Quick, isolated development environments
- **System testing**: Test system-level configurations without VMs
- **Docker workflow testing**: Test Docker Compose setups without nested virtualization

### Considerations

- **Shared kernel with host**: Less isolation than VMs (container-level security)
- **Container-level security**: Not VM-level isolation
- **Requires LXD installation**: Need to install and configure LXD

## Further Reading

- [Official LXD Documentation](https://documentation.ubuntu.com/lxd/)
- [LXD Tutorial](https://documentation.ubuntu.com/lxd/en/latest/tutorial/)
- [LXD vs Docker](https://ubuntu.com/blog/lxd-vs-docker)
