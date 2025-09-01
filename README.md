# Torrust Testing Infrastructure PoC

This repository contains OpenTofu configuration for provisioning virtual machines using Multipass for Torrust testing infrastructure.

## Prerequisites

Before you can provision VMs, ensure you have the following installed:

### Check if Already Installed

First, check if the required tools are already installed on your system:

```bash
# Check if Multipass is installed
multipass version

# Check if OpenTofu is installed
tofu version
```

If both commands return version information, you can skip the installation steps below and go directly to [Provisioning](#provisioning).

### Installation

1. **Multipass**: Install from [https://multipass.run/](https://multipass.run/)

   ```bash
   # On Ubuntu/Debian
   sudo snap install multipass

   # On macOS
   brew install multipass

   # On Windows
   # Download and install from the official website
   ```

2. **OpenTofu**: Install from [https://opentofu.org/](https://opentofu.org/)

   ```bash
   # Using package manager (example for Ubuntu/Debian)
   curl --proto '=https' --tlsv1.2 -fsSL https://get.opentofu.org/install-opentofu.sh -o install-opentofu.sh
   chmod +x install-opentofu.sh
   ./install-opentofu.sh --install-method deb
   ```

## Configuration

The main configuration consists of:

- **`templates/tofu/main.tf`** - OpenTofu configuration defining the Multipass VM
- **`templates/tofu/cloud-init.yml`** - Cloud-init configuration for VM initialization

The setup includes:

- A Multipass virtual machine with Ubuntu 24.04 LTS (Noble Numbat)
- 2 CPUs, 2GB RAM, and 10GB disk
- Basic cloud-init setup with essential packages
- SSH configuration for remote access

### Customization

Before provisioning, you may want to customize:

1. **SSH Key**: Edit the `templates/tofu/cloud-init.yml` file and replace the SSH key with your actual public key
2. **VM Specifications**: Adjust CPU, memory, and disk size in `templates/tofu/main.tf`
3. **VM Name**: Change the instance name from "torrust-vm" to your preferred name
4. **Packages**: Modify the packages list in `templates/tofu/cloud-init.yml` to include additional software

## Provisioning

To provision the virtual machine:

1. **Navigate to the OpenTofu template directory**:

   ```bash
   cd templates/tofu
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

5. **Get VM information**:

   ```bash
   tofu output
   ```

6. **Access the VM**:

   ```bash
   # Direct shell access
   multipass shell torrust-vm

   # SSH access (if you configured your SSH key)
   ssh torrust@<vm-ip-address>
   ```

## Managing the VM

### Access the VM

```bash
# Using multipass directly
multipass shell torrust-vm

# Or via SSH (if you configured your SSH key)
ssh torrust@<vm-ip-address>
```

### Check VM status

```bash
multipass list
```

### Stop the VM

```bash
multipass stop torrust-vm
```

### Start the VM

```bash
multipass start torrust-vm
```

### Execute commands remotely

```bash
# Check if cloud-init provisioning completed
multipass exec torrust-vm -- cat /tmp/provision_complete

# Check system information
multipass exec torrust-vm -- lsb_release -a

# Install packages manually (if needed during development)
multipass exec torrust-vm -- sudo apt update
multipass exec torrust-vm -- sudo apt install -y git curl wget htop vim

# Check available disk space
multipass exec torrust-vm -- df -h

# Check running processes
multipass exec torrust-vm -- ps aux
```

## Cleanup

To destroy the virtual machine and clean up resources:

1. **Navigate to the OpenTofu template directory** (if not already there):

   ```bash
   cd templates/tofu
   ```

2. **Destroy the infrastructure**:

   ```bash
   tofu destroy
   ```

   Type `yes` when prompted to confirm the destruction.

## Troubleshooting

### Common Issues

1. **Multipass not found**: Ensure Multipass is installed and in your PATH
2. **Permission errors**: Make sure your user has permissions to use Multipass
3. **Network issues**: Check if your firewall allows Multipass networking

### Useful Commands

```bash
# Check Multipass version
multipass version

# List all instances
multipass list

# Get instance info
multipass info torrust-vm

# View instance logs
multipass logs torrust-vm
```

## File Structure

```text
├── templates/
│   └── tofu/
│       ├── main.tf           # OpenTofu configuration
│       └── cloud-init.yml    # Cloud-init configuration
├── README.md                 # Documentation
└── .gitignore                # Git ignore rules
```

## Next Steps

This is a basic setup. Future enhancements could include:

- Multiple VMs for different testing scenarios
- Custom images with pre-installed Torrust components
- Network configuration for multi-VM setups
- Integration with CI/CD pipelines
- Automated testing scripts
