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

**Beck's Four Rules of Simple Design:**

Follow Kent Beck's four rules of simple design (in priority order):

1. **Passes the tests**: The code must work as intended - testing is a first-class activity
2. **Reveals intention**: Code should be easy to understand, expressing purpose clearly
3. **No duplication**: Apply DRY (Don't Repeat Yourself) / Once and Only Once - eliminating duplication drives out good designs
4. **Fewest elements**: Remove anything that doesn't serve the prior three rules - avoid premature optimization for hypothetical future requirements

These rules feed off each other in refining code and apply to any language or paradigm. When in conflict, empathy for the reader wins over strict technical metrics.

Reference: [Beck Design Rules](https://martinfowler.com/bliki/BeckDesignRules.html)

These principles should guide all development decisions, code reviews, and feature implementations.

## 🔧 Essential Rules

1. **CRITICAL — `data/` is READ ONLY** (⚠️ **MOST FREQUENTLY VIOLATED**): Never create or edit files in `data/` — it contains application-managed deployment state. User configs belong in `envs/`. These are completely different JSON structures with different purposes. See the `create-environment-config` skill for details.

2. **Rust imports**: All imports at the top of the file, grouped (std → external crates → internal crate). Always prefer short imported names over fully-qualified paths (e.g., `Arc<UserOutput>`, not `std::sync::Arc<crate::presentation::views::UserOutput>`). Use full paths only to disambiguate naming conflicts.

3. **Continuous self-review**: All contributors (humans and AI agents) **must** continuously review their own work against the project's quality standards. Use the PR review checklist in [`docs/contributing/pr-review-guide.md`](docs/contributing/pr-review-guide.md) and the review skill in `.github/skills/dev/git-workflow/review-pr/skill.md` to systematically check your changes. Apply self-review at three levels:
   - **Mandatory** — before opening a pull request
   - **Strongly recommended** — before each commit
   - **Recommended** — after completing each small, independent, deployable change

   The sooner and more often you self-review, the less effort it takes to fix issues. Discovering problems early — while the change is fresh in your mind — is far cheaper than reworking code after a PR rejection. Treat self-review as a continuous habit, not a final gate.

## 🏗️ Deployed Instance Structure

After running the complete deployment workflow (`create → provision → configure → release → run`), the virtual machine has the following structure:

```text
/opt/torrust/                          # Application root directory
├── docker-compose.yml                 # Main orchestration file
├── .env                               # Environment variables
└── storage/                           # Persistent data volumes
    ├── tracker/
    │   ├── lib/                       # Database files (tracker.db for SQLite)
    │   ├── log/                       # Tracker logs
    │   └── etc/                       # Configuration (tracker.toml)
    ├── prometheus/
    │   └── etc/                       # Prometheus configuration
    └── grafana/
        ├── data/                      # Grafana database
        └── provisioning/              # Dashboards and datasources
```

**Key commands inside the VM**:

```bash
cd /opt/torrust                        # Application root
docker compose ps                      # Check services
docker compose logs tracker            # View logs
```

For detailed information about working with deployed instances, see [`docs/user-guide/`](docs/user-guide/README.md).

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
  - See [`docs/e2e-testing/`](docs/e2e-testing/) for detailed information about CI limitations
- **Manual E2E Testing**: For step-by-step manual testing with CLI commands, see [`docs/e2e-testing/manual-testing.md`](docs/e2e-testing/manual-testing.md). This guide covers:
  - Complete manual test workflow from template creation to deployment
  - Handling interrupted commands and state recovery
  - Troubleshooting common issues
  - Cleanup procedures for both application and LXD resources

Follow the project conventions and ensure all checks pass.

## 🎯 Auto-Invoke Skills

The project provides Agent Skills in `.github/skills/` for specialized workflows. Skills are loaded on-demand when tasks match their descriptions.

Available skills:

| Task                           | Skill to Load                                                                |
| ------------------------------ | ---------------------------------------------------------------------------- |
| Adding Ansible playbooks       | `.github/skills/dev/infrastructure/add-ansible-playbook/skill.md`            |
| Adding commands                | `.github/skills/dev/cli/add-new-command/skill.md`                            |
| Adding SDK examples            | `.github/skills/dev/sdk/add-sdk-example/skill.md`                            |
| Adding SDK methods             | `.github/skills/dev/sdk/add-sdk-method/skill.md`                             |
| Adding templates               | `.github/skills/dev/infrastructure/add-new-template/skill.md`                |
| Cleaning up completed issues   | `.github/skills/dev/planning/cleanup-completed-issues/skill.md`              |
| Checking system dependencies   | `.github/skills/usage/operations/check-system-dependencies/skill.md`         |
| Cleaning LXD environments      | `.github/skills/dev/testing/clean-lxd-environments/skill.md`                 |
| Committing changes             | `.github/skills/dev/git-workflow/commit-changes/skill.md`                    |
| Completing feature specs       | `.github/skills/dev/planning/complete-feature-spec/skill.md`                 |
| Completing refactor plans      | `.github/skills/dev/planning/complete-refactor-plan/skill.md`                |
| Config DTO architecture        | `.github/skills/dev/infrastructure/environment-config-architecture/skill.md` |
| Creating ADRs                  | `.github/skills/dev/planning/create-adr/skill.md`                            |
| Creating environment configs   | `.github/skills/usage/operations/create-environment-config/skill.md`         |
| Creating environment variables | `.github/skills/dev/infrastructure/create-environment-variables/skill.md`    |
| Creating feature branches      | `.github/skills/dev/git-workflow/create-feature-branch/skill.md`             |
| Creating feature specs         | `.github/skills/dev/planning/create-feature-spec/skill.md`                   |
| Creating issues                | `.github/skills/dev/planning/create-issue/skill.md`                          |
| Creating new skills            | `.github/skills/add-new-skill/skill.md`                                      |
| Creating refactor plans        | `.github/skills/dev/planning/create-refactor-plan/skill.md`                  |
| Debugging test errors          | `.github/skills/dev/testing/debug-test-errors/skill.md`                      |
| Handling errors in code        | `.github/skills/dev/rust-code-quality/handle-errors-in-code/skill.md`        |
| Handling secrets               | `.github/skills/dev/rust-code-quality/handle-secrets/skill.md`               |
| Handling user output           | `.github/skills/dev/cli/handle-user-output/skill.md`                         |
| Implementing domain types      | `.github/skills/dev/rust-code-quality/implement-domain-types/skill.md`       |
| Installing system dependencies | `.github/skills/usage/operations/install-system-dependencies/skill.md`       |
| Organizing Rust modules        | `.github/skills/dev/rust-code-quality/organize-rust-modules/skill.md`        |
| Placing code in DDD layers     | `.github/skills/dev/rust-code-quality/place-code-in-ddd-layers/skill.md`     |
| Regenerating CLI docs          | `.github/skills/dev/cli/regenerate-cli-docs/skill.md`                        |
| Rendering tracker artifacts    | `.github/skills/usage/operations/render-tracker-artifacts/skill.md`          |
| Reviewing pull requests        | `.github/skills/dev/git-workflow/review-pr/skill.md`                         |
| Running linters                | `.github/skills/dev/git-workflow/run-linters/skill.md`                       |
| Running local E2E tests        | `.github/skills/dev/testing/run-local-e2e-test/skill.md`                     |
| Running pre-commit checks      | `.github/skills/dev/git-workflow/run-pre-commit-checks/skill.md`             |
| Troubleshooting LXD instances  | `.github/skills/dev/testing/troubleshoot-lxd-instances/skill.md`             |
| Verifying template changes     | `.github/skills/dev/infrastructure/verify-template-changes/skill.md`         |
| Working with Tera templates    | `.github/skills/dev/infrastructure/work-with-tera-templates/skill.md`        |
| Writing Markdown docs          | `.github/skills/dev/planning/write-markdown-docs/skill.md`                   |
| Writing SDK integration tests  | `.github/skills/dev/sdk/write-sdk-integration-test/skill.md`                 |
| Writing unit tests             | `.github/skills/dev/testing/write-unit-test/skill.md`                        |

Skills supplement (not replace) the rules in this file. Rules apply always; skills activate when their workflows are needed.

**For VS Code**: Enable `chat.useAgentSkills` in settings to activate skill discovery.

**Learn more**: See [Agent Skills Specification (agentskills.io)](https://agentskills.io/specification) for the open format documentation.

## 📚 Documentation

The project has comprehensive documentation organized in the [`docs/`](docs/) directory. See the complete [Documentation Index](docs/README.md) for detailed navigation.

### Documentation by Category

**For Users:**

- Getting started: [`docs/user-guide/README.md`](docs/user-guide/README.md), [`docs/user-guide/quick-start/`](docs/user-guide/quick-start/README.md)
- Commands: [`docs/user-guide/commands/`](docs/user-guide/commands/), [`docs/console-commands.md`](docs/console-commands.md)
- Providers: [`docs/user-guide/providers/`](docs/user-guide/providers/)

**For Contributors:**

- Contributing Guide: [`docs/contributing/README.md`](docs/contributing/README.md)
- Architecture: [`docs/codebase-architecture.md`](docs/codebase-architecture.md), [`docs/contributing/ddd-layer-placement.md`](docs/contributing/ddd-layer-placement.md)
- Testing: [`docs/contributing/testing/`](docs/contributing/testing/), [`docs/e2e-testing/`](docs/e2e-testing/)
- Code Quality: [`docs/contributing/linting.md`](docs/contributing/linting.md), [`docs/development-principles.md`](docs/development-principles.md)

**For Maintainers:**

- Decisions: [`docs/decisions/`](docs/decisions/) (30+ ADRs)
- Features: [`docs/features/`](docs/features/) (active and planned features)
- Refactoring: [`docs/refactors/`](docs/refactors/) (ongoing improvements)
- Roadmap: [`docs/roadmap.md`](docs/roadmap.md)

**For Researchers/Architects:**

- Research: [`docs/research/`](docs/research/) (testing strategies, UX patterns, MVVM analysis)
- Analysis: [`docs/analysis/`](docs/analysis/) (code structure analysis)
- Vision: [`docs/vision-infrastructure-as-software.md`](docs/vision-infrastructure-as-software.md)

### Quick Navigation by Task

| Task                      | Start Here                                                                                                                     |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Start using the deployer  | [`docs/user-guide/README.md`](docs/user-guide/README.md)                                                                       |
| Self-review your changes  | [`docs/contributing/pr-review-guide.md`](docs/contributing/pr-review-guide.md)                                                 |
| Contribute code           | [`docs/contributing/README.md`](docs/contributing/README.md)                                                                   |
| Create a new issue        | [`docs/contributing/roadmap-issues.md`](docs/contributing/roadmap-issues.md)                                                   |
| Understand architecture   | [`docs/codebase-architecture.md`](docs/codebase-architecture.md)                                                               |
| Add code to correct layer | [`docs/contributing/ddd-layer-placement.md`](docs/contributing/ddd-layer-placement.md)                                         |
| Run E2E tests             | [`docs/e2e-testing/README.md`](docs/e2e-testing/README.md)                                                                     |
| Write unit tests          | [`docs/contributing/testing/unit-testing/naming-conventions.md`](docs/contributing/testing/unit-testing/naming-conventions.md) |
| Understand a decision     | [`docs/decisions/README.md`](docs/decisions/README.md)                                                                         |
| Plan a new feature        | [`docs/features/README.md`](docs/features/README.md)                                                                           |
| Fix external tool issues  | [`docs/external-issues/README.md`](docs/external-issues/README.md)                                                             |
| Work with templates       | [`docs/contributing/templates/`](docs/contributing/templates/)                                                                 |
| Handle errors properly    | [`docs/contributing/error-handling.md`](docs/contributing/error-handling.md)                                                   |
| Handle output properly    | [`docs/contributing/output-handling.md`](docs/contributing/output-handling.md)                                                 |
| Organize Rust modules     | [`docs/contributing/module-organization.md`](docs/contributing/module-organization.md)                                         |
