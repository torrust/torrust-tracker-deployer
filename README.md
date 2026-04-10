[![Linting](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/linting.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/linting.yml) [![Testing](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/testing.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/testing.yml) [![Test Dependency Installer](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-dependency-installer.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-dependency-installer.yml) [![E2E Infrastructure Tests](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-infrastructure.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-infrastructure.yml) [![E2E Deployment Tests](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-deployment.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-e2e-deployment.yml) [![SDK Examples](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-sdk-examples.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-sdk-examples.yml) [![Test LXD Container Provisioning](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-lxd-provision.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/test-lxd-provision.yml) [![Coverage](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/coverage.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/coverage.yml) [![Container](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/container.yaml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/container.yaml) [![Backup Container](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/backup-container.yaml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/backup-container.yaml) [![Docker Security Scan](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/docker-security-scan.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/docker-security-scan.yml) [![Cargo Security Audit](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/cargo-security-audit.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/cargo-security-audit.yml) [![Code Statistics](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/code-statistics.yml/badge.svg)](https://github.com/torrust/torrust-tracker-deployer/actions/workflows/code-statistics.yml)

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/torrust/torrust-tracker-deployer?quickstart=1)

# Torrust Tracker Deployer

> ⚠️ **DEVELOPMENT STATUS: Early Production Phase**
>
> This project is in **active development** with initial cloud provider support now available.
>
> **Current Scope:**
>
> - ✅ Local LXD virtual machine provisioning
> - ✅ **Hetzner Cloud support** for production deployments
> - ✅ Development and testing workflows
> - ✅ Multi-provider architecture (provider selection via configuration)
> - ✅ **Application deployment** (Torrust Tracker stack with Docker Compose)
>
> 📋 **MVP Goal:** After completing the [roadmap](docs/roadmap.md), we will have a fully automated deployment solution for Torrust Tracker with complete application stack management and multi-cloud provider support.

This Rust application provides automated deployment infrastructure for Torrust tracker projects. It supports **local development** with LXD and **production deployments** with Hetzner Cloud. The multi-provider architecture allows easy extension to additional cloud providers.

## 🎯 Project Goals

**Current Development Phase:**

- ✅ **Create local VMs supporting cloud-init** for development and CI testing
- ✅ **Test cloud-init execution and verification** in controlled environments
- ✅ **Support Docker Compose** inside VMs for application stacks
- ✅ **Fast, easy to install and use** local development solution
- ✅ **No nested virtualization dependency** (CI compatibility)
- ✅ **Multi-provider support** (LXD for local, Hetzner Cloud for production)
- ✅ **Application stack deployment** (Torrust Tracker with Docker Compose)

**Future MVP Goals:** (See [roadmap](docs/roadmap.md))

- 🔄 **Additional cloud providers** (AWS, GCP, Azure)
- 🔄 **Multi-environment management**
- 🔄 **Enhanced observability** (monitoring, alerting, metrics)

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

#### Automated Setup (Recommended)

The project provides a dependency installer tool that automatically detects and installs required dependencies:

```bash
# Install all required dependencies
cargo run --bin dependency-installer install

# Check which dependencies are installed
cargo run --bin dependency-installer check

# List all dependencies with status
cargo run --bin dependency-installer list
```

The installer supports: **OpenTofu**, **Ansible**, **LXD**, and **cargo-machete**.

For detailed information, see **[📖 Dependency Installer →](packages/dependency-installer/README.md)**.

#### Manual Setup

If you prefer manual installation or need to troubleshoot:

**Check installations:**

```bash
lxd version && tofu version && ansible --version && cargo --version
```

**Missing tools?** See detailed installation guides:

- **[📖 OpenTofu Setup Guide →](docs/tech-stack/opentofu.md)**
- **[📖 Ansible Setup Guide →](docs/tech-stack/ansible.md)**

**Quick manual install:**

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

#### 🐳 Docker (Recommended for Cloud Deployments)

The easiest way to use the deployer for **cloud provider deployments** (Hetzner) is with Docker - no local dependency installation required:

```bash
# Pull the image
docker pull torrust/tracker-deployer:latest

# Run a command (example: show help)
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  --help
```

> ⚠️ **Important**: Docker only supports **cloud providers** (Hetzner). For **LXD local development**, install the deployer directly on your host.

**[📖 See Docker documentation →](docker/deployer/README.md)**

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
cargo run --bin e2e-complete-workflow-tests

# Run individual E2E test suites
cargo run --bin e2e-deployment-workflow-tests         # Configuration, release, and run workflow tests
cargo run --bin e2e-infrastructure-lifecycle-tests   # Infrastructure provisioning tests

# Keep the test environment after completion for inspection
cargo run --bin e2e-complete-workflow-tests -- --keep
cargo run --bin e2e-infrastructure-lifecycle-tests -- --keep

# Use custom templates directory
cargo run --bin e2e-complete-workflow-tests -- --templates-dir ./custom/templates

# See all available options
cargo run --bin e2e-complete-workflow-tests -- --help
```

> **⚠️ Important Notes:**
>
> - E2E tests create **hardcoded environments** with predefined configurations
> - Use `--keep` flag to inspect generated `data/` and `build/` directories after tests
> - `e2e-complete-workflow-tests` can **only run locally** due to connectivity issues in GitHub runners
> - To see final OpenTofu and Ansible templates, check `build/` directories after running with `--keep`

### 📖 Manual Deployment Steps

> **✅ Complete deployment workflow is now available!** You can create, provision, configure, test, deploy, run, and destroy Torrust Tracker environments using the CLI.
>
> **Current Status:**
>
> - ✅ **Environment Management**: Create and manage deployment environments
> - ✅ **Infrastructure Provisioning**: Provision VM infrastructure with LXD or Hetzner Cloud
> - ✅ **Configuration**: Configure provisioned infrastructure (Docker, Docker Compose)
> - ✅ **Verification**: Test deployment infrastructure
> - ✅ **Application Deployment**: Deploy Torrust Tracker configuration and database
> - ✅ **Service Management**: Start and manage tracker services
>
> **Available Commands:**
>
> ```bash
> # 1. Generate configuration template
> torrust-tracker-deployer create template my-env.json
>
> # 2. Edit my-env.json with your settings
>
> # 3. Create environment from configuration
> torrust-tracker-deployer create environment -f my-env.json
>
> # 4. Provision VM infrastructure
> torrust-tracker-deployer provision my-environment
>
> # 5. Configure infrastructure (install Docker, Docker Compose)
> torrust-tracker-deployer configure my-environment
>
> # 6. Verify deployment infrastructure
> torrust-tracker-deployer test my-environment
>
> # 7. Deploy tracker application configuration
> torrust-tracker-deployer release my-environment
>
> # 8. Start tracker services
> torrust-tracker-deployer run my-environment
>
> # 9. Destroy environment when done
> torrust-tracker-deployer destroy my-environment
> ```
>
> **📖 For detailed command documentation and guides, see:**
>
> - **[Quick Start Guides](docs/user-guide/quick-start/README.md)** - Docker and native installation guides
> - **[Commands Reference](docs/user-guide/commands/)** - Detailed guide for each command _(coming soon)_
> - **[Console Commands](docs/console-commands.md)** - Technical reference
> - **[Advanced: Manual Commands](docs/user-guide/advanced-manual-commands.md)** - Manual OpenTofu and Ansible commands (advanced users only)

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
- **`.github/workflows/test-e2e-infrastructure.yml`** - **E2E Infrastructure Tests** - Tests infrastructure provisioning and destruction
- **`.github/workflows/test-e2e-deployment.yml`** - **E2E Deployment Tests** - Tests software installation, configuration, release, and run workflows
- **`.github/workflows/test-lxd-provision.yml`** - **LXD Provisioning** - Tests LXD VM provisioning specifically

> **Note:** The complete E2E workflow tests (`e2e-complete-workflow-tests`) can only be executed locally due to connectivity issues documented in [`docs/e2e-testing/`](docs/e2e-testing/).

## 🗺️ Roadmap

This project follows a structured development roadmap to evolve from the current local development focus to a production-ready deployment solution.

**Current Development Status:**

- ✅ **Local LXD Infrastructure**: VM provisioning, cloud-init, E2E testing
- ✅ **Development Workflows**: Linting, testing, CI/CD automation
- ✅ **Foundation Layer**: OpenTofu + Ansible + Docker integration

**Next Major Milestones:**

- 🔄 **Main Application Commands**: `create`, `deploy`, `destroy`, `status`
- ☁️ **Real Cloud Providers**: Starting with Hetzner, expanding to AWS/GCP/Azure
- 🔄 **Production Features**: HTTPS, backups, monitoring stack

**[📖 See complete roadmap →](docs/roadmap.md)**

## 📁 Repository Structure

```text
├── .github/                       # CI/CD workflows and GitHub configuration
│   └── workflows/                 # GitHub Actions workflow files
├── build/                         # 📁 Generated runtime configs (git-ignored)
│   ├── e2e-complete/              # E2E complete workflow test runtime files
│   ├── e2e-deployment/            # E2E deployment test runtime files
│   ├── e2e-infrastructure/        # E2E infrastructure test runtime files
│   └── manual-test-*/             # Manual test environment runtime files
├── data/                          # Environment-specific data and configurations
│   ├── e2e-complete/              # E2E complete workflow test environment data
│   ├── e2e-deployment/            # E2E deployment test environment data
│   ├── e2e-infrastructure/        # E2E infrastructure test environment data
│   ├── manual-test-*/             # Manual test environment data
│   └── logs/                      # Application logs
├── docker/                        # Docker-related configurations
│   └── provisioned-instance/      # Docker setup for provisioned instances
├── docs/                          # 📖 Detailed documentation
│   ├── tech-stack/                # Technology-specific documentation
│   │   ├── opentofu.md            # OpenTofu installation and usage
│   │   ├── ansible.md             # Ansible installation and usage
│   │   └── lxd.md                 # LXD virtual machines
│   ├── user-guide/                # User documentation
│   │   ├── commands/              # Command reference documentation
│   │   └── providers/             # Provider-specific guides (LXD, Hetzner)
│   ├── decisions/                 # Architecture Decision Records (ADRs)
│   ├── contributing/              # Contributing guidelines and conventions
│   │   ├── README.md              # Main contributing guide
│   │   ├── branching.md           # Git branching conventions
│   │   ├── commit-process.md      # Commit process and pre-commit checks
│   │   ├── error-handling.md      # Error handling principles
│   │   ├── module-organization.md # Module organization conventions
│   │   └── testing/               # Testing conventions and guides
│   ├── features/                  # Feature specifications and documentation
│   ├── research/                  # Research and analysis documents
│   └── *.md                       # Various documentation files
├── envs/                          # 📁 User environment configurations (git-ignored)
│   └── *.json                     # Environment configuration files for CLI
├── fixtures/                      # Test fixtures and sample data
│   ├── testing_rsa*               # SSH key pair for testing
│   └── tofu/                      # OpenTofu test fixtures
├── packages/                      # Rust workspace packages
│   ├── dependency-installer/      # Dependency detection and installation
│   └── linting/                   # Linting utilities package
│       └── src/                   # Linting implementation source code
├── scripts/                       # Development and utility scripts
│   └── setup/                     # Installation scripts for dependencies
├── src/                           # 🦀 Main Rust application source code (DDD Architecture)
│   ├── main.rs                    # Main application binary entry point
│   ├── lib.rs                     # Library root module
│   ├── container.rs               # Dependency injection container
│   ├── logging.rs                 # Logging configuration
│   ├── bin/                       # Binary executables
│   │   ├── linter.rs              # Unified linting command interface
│   │   └── e2e*.rs                # End-to-end testing binaries
│   ├── application/               # Application layer (use cases, commands)
│   ├── domain/                    # Domain layer (business logic, entities)
│   │   └── provider/              # Provider types (LXD, Hetzner)
│   ├── infrastructure/            # Infrastructure layer (external systems)
│   ├── presentation/              # Presentation layer (CLI interface)
│   ├── adapters/                  # External tool adapters (OpenTofu, Ansible, SSH, LXD)
│   ├── shared/                    # Shared utilities and common code
│   ├── testing/                   # Testing utilities and mocks
│   ├── config/                    # Configuration handling
│   ├── bootstrap/                 # Application bootstrapping
│   └── e2e/                       # End-to-end testing infrastructure
├── templates/                     # 📁 Template configurations (git-tracked)
│   ├── tofu/                      # 🏗️ OpenTofu/Terraform templates
│   │   ├── lxd/                   # LXD VM template configuration
│   │   └── hetzner/               # Hetzner Cloud template configuration
│   └── ansible/                   # 🤖 Ansible playbook templates
├── tests/                         # Integration and system tests
├── target/                        # 🦀 Rust build artifacts (git-ignored)
├── Cargo.toml                     # Rust workspace configuration
├── Cargo.lock                     # Rust dependency lock file
├── cspell.json                    # Spell checking configuration
├── project-words.txt              # Custom dictionary for spell checking
├── .markdownlint.json             # Markdown linting configuration
├── .prettierignore                # Prettier ignore rules (for Tera templates)
├── .taplo.toml                    # TOML formatting configuration
├── .yamllint-ci.yml               # YAML linting configuration for CI
├── README.md                      # This file - project overview
├── LICENSE                        # Project license
└── .gitignore                     # Git ignore rules
```

## 📚 Documentation

- **[👤 User Guide](docs/user-guide/README.md)** - Getting started, command reference, and usage examples
- **[☁️ Provider Guides](docs/user-guide/providers/README.md)** - LXD and Hetzner Cloud provider configuration
- **[🤝 Contributing Guide](docs/contributing/README.md)** - Git workflow, commit process, and linting conventions
- **[🗺️ Roadmap](docs/roadmap.md)** - Development roadmap and MVP goals
- **[📖 Documentation Organization Guide](docs/documentation.md)** - How documentation is organized and where to contribute
- **[📖 OpenTofu Setup Guide](docs/tech-stack/opentofu.md)** - Installation, common commands, and best practices
- **[📖 Ansible Setup Guide](docs/tech-stack/ansible.md)** - Installation, configuration, and project usage
- **[📖 VM Providers Comparison](docs/vm-providers.md)** - Detailed comparison and decision rationale

## 🔮 Next Steps

This project now supports multiple infrastructure providers. The path to production-ready deployment is outlined in our [📋 **Roadmap**](docs/roadmap.md).

**Recent achievements:**

- ✅ **Multi-Provider Support**: LXD for local development, Hetzner Cloud for production deployments
- ✅ **Provider Selection**: Choose your provider via `provider_config` in environment configuration
- ✅ **Complete CLI Commands**: `create`, `provision`, `configure`, `test`, and `destroy` commands

**Key upcoming milestones:**

- **Application Stack Management**: Complete Docker Compose stacks with Torrust Tracker, MySQL, Prometheus, and Grafana
- **HTTPS Support**: SSL/TLS configuration for all services
- **Backup & Recovery**: Database backups and disaster recovery procedures
- **Additional Cloud Providers**: AWS, GCP, and Azure support

**[📖 See full roadmap →](docs/roadmap.md)**
