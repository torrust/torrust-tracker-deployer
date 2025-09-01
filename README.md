# Torrust Testing Infrastructure PoC

This repository contains configurations for testing VM provisioning and cloud-init execution using different virtualization approaches. The goal is to find the best solution for creating VMs that support cloud-init both locally (development) and in CI environments (GitHub Actions).

## 🎯 Project Goals

- ✅ **Create VMs supporting cloud-init** locally and in GitHub runners
- ✅ **Test cloud-init execution and verification**
- ✅ **Support Docker Compose** inside VMs (planned)
- ✅ **Fast, easy to install and use** solutions
- ❌ **No nested virtualization dependency** (CI compatibility)

## 🔧 Available Approaches

This repository tests two different virtualization technologies:

### ☁️ **LXD Containers (`config/tofu/lxd/`)** - **OFFICIAL**

- **Technology**: System containers with cloud-init support
- **Status**: ✅ Official provider - Guaranteed GitHub Actions compatibility
- **Best for**: CI/CD environments, fast provisioning, local development
- **Requirements**: No special virtualization needed

**[📖 See detailed documentation →](config/tofu/lxd/README.md)**

### 🖥️ **Multipass (`config/tofu/multipass/`)** - **EXPERIMENTAL**

- **Technology**: Full VMs with nested virtualization
- **Status**: ⚠️ Experimental - Works in GitHub Actions but undocumented support
- **Best for**: Local development requiring full VM isolation
- **Requirements**: Nested virtualization support

**[📖 See detailed documentation →](config/tofu/multipass/README.md)**

## 🔄 **Quick Comparison**

| Feature                    | LXD Containers (Official)   | Multipass (Experimental)       |
| -------------------------- | --------------------------- | ------------------------------ |
| **Status**                 | ✅ Official Provider        | ⚠️ Experimental                |
| **GitHub Actions Support** | ✅ Guaranteed               | 🔶 Discovered but undocumented |
| **Nested Virtualization**  | ❌ Not needed               | ✅ Required                    |
| **Cloud-init Support**     | ✅ Container boot           | ✅ Full VM boot                |
| **Resource Usage**         | ✅ Lower (containers)       | ❌ Higher (full VMs)           |
| **Isolation Level**        | 🔶 Process-level            | ✅ Complete (separate kernel)  |
| **Boot Time**              | ✅ Faster (container start) | ❌ Slower (full boot)          |
| **Docker Support**         | ✅ Full support             | ✅ Full support                |
| **Setup Complexity**       | 🔶 Requires LXD setup       | ✅ Simple (snap install)       |

## 🚀 **Getting Started**

### 🏠 **Local Deployment (Recommended)**

The **LXD provider** is the official and recommended approach for both local development and CI/CD environments. Multipass is experimental as GitHub runners' full virtualization support is undocumented.

#### **1. Prerequisites Verification**

Before deploying, verify that all required tools are installed:

```bash
# Check LXD installation
lxd version

# Check OpenTofu installation
tofu version

# Check Ansible installation
ansible --version
```

**Install missing tools:**

```bash
# Install LXD (via snap - recommended)
sudo snap install lxd
sudo lxd init --auto
sudo usermod -a -G lxd $USER

# Install OpenTofu
curl --proto '=https' --tlsv1.2 -fsSL https://get.opentofu.org/install-opentofu.sh -o install-opentofu.sh
chmod +x install-opentofu.sh
./install-opentofu.sh --install-method deb

# Install Ansible
sudo apt update && sudo apt install ansible

# IMPORTANT: After adding user to lxd group, restart your terminal or run:
newgrp lxd
```

#### **2. Deploy Infrastructure with OpenTofu**

Navigate to the LXD configuration and deploy the VM:

```bash
# Navigate to LXD configuration
cd config/tofu/lxd

# Initialize OpenTofu
tofu init

# Review planned changes (optional)
tofu plan

# Deploy the infrastructure
tofu apply
# Type 'yes' when prompted

# View deployment results
tofu output
```

After successful deployment, you should see output similar to:

```text
container_info = {
  "image" = "ubuntu:24.04"
  "ip_address" = "10.140.190.177"
  "name" = "torrust-vm"
  "status" = "Running"
}
```

#### **3. Configure with Ansible**

Execute Ansible playbooks to configure and verify the deployed VM:

```bash
# Navigate to Ansible configuration
cd ../../ansible

# Update inventory with the VM's IP address
# Edit inventory.yml and update ansible_host with the IP from step 2

# Test connectivity
ansible all -m ping

# Execute the cloud-init verification playbook
ansible-playbook wait-cloud-init.yml
```

