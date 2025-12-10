# Tracker Deploy - AI Assistant Instructions

**Repository**: [torrust/torrust-tracker-deployer](https://github.com/torrust/torrust-tracker-deployer)

## 📋 Project Overview

This is a deployment infrastructure proof-of-concept for the Torrust ecosystem. It uses OpenTofu (Terraform), Ansible, and Rust to provision and manage deployment environments with LXD VM instances.

### Architecture

- **DDD Layers**: The codebase follows Domain-Driven Design with `domain/` (business logic), `application/` (use cases and commands), and `infrastructure/` (external integrations) layers.
- **Three-Level Pattern**: Commands orchestrate Steps, which execute remote Actions - providing clear separation of concerns across deployment operations (see `docs/codebase-architecture.md`).

## 🏗️ Tech Stack

- **Languages**: Rust, Shell scripts, YAML, TOML
- **Infrastructure**: OpenTofu (Terraform), Ansible
- **Virtualization Providers**: LXD VM instances
- **Tools**: Docker, cloud-init, testcontainers
- **Linting Tools**: markdownlint, yamllint, shellcheck, clippy, rustfmt, taplo (TOML)

## 📁 Key Directories

- `src/` - Rust source code organized by DDD layers (`domain/`, `application/`, `infrastructure/`, `presentation/`, `shared/`)
- `src/bin/` - Binary executables (linter, E2E tests, dependency installer)
- `data/` - Environment-specific data and source templates
- `templates/` - Generated template examples and test fixtures
- `build/` - Generated runtime configurations (git-ignored)
- `envs/` - User environment configurations (git-ignored) - recommended directory for storing environment JSON files passed to `create environment --env-file`. Contains user-specific deployment configurations that should not be committed to version control
- `docs/` - Project documentation
- `docs/user-guide/` - User-facing documentation (getting started, commands, configuration)
- `docs/decisions/` - Architectural Decision Records (ADRs)
- `scripts/` - Shell scripts for development tasks
- `fixtures/` - Test data and keys for development
- `packages/` - Rust workspace packages (see `packages/README.md` for details)
  - `dependency-installer/` - Dependency detection and installation for development setup
  - `linting/` - Unified linting framework

## 📄 Key Configuration Files

- `.markdownlint.json` - Markdown linting rules
- `.yamllint-ci.yml` - YAML linting configuration
- `.taplo.toml` - TOML formatting and linting
- `cspell.json` - Spell checking configuration
- `project-words.txt` - Project-specific dictionary

## Essential Principles

The development of this application is guided by fundamental principles that ensure quality, maintainability, and user experience. For detailed information, see [`docs/development-principles.md`](docs/development-principles.md).

**Core Principles:**

- **Observability**: If it happens, we can see it - even after it happens (includes deep traceability)
- **Testability**: Every component must be testable in isolation and as part of the whole
- **User Friendliness**: All errors must be clear, informative, and solution-oriented
- **Actionability**: The system must always tell users how to continue with detailed instructions

**Code Quality Standards:**

Both production and test code must be:

- **Clean**: Well-structured with clear naming and minimal complexity
- **Maintainable**: Easy to modify and extend without breaking existing functionality
- **Sustainable**: Long-term viability with proper documentation and patterns
- **Readable**: Clear intent that can be understood by other developers
- **Testable**: Designed to support comprehensive testing at all levels

These principles should guide all development decisions, code reviews, and feature implementations.

## 🔧 Essential Rules

1. **Before placing code in DDD layers**: Read [`docs/contributing/ddd-layer-placement.md`](docs/contributing/ddd-layer-placement.md) for comprehensive guidance on which code belongs in which layer (Domain, Application, Infrastructure, Presentation). This guide includes rules, red flags, examples, and a decision flowchart to help you make the right architectural decisions.

2. **Before creating branches**: Read [`docs/contributing/branching.md`](docs/contributing/branching.md) for naming conventions (`{issue-number}-{short-description}`)

3. **Before committing**: Read [`docs/contributing/commit-process.md`](docs/contributing/commit-process.md) for conventional commits

   - **With issue branch**: `{type}: [#{issue}] {description}` (when branch name starts with `{issue-number}-`)
   - **Without issue branch**: `{type}: {description}` (when working on main or branch without issue number prefix)

4. **Before committing**: Always run the pre-commit verification script - all checks must pass before staging files or creating commits, regardless of the tool or method used:

   ```bash
   ./scripts/pre-commit.sh
   ```

   This applies to **any** method of committing:

   - Terminal: `git add`, `git commit`, `git commit -am`, `cd ../ && git add ...`, `git add . && git commit -m "..."`
   - VS Code: Git panel, Source Control view, commit shortcuts
   - IDEs: IntelliJ, CLion, RustRover git integration
   - Git clients: GitHub Desktop, GitKraken, etc.
   - CI/CD: Any automated commits or merges

5. **Before working with Tera templates**: Read [`docs/contributing/templates.md`](docs/contributing/templates.md) for correct variable syntax - use `{{ variable }}` not `{ { variable } }`. Tera template files have the `.tera` extension.

6. **When adding new Ansible playbooks**: Read [`docs/contributing/templates.md`](docs/contributing/templates.md) for the complete guide. **CRITICAL**: Static playbooks (without `.tera` extension) must be registered in `src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs` in the `copy_static_templates` method, otherwise they won't be copied to the build directory and Ansible will fail with "playbook not found" error.

7. **When handling errors in code**: Read [`docs/contributing/error-handling.md`](docs/contributing/error-handling.md) for error handling principles. Prefer explicit enum errors over anyhow for better pattern matching and user experience. Make errors clear, include sufficient context for traceability, and ensure they are actionable with specific fix instructions.

8. **Understanding expected errors**: Read [`docs/contributing/known-issues.md`](docs/contributing/known-issues.md) for known issues and expected behaviors. Some errors that appear red in E2E test output (like SSH host key warnings) are normal and expected - not actual failures.

9. **Before making engineering decisions**: Document significant architectural or design decisions as Architectural Decision Records (ADRs) in `docs/decisions/`. Read [`docs/decisions/README.md`](docs/decisions/README.md) for the ADR template and guidelines. This ensures decisions are properly documented with context, rationale, and consequences for future reference.

10. **When organizing code within modules**: Follow the module organization conventions in [`docs/contributing/module-organization.md`](docs/contributing/module-organization.md). Use top-down organization with public items first, high-level abstractions before low-level details, and important responsibilities before secondary concerns like error types.

11. **When writing Rust imports** (CRITICAL for code style): Follow the import conventions in [`docs/contributing/module-organization.md`](docs/contributing/module-organization.md). Two essential rules:

    - **Imports Always First**: Keep all imports at the top of the file, organized in groups (std → external crates → internal crate).
    - **Prefer Imports Over Full Paths**: Always import types and use short names (e.g., `Arc<UserOutput>`) rather than fully-qualified paths. Never use long paths like `std::sync::Arc<crate::presentation::views::UserOutput>` in regular code - only use full paths when disambiguating naming conflicts.

12. **When writing Markdown documentation**: Be aware of GitHub Flavored Markdown's automatic linking behavior. Read [`docs/contributing/github-markdown-pitfalls.md`](docs/contributing/github-markdown-pitfalls.md) for critical patterns to avoid. **NEVER use hash-number patterns for enumeration or step numbering** - this creates unintended links to GitHub issues/PRs. Use ordered lists or alternative formats instead.

13. **When creating new environment variables**: Read [`docs/contributing/environment-variables-naming.md`](docs/contributing/environment-variables-naming.md) for comprehensive guidance on naming conventions (condition-based vs action-based), decision frameworks, and best practices. Also review [`docs/decisions/environment-variable-prefix.md`](docs/decisions/environment-variable-prefix.md) to ensure all project environment variables use the `TORRUST_TD_` prefix for proper namespacing and avoiding conflicts with system or user variables.

14. **When adding new templates**: Read [`docs/technical/template-system-architecture.md`](docs/technical/template-system-architecture.md) to understand the Project Generator pattern. The `templates/` directory contains source templates. Dynamic templates (`.tera`) are automatically processed, but static files must be explicitly registered in their respective `ProjectGenerator` to be copied to the build directory.

## 🧪 Build & Test

- **Setup Dependencies**: `cargo run --bin dependency-installer install` (sets up required development tools)
  - **Check dependencies**: `cargo run --bin dependency-installer check` (verifies installation)
  - **List dependencies**: `cargo run --bin dependency-installer list` (shows all dependencies with status)
  - Required tools: OpenTofu, Ansible, LXD, cargo-machete
  - See [`packages/dependency-installer/README.md`](packages/dependency-installer/README.md) for details
- **Lint**: `cargo run --bin linter all` (comprehensive - tests stable & nightly toolchains)
  - Individual linters: `cargo run --bin linter {markdown|yaml|toml|cspell|clippy|rustfmt|shellcheck}`
  - Alternative: `./scripts/lint.sh` (wrapper that calls the Rust binary)
- **Dependencies**: `cargo machete` (mandatory before commits - no unused dependencies)
- **Build**: `cargo build`
- **Test**: `cargo test`
- **Unit Tests**: When writing unit tests, follow conventions described in [`docs/contributing/testing/`](docs/contributing/testing/)
- **E2E Tests**:
  - `cargo run --bin e2e-complete-workflow-tests` - Comprehensive tests (⚠️ **LOCAL ONLY** - cannot run on GitHub Actions due to network connectivity issues)
  - `cargo run --bin e2e-infrastructure-lifecycle-tests` - Infrastructure provisioning and destruction tests (GitHub runner-compatible)
  - `cargo run --bin e2e-deployment-workflow-tests` - Software installation, configuration, release, and run workflow tests (GitHub runner-compatible)
  - Pre-commit hook runs the split tests (`e2e-infrastructure-lifecycle-tests` + `e2e-deployment-workflow-tests`) for GitHub Copilot compatibility
  - See [`docs/e2e-testing.md`](docs/e2e-testing.md) for detailed information about CI limitations

Follow the project conventions and ensure all checks pass.
