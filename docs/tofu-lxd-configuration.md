# LXD OpenTofu Configuration

This OpenTofu configuration provisions **LXD virtual machines** to create full VM environments with cloud-init support and complete kernel isolation.

## Overview

This configuration creates:

- An LXD virtual machine with Ubuntu 24.04 LTS
- Complete kernel isolation (true virtualization)
- Full systemd and cloud-init support
- Network isolation with VM networking
- 10GB disk allocation
- 2GB RAM and 2 CPU cores

For the architectural decision rationale, see [LXD VMs over Containers Decision](decisions/lxd-vm-over-containers.md).

For general information about LXD virtual machines vs containers, see the [Instance Types](#instance-types-virtual-machines-vs-containers) section below.

## Instance Types: Virtual Machines vs Containers

LXD supports two types of instances:

### Virtual Machines (Current Configuration)

- **Type**: `virtual-machine`
- **Isolation**: Full kernel isolation with separate kernel
- **Boot time**: ~20-30 seconds (proper kernel boot)
- **Resource overhead**: Higher (2GB RAM minimum recommended)
- **Networking**: Full network stack with proper interface names (`enp5s0`, etc.)
- **Security**: Strong isolation through hardware virtualization
- **Cloud-init**: Full compatibility with all cloud-init features
- **Use cases**: Production workloads, kernel-dependent software, maximum isolation

### Containers (Alternative)

- **Type**: `container`
- **Isolation**: Namespace isolation (shared kernel)
- **Boot time**: ~2-5 seconds (process startup)
- **Resource overhead**: Lower (can run with minimal RAM)
- **Networking**: Virtual networking with predictable names (`eth0`)
- **Security**: Good isolation but shared kernel
- **Cloud-init**: Limited compatibility
- **Use cases**: Development, testing, lightweight workloads

### Performance Comparison

Based on E2E test results:

| Instance Type   | E2E Test Time   | Boot Time | Resource Usage       | Isolation Level  |
| --------------- | --------------- | --------- | -------------------- | ---------------- |
| Virtual Machine | **~52 seconds** | ~20-30s   | 2GB RAM, 2 CPU       | Full (Hardware)  |
| Container       | ~85 seconds     | ~2-5s     | <1GB RAM, Shared CPU | Good (Namespace) |

**Virtual machines are ~37% faster** for complete deployment workflows due to:

- More predictable boot sequence
- Better cloud-init integration
- Fewer networking conflicts
- More robust systemd environment

### Architecture Decision: Why Virtual Machines?

This project uses **virtual machines** as the primary instance type for strategic reasons:

#### Production Alignment

- **Future cloud providers** (Hetzner, AWS, DigitalOcean) use virtual machines
- **Consistent behavior** across development and production environments
- **True isolation** that matches cloud VM characteristics

#### When to Consider Containers Instead

Containers could be valuable for:

- **Faster CI/CD testing** (~2-5s boot vs ~20-30s for VMs)
- **GitHub Actions shared runners** (limited resources, faster startup)
- **Resource-constrained development** (less RAM/CPU usage)
- **Integration testing** where speed > production fidelity

However, **virtual machines remain the default** to ensure deployment scripts, Ansible playbooks, and configurations work identically in production cloud environments.

### Switching Instance Types

To switch between virtual machines and containers, modify the `templates/tofu/lxd/main.tf` file:

```hcl
resource "lxd_instance" "torrust_vm" {
  name      = var.container_name
  image     = var.image
  type      = "virtual-machine"  # Change to "container" if needed
  profiles  = [lxd_profile.torrust_profile.name]

  config = {
    # VM-specific settings
    "boot.autostart"      = "true"
    "security.secureboot" = "false"
  }

  wait_for_network = true
}
```

**Note:** Container-specific configurations may require different settings:

- Remove `security.secureboot` (VM-only)
- Add `security.nesting = "true"` and `security.privileged = "false"` for containers
- Adjust resource limits in the profile accordingly

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
   cd build/tofu/lxd  # Use build directory for runtime operations
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
   instance_info = {
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
   cd build/tofu/lxd  # Use build directory for runtime operations
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

**Important Note**: The GitHub workflow uses `sudo chmod 666` on the LXD socket as a workaround for CI environments
where terminal restarts aren't practical. **This approach is not recommended for local development** due to security
implications. For local use, follow the proper group membership approach described in the
[LXD documentation](../../docs/tech-stack/lxd.md#proper-lxd-group-setup).

### Configuration Benefits for CI

- No nested virtualization required
- Guaranteed GitHub Actions compatibility
- Faster startup than full VMs
- Lower resource usage
- Full systemd support for system-level testing

## Additional Information

For more information about LXD system containers, including Docker support, general capabilities, and detailed
troubleshooting, see the [LXD documentation](../../docs/tech-stack/lxd.md).
