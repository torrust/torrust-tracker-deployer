# LXD Container Configuration

This configuration uses **LXD system containers** to create VM-like environments with cloud-init support, without requiring nested virtualization.

## Overview

- **Technology**: LXD system containers
- **Virtualization**: Container-based with systemd support
- **Best for**: GitHub Actions and CI environments, or local development without nested virtualization
- **Requirements**: No nested virtualization needed
- **Cloud-init**: Full support in container boot process
- **Docker Support**: Can run Docker Compose inside containers

## Prerequisites

Before you can provision containers, ensure you have the following installed:

### Check if Already Installed

```bash
# Check if LXD is installed
lxd version

# Check if OpenTofu is installed
tofu version
```

If both commands return version information, you can skip the installation steps below and go directly to [Provisioning](#provisioning).

### Installation

1. **LXD**: Install via snap (recommended)

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

2. **OpenTofu**: Install from [https://opentofu.org/](https://opentofu.org/)

   ```bash
   # Using package manager (example for Ubuntu/Debian)
   curl --proto '=https' --tlsv1.2 -fsSL https://get.opentofu.org/install-opentofu.sh -o install-opentofu.sh
   chmod +x install-opentofu.sh
   ./install-opentofu.sh --install-method deb
   ```

## Configuration

### Proper LXD Group Setup (Required for Direct LXD Access)

**The Problem**: LXD's client (`lxc`) connects to the daemon via a Unix socket that only members of the `lxd` group can access. Even if your user is in the `lxd` group, your current shell session might not have picked up that membership yet, resulting in "permission denied" errors.

**The Solution**: Based on the [official LXD documentation](https://documentation.ubuntu.com/lxd/en/latest/tutorial/first_steps/#add-the-current-user-to-the-lxd-group), follow these steps:

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

**Important**: The official LXD documentation states that if `lxd init --minimal` results in an error, "your group membership might not have taken effect. In this case, close and re-open your terminal, then try again."

### Container Configuration Files

The LXD configuration consists of:

- **`main.tf`** - OpenTofu configuration defining the LXD container and profile
- **`cloud-init.yml`** - Cloud-init configuration for container initialization

The setup includes:

- An LXD system container with Ubuntu 22.04 LTS
- Full systemd support (unlike regular containers)
- Basic cloud-init setup with essential packages
- Network isolation with container networking
- 10GB disk allocation

### Customization

Before provisioning, you may want to customize:

1. **SSH Key**: Edit the `cloud-init.yml` file and replace the SSH key with your actual public key
2. **Container Specifications**: Adjust resource limits in `main.tf`
3. **Container Name**: Change the instance name from "torrust-vm" to your preferred name
4. **Packages**: Modify the packages list in `cloud-init.yml` to include additional software

## Provisioning

To provision the container:

1. **Navigate to the LXD configuration directory**:

   ```bash
   cd config/lxd
   ```

2. **Initialize OpenTofu**:

   ```bash
   tofu init
   ```

3. **Plan the deployment** (optional, to see what will be created):

   ```bash
   tofu plan
   ```

4. **Apply the configuration**:

   ```bash
   tofu apply
   ```

   Type `yes` when prompted to confirm the creation.

   **Note**: If you encounter LXD socket permission issues, use:

   ```bash
   sg lxd -c "tofu apply"
   ```

   Example successful output:

   ```text
   Apply complete! Resources: 2 added, 0 changed, 0 destroyed.

   Outputs:
   container_info = {
     "image" = "ubuntu:22.04"
     "ip_address" = "10.140.190.155"
     "name" = "torrust-vm"
     "status" = "Running"
   }
   ```

5. **Get container information**:

   ```bash
   tofu output
   ```

6. **Access the container**:

   ```bash
   # Direct shell access
   lxc exec torrust-vm -- /bin/bash

   # If you have LXD permission issues, use:
   sg lxd -c "lxc exec torrust-vm -- /bin/bash"

   # SSH access (if you configured your SSH key and networking)
   ssh torrust@<container-ip-address>
   ```

## Managing the Container

### Access the Container

```bash
# Using lxc directly
lxc exec torrust-vm -- /bin/bash

# If you have permission issues, use sg to switch to lxd group
sg lxd -c "lxc exec torrust-vm -- /bin/bash"

# Or via SSH (if you configured your SSH key)
ssh torrust@<container-ip-address>
```

### Check Container Status

```bash
# Preferred (if group membership is active)
lxc list

# If you get permission denied, use:
sg lxd -c "lxc list"
```

### Stop the Container

```bash
lxc stop torrust-vm
```

### Start the Container

```bash
lxc start torrust-vm
```

### Execute Commands Remotely

```bash
# Check if cloud-init provisioning completed
lxc exec torrust-vm -- cat /tmp/provision_complete

# Check system information
lxc exec torrust-vm -- lsb_release -a

# Check systemd services (works in LXD system containers!)
lxc exec torrust-vm -- systemctl status ssh

# Install packages manually (if needed during development)
lxc exec torrust-vm -- sudo apt update
lxc exec torrust-vm -- sudo apt install -y git curl wget htop vim

# Check available disk space
lxc exec torrust-vm -- df -h

# Check running processes
lxc exec torrust-vm -- ps aux

# Check cloud-init status
lxc exec torrust-vm -- cloud-init status
```

## Cleanup

To destroy the container and clean up resources:

1. **Navigate to the configuration directory** (if not already there):

   ```bash
   cd config/lxd
   ```

2. **Destroy the infrastructure**:

   ```bash
   tofu destroy
   ```

   Type `yes` when prompted to confirm the destruction.

   **Note**: If you encounter LXD permission issues, use:

   ```bash
   sg lxd -c "tofu destroy"
   ```

## Troubleshooting

### Common Issues

1. **LXD not found**: Ensure LXD is installed via snap
2. **Permission errors**: Make sure your user is in the `lxd` group
3. **Socket permissions**: In CI environments, you may need to adjust socket permissions
4. **LXD unix socket not accessible**: If you get `Error: LXD unix socket "/var/snap/lxd/common/lxd/unix.socket" not accessible: permission denied`, follow the [Proper LXD Group Setup](#proper-lxd-group-setup-required-for-direct-lxd-access) section above. Quick fixes:
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

# List all containers
lxc list

# Get container info
lxc info torrust-vm

# View container logs
lxc info torrust-vm --show-log

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
sg lxd -c "lxc info torrust-vm"

# Or as a workaround, use sudo (not recommended for regular use)
sudo lxc list
```

## GitHub Actions Support

✅ **Status**: Guaranteed compatibility

This configuration is designed specifically for CI/CD environments like GitHub Actions where nested virtualization is not available or reliable:

- **Workflow**: `.github/workflows/test-lxd-provision.yml`
- **Status**: Fully supported and tested
- **No special requirements**: Works in standard GitHub Actions runners

**Important Note**: The GitHub workflow uses `sudo chmod 666` on the LXD socket as a workaround for CI environments where terminal restarts aren't practical. **This approach is not recommended for local development** due to security implications. For local use, follow the proper group membership approach described in the troubleshooting section.

### Pros and Cons

**✅ Advantages:**

- No nested virtualization required
- Guaranteed GitHub Actions compatibility
- Faster startup than full VMs
- Lower resource usage
- Full systemd support (unlike regular containers)
- Complete cloud-init compatibility

**⚠️ Considerations:**

- Shared kernel with host (less isolation than VMs)
- Container-level security (not VM-level)
- Requires LXD installation and setup

## Docker Support

LXD system containers fully support Docker and Docker Compose:

```bash
# Docker can be installed and run inside LXD containers
lxc exec torrust-vm -- sudo apt install -y docker.io
lxc exec torrust-vm -- sudo systemctl enable docker
lxc exec torrust-vm -- sudo systemctl start docker

# Test Docker functionality
lxc exec torrust-vm -- sudo docker run hello-world
```

This makes LXD containers perfect for testing Docker Compose workflows without nested virtualization complexity.

## Why LXD System Containers?

LXD system containers bridge the gap between traditional containers and VMs:

- **Like VMs**: Full init system (systemd), cloud-init support, multi-user environment
- **Like Containers**: Fast startup, efficient resource usage, no nested virtualization
- **Perfect for**: CI/CD testing of system-level configurations and Docker workloads

This approach gives you the VM-like experience you need for cloud-init testing while maintaining compatibility with CI environments.
