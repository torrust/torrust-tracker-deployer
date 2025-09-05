# Torrust Tracker Deploy - AI Assistant Instructions

## 📋 Project Overview

This is a deployment infrastructure proof-of-concept for the Torrust ecosystem. It uses OpenTofu (Terraform), Ansible, and Rust to provision and manage deployment environments with LXD containers and Multipass VMs.

## 🏗️ Tech Stack

- **Languages**: Rust, Shell scripts, YAML
- **Infrastructure**: OpenTofu (Terraform), Ansible
- **Providers**: LXD containers, Multipass VMs
- **Tools**: Docker, cloud-init

## 📁 Key Directories

- `src/` - Rust source code and binaries
- `config/ansible/` - Ansible playbooks and inventory
- `config/tofu/` - OpenTofu/Terraform configurations
- `scripts/linting/` - Code quality scripts
- `docs/` - Project documentation

## 🔧 Essential Rules

1. **Before creating branches**: Read [`docs/contributing/branching.md`](../docs/contributing/branching.md) for naming conventions (`{issue-number}-{short-description}`)

2. **Before committing**: Read [`docs/contributing/commit-process.md`](../docs/contributing/commit-process.md) for conventional commits

   - **With issue branch**: `{type}: [#{issue}] {description}` (when branch name starts with `{issue-number}-`)
   - **Without issue branch**: `{type}: {description}` (when working on main or branch without issue number prefix)

3. **Before committing**: Always run `cargo run --bin linter all` - all linters must pass

## 🧪 Build & Test

- **Build**: `cargo build`
- **Test**: `cargo test`
- **Lint**: `cargo run --bin linter all` (mandatory before commits)
- **E2E**: `cargo run --bin e2e-tests wait-cloud-init`

Follow the project conventions and ensure all checks pass.
