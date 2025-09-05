[![Linting](https://github.com/torrust/torrust-tracker-deploy-rust-poc/actions/workflows/linting.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deploy-rust-poc/actions/workflows/linting.yml) [![Testing](https://github.com/torrust/torrust-tracker-deploy-rust-poc/actions/workflows/testing.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deploy-rust-poc/actions/workflows/testing.yml) [![E2E Tests](https://github.com/torrust/torrust-tracker-deploy-rust-poc/actions/workflows/test-e2e.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deploy-rust-poc/actions/workflows/test-e2e.yml) [![Test LXD Container Provisioning](https://github.com/torrust/torrust-tracker-deploy-rust-poc/actions/workflows/test-lxd-provision.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deploy-rust-poc/actions/workflows/test-lxd-provision.yml)

# Torrust Tracker Deploy

This Rust application provides automated deployment infrastructure for Torrust tracker projects. It manages VM provisioning and
cloud-init execution using LXD containers, with the goal of finding the best solution for
creating VMs that support cloud-init both locally (development) and in CI environments (GitHub Actions).

## 🎯 Project Goals

- ✅ **Create VMs supporting cloud-init** locally and in GitHub runners
- ✅ **Test cloud-init execution and verification**
- ✅ **Support Docker Compose** inside VMs (planned)
- ✅ **Fast, easy to install and use** solutions
- ❌ **No nested virtualization dependency** (CI compatibility)

## 🔧 Available Approaches

This repository uses LXD containers for virtualization:

### ☁️ **LXD Containers (`templates/tofu/lxd/`)** - **OFFICIAL**

- **Technology**: System containers with cloud-init support
- **Status**: ✅ Official provider - Guaranteed GitHub Actions compatibility
- **Best for**: CI/CD environments, fast provisioning, local development
- **Requirements**: No special virtualization needed

**[📖 See detailed documentation →](templates/tofu/lxd/README.md)**

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

This is a Rust application that automates deployment infrastructure using OpenTofu and Ansible.

Install the required tools:

```bash
# Check installations
lxd version && tofu version && ansible --version && cargo --version
```

**Missing tools?** See detailed installation guides:

- **[📖 OpenTofu Setup Guide →](docs/tech-stack/opentofu.md)**
- **[📖 Ansible Setup Guide →](docs/tech-stack/ansible.md)**

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
torrust-tracker-deploy
```

#### Development Tasks

This project includes convenient scripts for common development tasks:

```bash
# Run all linters (markdown, YAML, TOML, shell scripts, Rust)
cargo run --bin linter all
```

Or run individual linters:

```bash
cargo run --bin linter markdown    # Markdown linting
cargo run --bin linter yaml        # YAML linting
cargo run --bin linter toml        # TOML linting
cargo run --bin linter clippy      # Rust code analysis
cargo run --bin linter rustfmt     # Rust formatting check
cargo run --bin linter shellcheck  # Shell script linting
```

**[📖 See linting documentation →](docs/linting.md)**

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
cd templates/tofu/lxd

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

**[📖 See detailed Ansible documentation →](docs/tech-stack/ansible.md)**

## 🧪 Testing in GitHub Actions

The repository includes comprehensive GitHub Actions workflows for CI testing:

- **`.github/workflows/test-e2e.yml`** - **End-to-End Tests** - Runs automated E2E tests using the Rust binary
- **`.github/workflows/test-lxd-provision.yml`** - Tests LXD container provisioning

## 📊 Current Status

### ✅ Completed

- [x] LXD container provisioning (local + GitHub Actions)
- [x] Cloud-init support for LXD containers
- [x] OpenTofu infrastructure as code
- [x] Ansible configuration management setup
- [x] Basic cloud-init verification playbook
- [x] Docker installation playbook
- [x] Docker Compose installation playbook
- [x] Automated testing workflows
- [x] End-to-End (E2E) deployment infrastructure and workflows

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
│   ├── tech-stack/          # Generic technology documentation
│   │   ├── opentofu.md      # OpenTofu installation and usage
│   │   ├── ansible.md       # Ansible installation and usage
│   │   └── lxd.md          # LXD system containers
│   ├── decisions/           # Architecture Decision Records (ADRs)
│   │   └── meson-removal.md # Decision to remove Meson build system
│   ├── documentation.md     # Documentation organization guide
│   └── vm-providers.md      # Provider comparison for this project
├── templates/               # 📁 Template configurations (git-tracked)
│   ├── tofu/                # 🏗️ OpenTofu/Terraform templates
│   │   └── lxd/             # LXD container template configuration
│   └── ansible/             # 🤖 Ansible playbook templates
├── build/                   # 📁 Generated runtime configs (git-ignored)
│   ├── tofu/                # 🏗️ Runtime OpenTofu configs
│   └── ansible/             # 🤖 Runtime Ansible configs
├── scripts/                  # Development and utility scripts
│   └── setup/               # Setup scripts for dependencies
├── src/                     # Rust source code
│   ├── bin/                 # Binary executables
│   │   ├── linter.rs        # Unified linting command interface
│   │   └── e2e_tests.rs     # End-to-end testing binary
│   └── linting/             # Linting module and implementations
├── .github/workflows/       # CI/CD workflows
├── Cargo.toml              # Rust project configuration
├── README.md               # This file - project overview
├── target/                 # Rust build artifacts (ignored)
└── .gitignore              # Git ignore rules
```

## 📚 Documentation

- **[🤝 Contributing Guide](docs/contributing/README.md)** - Git workflow, commit process, and linting conventions
- **[📖 Documentation Organization Guide](docs/documentation.md)** - How documentation is organized and where to contribute
- **[📖 OpenTofu Setup Guide](docs/tech-stack/opentofu.md)** - Installation, common commands, and best practices
- **[📖 Ansible Setup Guide](docs/tech-stack/ansible.md)** - Installation, configuration, and project usage
- **[📖 VM Providers Comparison](docs/vm-providers.md)** - Detailed comparison and decision rationale

## 🔮 Next Steps

This is a basic setup. Future enhancements could include:

- Multiple VMs for different testing scenarios
- Custom images with pre-installed Torrust components
- Network configuration for multi-VM setups
- Enhanced CI/CD integration with nested virtualization
- Automated testing scripts for Torrust applications
