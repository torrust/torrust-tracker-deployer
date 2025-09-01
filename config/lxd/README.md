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

   # You may need to log out and back in for group changes to take effect
   ```

2. **OpenTofu**: Install from [https://opentofu.org/](https://opentofu.org/)

   ```bash
   # Using package manager (example for Ubuntu/Debian)
   curl --proto '=https' --tlsv1.2 -fsSL https://get.opentofu.org/install-opentofu.sh -o install-opentofu.sh
   chmod +x install-opentofu.sh
   ./install-opentofu.sh --install-method deb
   ```

## Configuration

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

5. **Get container information**:

   ```bash
   tofu output
   ```

6. **Access the container**:

   ```bash
   # Direct shell access
   lxc exec torrust-vm -- /bin/bash

   # SSH access (if you configured your SSH key and networking)
   ssh torrust@<container-ip-address>
   ```

## Managing the Container

### Access the Container

```bash
# Using lxc directly
lxc exec torrust-vm -- /bin/bash

# Or via SSH (if you configured your SSH key)
ssh torrust@<container-ip-address>
```

### Check Container Status

```bash
lxc list
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

## Troubleshooting

### Common Issues

1. **LXD not found**: Ensure LXD is installed via snap
2. **Permission errors**: Make sure your user is in the `lxd` group
3. **Socket permissions**: In CI environments, you may need to adjust socket permissions

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
```

## GitHub Actions Support

✅ **Status**: Guaranteed compatibility

This configuration is designed specifically for CI/CD environments like GitHub Actions where nested virtualization is not available or reliable:

- **Workflow**: `.github/workflows/test-lxd-provision.yml`
- **Status**: Fully supported and tested
- **No special requirements**: Works in standard GitHub Actions runners

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
