# LXD OpenTofu Configuration

This OpenTofu configuration provisions **LXD system containers** to create VM-like environments with cloud-init support, without requiring nested virtualization.

## Overview

This configuration creates:

- An LXD system container with Ubuntu 22.04 LTS
- Full systemd support (unlike regular containers)
- Basic cloud-init setup with essential packages
- Network isolation with container networking
- 10GB disk allocation

For general information about LXD system containers, see the [LXD documentation](../../docs/tech-stack/lxd.md).

## Prerequisites

Before provisioning, ensure you have:

### Required Software

1. **LXD**: See [LXD installation guide](../../docs/tech-stack/lxd.md#installation) for detailed setup instructions
2. **OpenTofu**: Install from [https://opentofu.org/](https://opentofu.org/)

   ```bash
   # Using package manager (example for Ubuntu/Debian)
   curl --proto '=https' --tlsv1.2 -fsSL https://get.opentofu.org/install-opentofu.sh -o install-opentofu.sh
   chmod +x install-opentofu.sh
   ./install-opentofu.sh --install-method deb
   ```

### Check Installation

```bash
# Check if LXD is installed and accessible
lxd version

# Check if OpenTofu is installed
tofu version
```

If both commands return version information, you can proceed to [Configuration](#configuration).

## Configuration

### LXD Access Setup

Ensure your user has proper access to LXD. If you encounter permission errors, see the [LXD Group Setup guide](../../docs/tech-stack/lxd.md#proper-lxd-group-setup) for detailed instructions.

### Configuration Files

This OpenTofu configuration consists of:

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

After provisioning, you can manage the `torrust-vm` container using standard LXD commands:

### Access the Container

```bash
# Direct shell access
lxc exec torrust-vm -- /bin/bash

# If you have LXD permission issues, use:
sg lxd -c "lxc exec torrust-vm -- /bin/bash"

# SSH access (if you configured your SSH key and networking)
ssh torrust@<container-ip-address>
```

### Check Container Status

```bash
# Check the status of our specific container
lxc info torrust-vm

# List all containers (including torrust-vm)
lxc list
```

### Container Lifecycle

```bash
# Stop the container
lxc stop torrust-vm

# Start the container
lxc start torrust-vm

# Restart the container
lxc restart torrust-vm
```

### Common Operations

```bash
# Check if cloud-init provisioning completed
lxc exec torrust-vm -- cat /tmp/provision_complete

# Check system information
lxc exec torrust-vm -- lsb_release -a

# Check systemd services (works in LXD system containers!)
lxc exec torrust-vm -- systemctl status ssh

# Install additional packages (if needed during development)
lxc exec torrust-vm -- sudo apt update
lxc exec torrust-vm -- sudo apt install -y git curl wget htop vim

# Check available disk space
lxc exec torrust-vm -- df -h

# Check running processes
lxc exec torrust-vm -- ps aux

# Check cloud-init status
lxc exec torrust-vm -- cloud-init status
```

For more LXD commands and troubleshooting, see the [LXD documentation](../../docs/tech-stack/lxd.md).

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

### OpenTofu-Specific Issues

1. **OpenTofu not found**: Ensure OpenTofu is installed and in your PATH
2. **LXD provider errors**: Verify LXD is running and accessible
3. **Permission errors**: See [LXD Group Setup guide](../../docs/tech-stack/lxd.md#proper-lxd-group-setup)
4. **Container creation fails**: Check if the specified image is available

### LXD Permission Issues

If you encounter LXD socket permission errors during `tofu apply` or `tofu destroy`, use:

```bash
# Run OpenTofu commands with proper LXD group access
sg lxd -c "tofu apply"
sg lxd -c "tofu destroy"
```

For detailed LXD troubleshooting, see the [LXD documentation](../../docs/tech-stack/lxd.md#troubleshooting).

## GitHub Actions Support

✅ **Status**: Guaranteed compatibility

This OpenTofu configuration is designed specifically for CI/CD environments like GitHub Actions where nested virtualization is not available:

- **Workflow**: `.github/workflows/test-lxd-provision.yml`
- **Status**: Fully supported and tested
- **Requirements**: Works in standard GitHub Actions runners

**Important Note**: The GitHub workflow uses `sudo chmod 666` on the LXD socket as a workaround for CI environments where terminal restarts aren't practical. **This approach is not recommended for local development** due to security implications. For local use, follow the proper group membership approach described in the [LXD documentation](../../docs/tech-stack/lxd.md#proper-lxd-group-setup).

### Configuration Benefits for CI

- No nested virtualization required
- Guaranteed GitHub Actions compatibility
- Faster startup than full VMs
- Lower resource usage
- Full systemd support for system-level testing

## Additional Information

For more information about LXD system containers, including Docker support, general capabilities, and detailed troubleshooting, see the [LXD documentation](../../docs/tech-stack/lxd.md).
