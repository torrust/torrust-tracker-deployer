# Torrust Tracker Deploy - AI Assistant Instructions

## 📋 Project Overview

This is a deployment infrastructure proof-of-concept for the Torrust ecosystem. It uses OpenTofu (Terraform), Ansible, and Rust to provision and manage deployment environments with LXD containers and Multipass VMs.

## 🏗️ Tech Stack

- **Languages**: Rust, Shell scripts, YAML, TOML
- **Infrastructure**: OpenTofu (Terraform), Ansible
- **Providers**: LXD containers, Multipass VMs
- **Tools**: Docker, cloud-init

## 📁 Key Directories

- `src/` - Rust source code and binaries
- `templates/ansible/` - Ansible playbook templates
- `templates/tofu/` - OpenTofu/Terraform configuration templates
- `build/` - Generated runtime configurations (git-ignored)
- `docs/` - Project documentation

## 🔧 Essential Rules

1. **Before creating branches**: Read [`docs/contributing/branching.md`](../docs/contributing/branching.md) for naming conventions (`{issue-number}-{short-description}`)

2. **Before committing**: Read [`docs/contributing/commit-process.md`](../docs/contributing/commit-process.md) for conventional commits

   - **With issue branch**: `{type}: [#{issue}] {description}` (when branch name starts with `{issue-number}-`)
   - **Without issue branch**: `{type}: {description}` (when working on main or branch without issue number prefix)

3. **Before committing**: Always run these verifications - all must pass before using `git add` or `git commit`:

   ```bash
   # Run cargo machete
   cargo machete
   # Run linters
   cargo run --bin linter all
   # Run tests
   cargo test
   # Run e2e tests
   cargo run --bin e2e-tests
   ```

## 🧪 Build & Test

- **Build**: `cargo build`
- **Test**: `cargo test`
- **Unit Tests**: When writing unit tests, follow conventions described in [`docs/contributing/testing.md`](../docs/contributing/testing.md)
- **Lint**: `cargo run --bin linter all` (mandatory before commits)
- **Dependencies**: `cargo machete` (mandatory before commits - no unused dependencies)
- **E2E**: `cargo run --bin e2e-tests`

Follow the project conventions and ensure all checks pass.
