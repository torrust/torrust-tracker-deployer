# ToThis is a deploymThese principles should guide all development decisions, code reviews, and feature implementations.

## 🏗️ Tech Stack infrastructure proof-of-concept for the Torrust ecosystem. It uses OpenTofu (Terraform), Ansible, and Rust to provision and manage deployment environments with LXD containers and Multipass VMs.

## 🎯 Essential Principles

The development of this application is guided by fundamental principles that ensure quality, maintainability, and user experience. For detailed information, see [`docs/development-principles.md`](../docs/development-principles.md). Tracker Deploy - AI Assistant Instructions

## 📋 Project Overview

This is a deployment infrastructure proof-of-concept for the Torrust ecosystem. It uses OpenTofu (Terraform), Ansible, and Rust to provision and manage deployment environments with LXD containers and Multipass VMs.

## � Essential Principles

The development of this application is guided by fundamental principles that ensure quality, maintainability, and user experience. For detailed information, see [`docs/development-principles.md`](../docs/development-principles.md).

**Core Principles:**

- **Observability**: If it happens, we can see it - even after it happens (includes deep traceability)
- **Testability**: Every component must be testable in isolation and as part of the whole
- **User Friendliness**: All errors must be clear, informative, and solution-oriented
- **Actionability**: The system must always tell users how to continue with detailed instructions

These principles should guide all development decisions, code reviews, and feature implementations.

## �🏗️ Tech Stack

- **Languages**: Rust, Shell scripts, YAML, TOML
- **Infrastructure**: OpenTofu (Terraform), Ansible
- **Virtualization Providers**: LXD VM instances
- **Tools**: Docker, cloud-init, testcontainers
- **Linting Tools**: markdownlint, yamllint, shellcheck, clippy, rustfmt, taplo (TOML)

## 📁 Key Directories

- `src/` - Rust source code and binaries
- `data/templates/` - Source template files for Ansible and OpenTofu
- `templates/` - Generated template examples and test fixtures
- `build/` - Generated runtime configurations (git-ignored)
- `docs/` - Project documentation
- `scripts/` - Shell scripts for development tasks
- `examples/` - Example configurations and usage patterns
- `fixtures/` - Test data and keys for development
- `packages/` - Rust workspace packages (linting tools)

## 📄 Key Configuration Files

- `.markdownlint.json` - Markdown linting rules
- `.yamllint-ci.yml` - YAML linting configuration
- `.taplo.toml` - TOML formatting and linting
- `cspell.json` - Spell checking configuration
- `project-words.txt` - Project-specific dictionary

## 🔧 Essential Rules

1. **Before creating branches**: Read [`docs/contributing/branching.md`](../docs/contributing/branching.md) for naming conventions (`{issue-number}-{short-description}`)

2. **Before committing**: Read [`docs/contributing/commit-process.md`](../docs/contributing/commit-process.md) for conventional commits

   - **With issue branch**: `{type}: [#{issue}] {description}` (when branch name starts with `{issue-number}-`)
   - **Without issue branch**: `{type}: {description}` (when working on main or branch without issue number prefix)

3. **Before committing**: Always run these verifications - all must pass before staging files or creating commits, regardless of the tool or method used:

   ```bash
   cargo machete                    # Run cargo machete (MANDATORY - no unused dependencies)
   cargo run --bin linter all       # Run linters (comprehensive - stable & nightly toolchains)
   cargo test                       # Run tests
   cargo run --bin e2e-tests-full   # Run comprehensive e2e tests
   ```

   This applies to **any** method of committing:

   - Terminal: `git add`, `git commit`, `git commit -am`, `cd ../ && git add ...`, `git add . && git commit -m "..."`
   - VS Code: Git panel, Source Control view, commit shortcuts
   - IDEs: IntelliJ, CLion, RustRover git integration
   - Git clients: GitHub Desktop, GitKraken, etc.
   - CI/CD: Any automated commits or merges

4. **Before working with Tera templates**: Read [`docs/contributing/templates.md`](../docs/contributing/templates.md) for correct variable syntax - use `{{ variable }}` not `{ { variable } }`. Tera template files have the `.tera` extension.

## 🧪 Build & Test

- **Lint**: `cargo run --bin linter all` (comprehensive - tests stable & nightly toolchains)
  - Individual linters: `cargo run --bin linter {markdown|yaml|toml|clippy|rustfmt|shellcheck}`
  - Alternative: `./scripts/lint.sh` (wrapper that calls the Rust binary)
- **Dependencies**: `cargo machete` (mandatory before commits - no unused dependencies)
- **Build**: `cargo build`
- **Test**: `cargo test`
- **Unit Tests**: When writing unit tests, follow conventions described in [`docs/contributing/testing.md`](../docs/contributing/testing.md)
- **E2E Tests**: `cargo run --bin e2e-tests-full` (comprehensive - all tests) or individual tests:
  - `cargo run --bin e2e-provision-tests` - Infrastructure provisioning tests
  - `cargo run --bin e2e-config-tests` - Configuration validation tests

Follow the project conventions and ensure all checks pass.