#### **4. Verification**

Verify the deployment is working correctly:

```bash
# Check VM status
lxc list torrust-vm

# Connect to the VM
lxc exec torrust-vm -- /bin/bash

# Test SSH connection (from Ansible directory)
ssh -i ~/.ssh/testing_rsa torrust@<VM_IP>
```

### 🧪 **Alternative Approaches**

Choose your preferred approach for specific use cases:

1. **For local development**: Start with [LXD configuration](config/tofu/lxd/README.md) (recommended)
2. **For experimental testing**: Try [Multipass configuration](config/tofu/multipass/README.md) (nested virtualization required)
3. **For testing both**: Compare both approaches to evaluate differences

## 🎭 **Ansible Configuration Management**

Once VMs are provisioned by OpenTofu, we use **Ansible** to execute tasks and manage configuration on the running instances.

### ⚙️ **Ansible Setup (`config/ansible/`)**

- **Technology**: Agentless configuration management and task automation
- **Purpose**: Execute tasks on OpenTofu-provisioned VMs
- **Features**: Cloud-init verification, system configuration, application deployment

**[📖 See detailed Ansible documentation →](config/ansible/README.md)**

### 🔄 **Infrastructure Workflow**

1. **Provision**: OpenTofu creates and configures VMs with cloud-init
2. **Configure**: Ansible connects to VMs and executes management tasks
3. **Verify**: Automated checks ensure proper setup and functionality

| Phase              | Tool               | Purpose                                     |
| ------------------ | ------------------ | ------------------------------------------- |
| **Infrastructure** | OpenTofu/Terraform | VM provisioning and cloud-init setup        |
| **Configuration**  | Ansible            | Task execution and configuration management |
| **Verification**   | Ansible Playbooks  | System checks and validation                |

## 🧪 **Testing in GitHub Actions**

Both configurations include GitHub Actions workflows for CI testing:

- **`.github/workflows/test-multipass-provision.yml`** - Tests Multipass VMs
- **`.github/workflows/test-lxd-provision.yml`** - Tests LXD containers

## 📊 **Current Status**

### ✅ **Completed**

- [x] Multipass VM provisioning (local + GitHub Actions)
- [x] LXD container provisioning (local + GitHub Actions)
- [x] Cloud-init support in both approaches
- [x] OpenTofu infrastructure as code
- [x] Ansible configuration management setup
- [x] Basic cloud-init verification playbook
- [x] Automated testing workflows

### 🔄 **In Progress**

- [ ] Extended Ansible playbooks for application deployment
- [ ] Docker Compose integration testing
- [ ] Performance benchmarking
- [ ] Official GitHub Actions nested virtualization clarification

### 📋 **Planned**

- [ ] Additional VM providers evaluation
- [ ] Integration with Torrust application testing
- [ ] Multi-architecture support (ARM64)

## 📁 **Repository Structure**

```text
├── config/
│   ├── tofu/
│   │   ├── multipass/
│   │   │   ├── main.tf           # OpenTofu configuration for Multipass VMs
│   │   │   ├── cloud-init.yml    # Cloud-init configuration
│   │   │   └── README.md         # Multipass-specific documentation
│   │   └── lxd/
│   │       ├── main.tf           # OpenTofu configuration for LXD containers
│   │       ├── cloud-init.yml    # Cloud-init configuration (same as multipass)
│   │       └── README.md         # LXD-specific documentation
│   └── ansible/
│       ├── ansible.cfg           # Ansible configuration
│       ├── inventory.yml         # Host inventory for provisioned VMs
│       ├── wait-cloud-init.yml   # Playbook to wait for cloud-init completion
│       └── README.md             # Ansible-specific documentation
├── .github/
│   └── workflows/
│       ├── test-multipass-provision.yml  # Tests Multipass VMs
│       └── test-lxd-provision.yml        # Tests LXD containers
├── README.md                 # This file - project overview
└── .gitignore                # Git ignore rules
```

The repository now properly documents this significant discovery and provides a clear path for others to follow the official GitHub Actions team response. The commit message follows conventional commit standards and clearly describes the documentation improvements.

## Next Steps

This is a basic setup. Future enhancements could include:

- Multiple VMs for different testing scenarios
- Custom images with pre-installed Torrust components
- Network configuration for multi-VM setups
- Enhanced CI/CD integration with nested virtualization
- Automated testing scripts
