# Torrust Testing Infrastructure

This Rust application provides automated testing infrastructure for Torrust projects. It manages VM provisioning and cloud-init execution using different virtualization approaches, with the goal of finding the best solution for creating VMs that support cloud-init both locally (development) and in CI environments (GitHub Actions).

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

## � Provider Comparison

**[📖 See detailed comparison →](docs/vm-providers.md)**

| Feature                    | LXD (Official)   | Multipass (Experimental) |
| -------------------------- | ---------------- | ------------------------ |
| **GitHub Actions Support** | ✅ Guaranteed    | ⚠️ Undocumented          |
| **Nested Virtualization**  | ❌ Not needed    | ✅ Required              |
| **Boot Time**              | ✅ ~5-10s        | ❌ ~30-60s               |
| **Resource Usage**         | ✅ Lower         | ❌ Higher                |
| **Isolation Level**        | 🔶 Process-level | ✅ Hardware-level        |

## 🚀 Quick Start

### Prerequisites

This is a Rust application that automates testing infrastructure deployment using OpenTofu and Ansible.

Install the required tools:

```bash
# Check installations
lxd version && tofu version && ansible --version && cargo --version
```

**Missing tools?** See detailed installation guides:

- **[📖 OpenTofu Setup Guide →](docs/opentofu.md)**
- **[📖 Ansible Setup Guide →](docs/ansible.md)**

**Quick install:**

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install LXD
sudo snap install lxd && sudo lxd init --auto && sudo usermod -a -G lxd $USER && newgrp lxd

# Install OpenTofu
curl -fsSL https://get.opentofu.org/install-opentofu.sh | sudo bash

# Install Ansible
sudo apt install ansible
```

### Usage

#### Main Application

The main application provides usage instructions:

```bash
# Build and run the application
cargo run

# Or install and run directly
cargo install --path .
torrust-testing-infra
```

#### Running E2E Tests

Use the E2E tests binary to run automated infrastructure tests:

```bash
# Run the wait-cloud-init test
cargo run --bin e2e-tests -- wait-cloud-init

# Keep the test environment after completion
cargo run --bin e2e-tests -- wait-cloud-init --keep

# Run with verbose output
cargo run --bin e2e-tests -- wait-cloud-init --verbose

# See all available options
cargo run --bin e2e-tests -- --help
```

### Manual Deployment Steps

If you prefer manual deployment instead of using the E2E tests:

#### 1. Deploy Infrastructure

```bash
# Navigate to LXD configuration
cd config/tofu/lxd

# Initialize and deploy
tofu init && tofu apply
```

#### 2. Configure with Ansible

```bash
# Navigate to Ansible configuration
cd ../../ansible

# Update inventory.yml with the VM's IP from step 1
# Then run the verification playbook
ansible-playbook wait-cloud-init.yml

# Install Docker on the VM
ansible-playbook install-docker.yml

# Install Docker Compose on the VM (optional)
ansible-playbook install-docker-compose.yml
```

#### 3. Verify Deployment

```bash
# Check VM status
lxc list torrust-vm

# Connect to VM
lxc exec torrust-vm -- /bin/bash

# Test SSH connection
ssh -i ~/.ssh/testing_rsa torrust@<VM_IP>

# Verify Docker installation
lxc exec torrust-vm -- docker --version
lxc exec torrust-vm -- docker run --rm hello-world

# Verify Docker Compose installation (if installed)
lxc exec torrust-vm -- docker-compose --version
```

## 🎭 Infrastructure Workflow

1. **Provision**: OpenTofu creates and configures VMs with cloud-init
2. **Configure**: Ansible connects to VMs and executes management tasks
3. **Verify**: Automated checks ensure proper setup and functionality

| Phase              | Tool              | Purpose                                     |
| ------------------ | ----------------- | ------------------------------------------- |
| **Infrastructure** | OpenTofu          | VM provisioning and cloud-init setup        |
| **Configuration**  | Ansible           | Task execution and configuration management |
| **Verification**   | Ansible Playbooks | System checks and validation                |

**[📖 See detailed Ansible documentation →](docs/ansible.md)**

## 🧪 Testing in GitHub Actions

Both configurations include GitHub Actions workflows for CI testing:

- **`.github/workflows/test-multipass-provision.yml`** - Tests Multipass VMs
- **`.github/workflows/test-lxd-provision.yml`** - Tests LXD containers

## 📊 Current Status

### ✅ Completed

- [x] Multipass VM provisioning (local + GitHub Actions)
- [x] LXD container provisioning (local + GitHub Actions)
- [x] Cloud-init support in both approaches
- [x] OpenTofu infrastructure as code
- [x] Ansible configuration management setup
- [x] Basic cloud-init verification playbook
- [x] Docker installation playbook
- [x] Docker Compose installation playbook
- [x] Automated testing workflows

### 🔄 In Progress

- [ ] Extended Ansible playbooks for application deployment
- [ ] Performance benchmarking
- [ ] Official GitHub Actions nested virtualization clarification

### 📋 Planned

- [ ] Additional VM providers evaluation
- [ ] Integration with Torrust application testing
- [ ] Multi-architecture support (ARM64)

## 📁 Repository Structure

```text
├── src/                      # Rust application source code
│   ├── main.rs              # Main application binary
│   └── bin/
│       └── e2e_tests.rs     # E2E tests binary
├── docs/                     # Detailed documentation
│   ├── opentofu.md          # OpenTofu setup and usage guide
│   ├── ansible.md           # Ansible setup and usage guide
│   └── vm-providers.md      # Detailed provider comparison
├── config/
│   ├── tofu/
│   │   ├── multipass/       # Multipass VM configuration
│   │   └── lxd/             # LXD container configuration
│   └── ansible/             # Ansible configuration management
├── .github/workflows/       # CI/CD workflows
├── Cargo.toml              # Rust project configuration
├── README.md               # This file - project overview
├── target/                 # Rust build artifacts (ignored)
└── .gitignore              # Git ignore rules
```

## 📚 Documentation

- **[📖 OpenTofu Setup Guide](docs/opentofu.md)** - Installation, common commands, and best practices
- **[📖 Ansible Setup Guide](docs/ansible.md)** - Installation, configuration, and project usage
- **[📖 VM Providers Comparison](docs/vm-providers.md)** - Detailed comparison and decision rationale

## 🔮 Next Steps

This is a basic setup. Future enhancements could include:

- Multiple VMs for different testing scenarios
- Custom images with pre-installed Torrust components
- Network configuration for multi-VM setups
- Enhanced CI/CD integration with nested virtualization
- Automated testing scripts for Torrust applications
