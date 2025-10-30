Tracker Deploy - AI Assistant Instructions

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

- `src/` - Rust source code organized by DDD layers (`domain/`, `application/`, `infrastructure/`, `shared/`)
- `src/bin/` - Binary executables (linter, E2E tests)
- `data/` - Environment-specific data and source templates
- `templates/` - Generated template examples and test fixtures
- `build/` - Generated runtime configurations (git-ignored)
- `docs/` - Project documentation
- `docs/user-guide/` - User-facing documentation (getting started, commands, configuration)
- `docs/decisions/` - Architectural Decision Records (ADRs)
- `scripts/` - Shell scripts for development tasks
- `fixtures/` - Test data and keys for development
- `packages/` - Rust workspace packages (linting tools)

## 📄 Key Configuration Files

- `.markdownlint.json` - Markdown linting rules
- `.yamllint-ci.yml` - YAML linting configuration
- `.taplo.toml` - TOML formatting and linting
- `cspell.json` - Spell checking configuration
- `project-words.txt` - Project-specific dictionary

## Essential Principles

The development of this application is guided by fundamental principles that ensure quality, maintainability, and user experience. For detailed information, see [`docs/development-principles.md`](../docs/development-principles.md).

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

1. **Before creating branches**: Read [`docs/contributing/branching.md`](../docs/contributing/branching.md) for naming conventions (`{issue-number}-{short-description}`)

2. **Before committing**: Read [`docs/contributing/commit-process.md`](../docs/contributing/commit-process.md) for conventional commits

   - **With issue branch**: `{type}: [#{issue}] {description}` (when branch name starts with `{issue-number}-`)
   - **Without issue branch**: `{type}: {description}` (when working on main or branch without issue number prefix)

3. **Before committing**: Always run the pre-commit verification script - all checks must pass before staging files or creating commits, regardless of the tool or method used:

   ```bash
   ./scripts/pre-commit.sh
   ```

   This applies to **any** method of committing:

   - Terminal: `git add`, `git commit`, `git commit -am`, `cd ../ && git add ...`, `git add . && git commit -m "..."`
   - VS Code: Git panel, Source Control view, commit shortcuts
   - IDEs: IntelliJ, CLion, RustRover git integration
   - Git clients: GitHub Desktop, GitKraken, etc.
   - CI/CD: Any automated commits or merges

4. **Before working with Tera templates**: Read [`docs/contributing/templates.md`](../docs/contributing/templates.md) for correct variable syntax - use `{{ variable }}` not `{ { variable } }`. Tera template files have the `.tera` extension.

5. **When adding new Ansible playbooks**: Read [`docs/contributing/templates.md`](../docs/contributing/templates.md) for the complete guide. **CRITICAL**: Static playbooks (without `.tera` extension) must be registered in `src/infrastructure/external_tools/ansible/template/renderer/mod.rs` in the `copy_static_templates` method, otherwise they won't be copied to the build directory and Ansible will fail with "playbook not found" error.

6. **When handling errors in code**: Read [`docs/contributing/error-handling.md`](../docs/contributing/error-handling.md) for error handling principles. Prefer explicit enum errors over anyhow for better pattern matching and user experience. Make errors clear, include sufficient context for traceability, and ensure they are actionable with specific fix instructions.

7. **Understanding expected errors**: Read [`docs/contributing/known-issues.md`](../docs/contributing/known-issues.md) for known issues and expected behaviors. Some errors that appear red in E2E test output (like SSH host key warnings) are normal and expected - not actual failures.

8. **Before making engineering decisions**: Document significant architectural or design decisions as Architectural Decision Records (ADRs) in `docs/decisions/`. Read [`docs/decisions/README.md`](../docs/decisions/README.md) for the ADR template and guidelines. This ensures decisions are properly documented with context, rationale, and consequences for future reference.

9. **When organizing code within modules**: Follow the module organization conventions in [`docs/contributing/module-organization.md`](../docs/contributing/module-organization.md). Use top-down organization with public items first, high-level abstractions before low-level details, and important responsibilities before secondary concerns like error types.

## 🧪 Build & Test

- **Lint**: `cargo run --bin linter all` (comprehensive - tests stable & nightly toolchains)
  - Individual linters: `cargo run --bin linter {markdown|yaml|toml|cspell|clippy|rustfmt|shellcheck}`
  - Alternative: `./scripts/lint.sh` (wrapper that calls the Rust binary)
- **Dependencies**: `cargo machete` (mandatory before commits - no unused dependencies)
- **Build**: `cargo build`
- **Test**: `cargo test`
- **Unit Tests**: When writing unit tests, follow conventions described in [`docs/contributing/testing/`](../docs/contributing/testing/)
- **E2E Tests**: `cargo run --bin e2e-tests-full` (comprehensive - all tests) or individual tests:
  - `cargo run --bin e2e-provision-tests` - Infrastructure provisioning tests
  - `cargo run --bin e2e-config-tests` - Configuration validation tests

Follow the project conventions and ensure all checks pass.
