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

3. **Before committing**: Always run these verifications - all must pass before staging files or creating commits, regardless of the tool or method used:

   ```bash
   # Run cargo machete
   cargo machete
   # Run linters
   ./scripts/lint.sh
   # Run tests
   cargo test
   # Run e2e tests
   cargo run --bin e2e-tests
   ```

   This applies to **any** method of committing:

   - Terminal: `git add`, `git commit`, `git commit -am`, `cd ../ && git add ...`, `git add . && git commit -m "..."`
   - VS Code: Git panel, Source Control view, commit shortcuts
   - IDEs: IntelliJ, CLion, RustRover git integration
   - Git clients: GitHub Desktop, GitKraken, etc.
   - CI/CD: Any automated commits or merges

## 🧪 Build & Test

- **Lint**: `./scripts/lint.sh` (comprehensive - tests stable & nightly toolchains)
- **Dependencies**: `cargo machete` (mandatory before commits - no unused dependencies)
- **Build**: `cargo build`
- **Test**: `cargo test`
- **Unit Tests**: When writing unit tests, follow conventions described in [`docs/contributing/testing.md`](../docs/contributing/testing.md)
- **E2E Tests**: `cargo e2e-provision && cargo e2e-config`

Follow the project conventions and ensure all checks pass.
