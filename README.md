[![Linting](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/linting.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/linting.yml) [![Testing](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/testing.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/testing.yml) [![E2E Provision Tests](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-provision.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-provision.yml) [![E2E Config Tests](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-config.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-config.yml) [![Test LXD Container Provisioning](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-lxd-provision.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-lxd-provision.yml)

# Torrust Tracker Deployer

> ⚠️ **DEVELOPMENT STATUS: Not Production-Ready**
>
> This project is currently in **active development** and is **not production-ready**. It currently only supports **local deployment** using LXD virtualization for development and testing purposes.
>
> **Current Scope:**
>
> - ✅ Local LXD virtual machine provisioning
> - ✅ Development and testing workflows
> - ❌ **No real cloud provider support** (AWS, GCP, Azure, Hetzner, etc.)
> - ❌ **No production deployment capabilities**
>
> 📋 **MVP Goal:** After completing the [roadmap](docs/roadmap.md), we will have a Minimum Viable Product (MVP) that supports real cloud providers and production deployments.

This Rust application provides automated deployment infrastructure for Torrust tracker projects. Currently focused on **local development and testing**, it manages VM provisioning and cloud-init execution using LXD virtual machines. The goal is to establish the foundational deployment patterns that will later support real cloud providers.

## 🎯 Project Goals

**Current Development Phase:**

- ✅ **Create local VMs supporting cloud-init** for development and CI testing
- ✅ **Test cloud-init execution and verification** in controlled environments
- ✅ **Support Docker Compose** inside VMs for application stacks
- ✅ **Fast, easy to install and use** local development solution
- ✅ **No nested virtualization dependency** (CI compatibility)

**Future MVP Goals:** (See [roadmap](docs/roadmap.md))

- 🔄 **Real cloud provider support** (Hetzner, AWS, GCP, Azure)
- 🔄 **Production deployment capabilities**
- 🔄 **Multi-environment management**

## 🔧 Local Development Approach

This repository uses LXD virtual machines for local virtualization and development:

### ☁️ **LXD Virtual Machines (`templates/tofu/lxd/`)** - **LOCAL DEVELOPMENT**

- **Technology**: Virtual machines with cloud-init support
- **Status**: ✅ Production-ready for local development and CI testing
- **Best for**: Local development, CI/CD environments, fast iteration
- **Requirements**: No special virtualization needed

**[📖 See detailed documentation →](docs/tofu-lxd-configuration.md)**

## 📊 LXD Benefits

**[📖 See detailed comparison →](docs/vm-providers.md)**

| Feature                    | LXD Virtual Machines |
| -------------------------- | -------------------- |
| **GitHub Actions Support** | ✅ Guaranteed        |
| **Nested Virtualization**  | ❌ Not needed        |
| **Boot Time**              | ✅ Fast (~5-10s)     |
| **Resource Usage**         | ✅ Efficient         |
| **Installation**           | ✅ Simple setup      |

## 🚀 Quick Start

### 📋 Prerequisites

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

### 💻 Usage

#### 🚀 Main Application

The main application provides usage instructions:

```bash
# Build and run the application
cargo run

# Or install and run directly
cargo install --path .
torrust-tracker-deployer
```

For detailed usage instructions, command reference, and examples, see the **[👤 User Guide](docs/user-guide/README.md)**.

The application includes comprehensive logging with configurable format, output mode, and directory. See **[📖 Logging Guide](docs/user-guide/logging.md)** for details on logging configuration options.

#### 🔧 Development Tasks

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
cargo run --bin linter cspell      # Spell checking
cargo run --bin linter clippy      # Rust code analysis
cargo run --bin linter rustfmt     # Rust formatting check
cargo run --bin linter shellcheck  # Shell script linting
```

**[📖 See linting documentation →](docs/linting.md)**

#### 🧪 Running E2E Tests

Use the E2E test binaries to run automated infrastructure tests with hardcoded environments:

```bash
# Run comprehensive E2E tests (LOCAL ONLY - connectivity issues in GitHub runners)
cargo run --bin e2e-tests-full

# Run individual E2E test suites
cargo run --bin e2e-config-tests      # Configuration generation and validation tests
cargo run --bin e2e-provision-tests   # Infrastructure provisioning tests

# Keep the test environment after completion for inspection
cargo run --bin e2e-tests-full -- --keep
cargo run --bin e2e-provision-tests -- --keep

# Use custom templates directory
cargo run --bin e2e-tests-full -- --templates-dir ./custom/templates

# See all available options
cargo run --bin e2e-tests-full -- --help
```

> **⚠️ Important Notes:**
>
> - E2E tests create **hardcoded environments** with predefined configurations
> - Use `--keep` flag to inspect generated `data/` and `build/` directories after tests
> - `e2e-tests-full` can **only run locally** due to connectivity issues in GitHub runners
> - To see final OpenTofu and Ansible templates, check `build/` directories after running with `--keep`

### 📖 ~~Manual Deployment Steps~~ _(Not Currently Supported)_

> **⚠️ Manual deployment is not currently possible.** The application only supports E2E tests with hardcoded environments.
>
> **Temporary Workaround:** Until the `provision` command is available as a user-facing command, you can use E2E tests to provision infrastructure for experimentation. See the reference section below for details.
>
> **To explore the deployment process:**
>
> 1. Run E2E tests with `--keep` flag: `cargo run --bin e2e-tests-full -- --keep`
> 2. Inspect generated templates in `build/` directories
> 3. Review environment data in `data/` directories
>
> **Manual deployment commands will be available after completing the [roadmap](docs/roadmap.md).**

<details>
<summary>📋 <strong>Reference: Experimenting with OpenTofu and Ansible manually (Temporary Workaround)</strong></summary>

> **Note:** This is a **temporary workaround** for experimentation until the `provision` command is available as a user-facing CLI command. Currently, infrastructure provisioning is only implemented in E2E tests.

If you want to experiment with OpenTofu and Ansible commands directly using the generated templates:

#### 1️⃣ Generate Resolved Templates

```bash
# Run E2E tests but keep the infrastructure for manual experimentation
cargo run --bin e2e-tests-full -- --keep

# This creates resolved templates (no variables) in build/ directories
# ✅ Verified: Creates build/e2e-full/tofu/lxd/ and build/e2e-full/ansible/
```

#### 2️⃣ Navigate to Generated Templates

```bash
# Navigate to the specific environment's resolved OpenTofu templates
cd build/e2e-full/tofu/lxd/

# Or navigate to resolved Ansible templates
cd build/e2e-full/ansible/

# Other available environments:
# cd build/e2e-provision/tofu/lxd/
# cd build/e2e-provision/ansible/
# cd build/e2e-config/ansible/   # (config tests don't create tofu resources)
```

#### 3️⃣ Execute Commands Manually

```bash
# From build/e2e-full/tofu/lxd/ - Execute OpenTofu commands
tofu plan -var-file=variables.tfvars    # ✅ Verified: Works with resolved templates
tofu validate                           # Validate configuration
tofu output -json                       # View current outputs
# Note: tofu apply already executed during E2E test

# From build/e2e-full/ansible/ - Execute Ansible commands
ansible-playbook --list-hosts -i inventory.yml wait-cloud-init.yml  # ✅ Verified: Works
ansible-playbook -i inventory.yml wait-cloud-init.yml              # Run playbook
ansible-playbook -i inventory.yml install-docker.yml               # Install Docker
```

#### 4️⃣ Connect to the Instance

```bash
# Connect to the running LXD instance directly
lxc exec torrust-tracker-vm-e2e-full -- /bin/bash

# Or via SSH (IP may vary, check tofu output)
ssh -i fixtures/testing_rsa torrust@$(cd build/e2e-full/tofu/lxd && tofu output -json | jq -r '.instance_info.value.ip_address')
```

#### 5️⃣ Destroy Infrastructure

```bash
# ✅ Verified: Destroy the infrastructure when done experimenting
cd build/e2e-full/tofu/lxd/
tofu destroy -var-file=variables.tfvars -auto-approve

# ✅ Verified: This removes both the VM instance and the LXD profile
# Alternative: Use LXD commands directly
# lxc delete torrust-tracker-vm-e2e-full --force
# lxc profile delete torrust-profile-e2e-full
```

> **⚠️ Important:** Currently there's no application command to destroy infrastructure manually. You must use either:
>
> 1. **OpenTofu destroy** (recommended) - Uses resolved templates in `build/` directories
> 2. **LXD commands** - Direct LXD resource management
> 3. **Re-run E2E tests** - Automatically destroys and recreates infrastructure
>
> **📖 For comprehensive LXD commands and examples, see [LXD documentation](docs/tech-stack/lxd.md)**

</details>

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

- **`.github/workflows/linting.yml`** - **Code Quality** - Runs all linters (markdown, YAML, TOML, Rust, shell scripts)
- **`.github/workflows/testing.yml`** - **Unit Tests** - Runs Rust unit tests and basic validation
- **`.github/workflows/test-e2e-config.yml`** - **E2E Config Tests** - Tests configuration generation and validation
- **`.github/workflows/test-e2e-provision.yml`** - **E2E Provision Tests** - Tests infrastructure provisioning workflows
- **`.github/workflows/test-lxd-provision.yml`** - **LXD Provisioning** - Tests LXD VM provisioning specifically

> **Note:** The full E2E tests (`e2e-tests-full`) can only be executed locally due to connectivity issues documented in [`docs/github-actions-issues/`](docs/github-actions-issues/).

## � Roadmap

This project follows a structured development roadmap to evolve from the current local development focus to a production-ready deployment solution.

**Current Development Status:**

- ✅ **Local LXD Infrastructure**: VM provisioning, cloud-init, E2E testing
- ✅ **Development Workflows**: Linting, testing, CI/CD automation
- ✅ **Foundation Layer**: OpenTofu + Ansible + Docker integration

**Next Major Milestones:**

- 🔄 **Main Application Commands**: `create`, `deploy`, `destroy`, `status`
- � **Real Cloud Providers**: Starting with Hetzner, expanding to AWS/GCP/Azure
- 🔄 **Production Features**: HTTPS, backups, monitoring stack

**[📖 See complete roadmap →](docs/roadmap.md)**

## 📁 Repository Structure

```text
├── .github/                  # CI/CD workflows and GitHub configuration
│   └── workflows/           # GitHub Actions workflow files
├── build/                   # 📁 Generated runtime configs (git-ignored)
│   ├── e2e-config/          # E2E config test runtime files
│   ├── e2e-full/            # E2E full test runtime files
│   └── e2e-provision/       # E2E provision test runtime files
├── data/                    # Environment-specific data and configurations
│   ├── e2e-config/          # E2E config test environment data
│   ├── e2e-full/            # E2E full test environment data
│   ├── e2e-provision/       # E2E provision test environment data
│   └── logs/                # Application logs
├── docker/                  # Docker-related configurations
│   └── provisioned-instance/ # Docker setup for provisioned instances
├── docs/                    # 📖 Detailed documentation
│   ├── tech-stack/          # Technology-specific documentation
│   │   ├── opentofu.md      # OpenTofu installation and usage
│   │   ├── ansible.md       # Ansible installation and usage
│   │   └── lxd.md          # LXD virtual machines
│   ├── decisions/           # Architecture Decision Records (ADRs)
│   ├── contributing/        # Contributing guidelines and conventions
│   │   ├── README.md        # Main contributing guide
│   │   ├── branching.md     # Git branching conventions
│   │   ├── commit-process.md # Commit process and pre-commit checks
│   │   ├── error-handling.md # Error handling principles
│   │   ├── module-organization.md # Module organization conventions
│   │   └── testing/         # Testing conventions and guides
│   ├── features/            # Feature specifications and documentation
│   ├── research/            # Research and analysis documents
│   └── *.md                 # Various documentation files
├── examples/                # Example configurations and usage
├── fixtures/                # Test fixtures and sample data
│   ├── testing_rsa*         # SSH key pair for testing
│   └── tofu/               # OpenTofu test fixtures
├── packages/                # Rust workspace packages
│   └── linting/            # Linting utilities package
│       └── src/            # Linting implementation source code
├── scripts/                 # Development and utility scripts
│   └── setup/              # Installation scripts for dependencies
├── src/                     # 🦀 Main Rust application source code (DDD Architecture)
│   ├── main.rs             # Main application binary entry point
│   ├── lib.rs              # Library root module
│   ├── container.rs        # Dependency injection container
│   ├── logging.rs          # Logging configuration
│   ├── bin/                # Binary executables
│   │   ├── linter.rs       # Unified linting command interface
│   │   └── e2e*.rs         # End-to-end testing binaries
│   ├── application/        # Application layer (use cases, commands)
│   ├── domain/             # Domain layer (business logic, entities)
│   ├── infrastructure/     # Infrastructure layer (external systems)
│   ├── shared/             # Shared utilities and common code
│   ├── testing/            # Testing utilities and mocks
│   ├── config/             # Configuration handling
│   └── e2e/                # End-to-end testing infrastructure
├── templates/               # 📁 Template configurations (git-tracked)
│   ├── tofu/               # 🏗️ OpenTofu/Terraform templates
│   │   └── lxd/            # LXD VM template configuration
│   └── ansible/            # 🤖 Ansible playbook templates
├── tests/                  # Integration and system tests
├── target/                 # 🦀 Rust build artifacts (git-ignored)
├── Cargo.toml             # Rust workspace configuration
├── Cargo.lock             # Rust dependency lock file
├── cspell.json            # Spell checking configuration
├── project-words.txt      # Custom dictionary for spell checking
├── .markdownlint.json     # Markdown linting configuration
├── .taplo.toml            # TOML formatting configuration
├── .yamllint-ci.yml       # YAML linting configuration for CI
├── README.md              # This file - project overview
├── LICENSE                # Project license
└── .gitignore             # Git ignore rules
```

## 📚 Documentation

- **[👤 User Guide](docs/user-guide/README.md)** - Getting started, command reference, and usage examples
- **[🤝 Contributing Guide](docs/contributing/README.md)** - Git workflow, commit process, and linting conventions
- **[🗺️ Roadmap](docs/roadmap.md)** - Development roadmap and MVP goals
- **[📖 Documentation Organization Guide](docs/documentation.md)** - How documentation is organized and where to contribute
- **[📖 OpenTofu Setup Guide](docs/tech-stack/opentofu.md)** - Installation, common commands, and best practices
- **[📖 Ansible Setup Guide](docs/tech-stack/ansible.md)** - Installation, configuration, and project usage
- **[📖 VM Providers Comparison](docs/vm-providers.md)** - Detailed comparison and decision rationale

## 🔮 Next Steps

This project is currently focused on local development. The path to production-ready deployment is outlined in our [📋 **Roadmap**](docs/roadmap.md).

**Key upcoming milestones:**

- **Real Cloud Provider Support**: Starting with Hetzner, then expanding to AWS, GCP, and Azure
- **Production Commands**: `create`, `deploy`, `destroy`, and `status` commands for production environments
- **Application Stack Management**: Complete Docker Compose stacks with Torrust Tracker, MySQL, Prometheus, and Grafana
- **HTTPS Support**: SSL/TLS configuration for all services
- **Backup & Recovery**: Database backups and disaster recovery procedures

**[📖 See full roadmap →](docs/roadmap.md)**
