# Documentation Index

Welcome to the Torrust Tracker Deployer documentation! This index helps you quickly find the right documentation for your needs.

## 📚 Quick Reference

### 🎯 Most Important Documents (Read These First)

| When You Need To...                  | Read This Document                                                                                                       |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| Understand the codebase architecture | [`codebase-architecture.md`](codebase-architecture.md) - DDD layers, module organization                                 |
| Place code in the correct layer      | [`contributing/ddd-layer-placement.md`](contributing/ddd-layer-placement.md) - **CRITICAL** decision flowchart           |
| Follow development principles        | [`development-principles.md`](development-principles.md) - Observability, Testability, User Friendliness, Actionability  |
| Commit code                          | [`contributing/commit-process.md`](contributing/commit-process.md) - Pre-commit checks, conventional commits             |
| Handle errors properly               | [`contributing/error-handling.md`](contributing/error-handling.md) - Explicit enums, actionable messages                 |
| Organize Rust code                   | [`contributing/module-organization.md`](contributing/module-organization.md) - **CRITICAL** import conventions           |
| Work with templates                  | [`contributing/templates.md`](contributing/templates.md) - **CRITICAL** Tera syntax, registration                        |
| Write unit tests                     | [`contributing/testing/unit-testing.md`](contributing/testing/unit-testing.md) - Naming conventions (no `test_` prefix!) |
| Run E2E tests                        | [`e2e-testing/README.md`](e2e-testing/README.md) - Quick start, test suites                                              |
| Understand a past decision           | [`decisions/README.md`](decisions/README.md) - 30+ ADRs indexed                                                          |
| Use the CLI                          | [`user-guide/README.md`](user-guide/README.md) - Complete user guide                                                     |

### 📂 Documentation Directory Structure

```text
docs/
├── contributing/          🤝 Contribution guidelines (branching, commits, DDD, errors, templates, testing)
├── decisions/            📋 Architectural Decision Records (30+ ADRs with context and rationale)
├── e2e-testing/          🧪 E2E test documentation (architecture, running, manual, troubleshooting)
├── features/             ✨ Feature specifications and development tracking (5 active features)
├── user-guide/           📖 User documentation (commands, providers, quick start)
├── tech-stack/           🛠️ Technology docs (Ansible, LXD, OpenTofu, SSH)
├── technical/            🔧 Technical deep dives (template system, type erasure patterns)
├── research/             🔬 Research notes (Ansible testing, UX patterns, MVVM analysis)
├── refactors/            🔄 Refactoring plans and tracking
├── implementation-plans/ 📝 Step-by-step plans for complex changes
├── issues/               📋 Issue templates and specifications
├── github-actions-issues/ ⚠️ CI/CD troubleshooting
└── analysis/             📊 Code analysis (presentation layer structure)
```

## 📚 Documentation by Category

### For Users

- **Getting Started**: [`user-guide/README.md`](user-guide/README.md), [`user-guide/quick-start.md`](user-guide/quick-start.md)
- **Commands**: [`user-guide/commands/`](user-guide/commands/), [`console-commands.md`](console-commands.md)
- **Providers**: [`user-guide/providers/`](user-guide/providers/)

### For Contributors

- **Contributing Guide**: [`contributing/README.md`](contributing/README.md)
- **Architecture**: [`codebase-architecture.md`](codebase-architecture.md), [`contributing/ddd-layer-placement.md`](contributing/ddd-layer-placement.md)
- **Testing**: [`contributing/testing/`](contributing/testing/), [`e2e-testing/`](e2e-testing/)
- **Code Quality**: [`contributing/linting.md`](contributing/linting.md), [`development-principles.md`](development-principles.md)

### For Maintainers

- **Decisions**: [`decisions/`](decisions/) (30+ ADRs)
- **Features**: [`features/`](features/) (active and planned features)
- **Refactoring**: [`refactors/`](refactors/) (ongoing improvements)
- **Roadmap**: [`roadmap.md`](roadmap.md)

### For Researchers/Architects

- **Research**: [`research/`](research/) (testing strategies, UX patterns, MVVM analysis)
- **Analysis**: [`analysis/`](analysis/) (code structure analysis)
- **Vision**: [`vision-infrastructure-as-software.md`](vision-infrastructure-as-software.md)

## 🎯 Quick Navigation by Task

### "I want to..."

| Task                      | Start Here                                                                     |
| ------------------------- | ------------------------------------------------------------------------------ |
| Start using the deployer  | [`user-guide/README.md`](user-guide/README.md)                                 |
| Contribute code           | [`contributing/README.md`](contributing/README.md)                             |
| Understand architecture   | [`codebase-architecture.md`](codebase-architecture.md)                         |
| Add code to correct layer | [`contributing/ddd-layer-placement.md`](contributing/ddd-layer-placement.md)   |
| Run E2E tests             | [`e2e-testing/README.md`](e2e-testing/README.md)                               |
| Write unit tests          | [`contributing/testing/unit-testing.md`](contributing/testing/unit-testing.md) |
| Understand a decision     | [`decisions/README.md`](decisions/README.md)                                   |
| Plan a new feature        | [`features/README.md`](features/README.md)                                     |
| Fix a CI issue            | [`github-actions-issues/README.md`](github-actions-issues/README.md)           |
| Work with templates       | [`contributing/templates.md`](contributing/templates.md)                       |
| Handle errors properly    | [`contributing/error-handling.md`](contributing/error-handling.md)             |
| Organize Rust modules     | [`contributing/module-organization.md`](contributing/module-organization.md)   |

---

## 📋 Complete Documentation Inventory

<details>
<summary><strong>📁 Root Level Documentation (Click to Expand)</strong></summary>

| File                                                                           | Description                                                                                                                                                   |
| ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [`codebase-architecture.md`](codebase-architecture.md)                         | Comprehensive overview of DDD architecture, layer structure (Domain, Application, Infrastructure, Presentation), and module organization                      |
| [`console-commands.md`](console-commands.md)                                   | Complete reference for CLI commands, deployment states, and implementation status of all commands (create, provision, configure, release, run, destroy, etc.) |
| [`deployment-overview.md`](deployment-overview.md)                             | High-level overview of deployment lifecycle states, command relationships, and state transitions with visual diagrams                                         |
| [`development-principles.md`](development-principles.md)                       | Core principles: Observability, Testability, User Friendliness, Actionability - guiding all development decisions and code quality standards                  |
| [`documentation.md`](documentation.md)                                         | Guide explaining documentation organization structure, categories, and where to place different types of documentation                                        |
| [`linting.md`](linting.md)                                                     | Linting tool usage, configuration files, and quick commands for maintaining code quality across all file types                                                |
| [`roadmap.md`](roadmap.md)                                                     | Development roadmap with epics, features, and implementation status tracking                                                                                  |
| [`roadmap-questions.md`](roadmap-questions.md)                                 | Open questions and discussions about roadmap priorities and future development direction                                                                      |
| [`tofu-lxd-configuration.md`](tofu-lxd-configuration.md)                       | OpenTofu configuration details for LXD provider setup and usage                                                                                               |
| [`vision-infrastructure-as-software.md`](vision-infrastructure-as-software.md) | Long-term vision for evolving Infrastructure as Code to Infrastructure as Software with DDD principles                                                        |
| [`vm-providers.md`](vm-providers.md)                                           | Overview of supported VM providers (LXD, Hetzner) and provider architecture                                                                                   |

</details>

<details>
<summary><strong>🤝 Contributing Guidelines (18 documents) (Click to Expand)</strong></summary>

Essential guides: DDD layer placement, module organization, error handling, templates, commit process, testing conventions

**Key Documents:**

- [`contributing/README.md`](contributing/README.md) - Quick reference guide to all contribution documentation
- [`contributing/ddd-layer-placement.md`](contributing/ddd-layer-placement.md) - **CRITICAL**: Rules for placing code in correct DDD layers
- [`contributing/module-organization.md`](contributing/module-organization.md) - **CRITICAL**: Module organization and Rust import patterns
- [`contributing/templates.md`](contributing/templates.md) - **CRITICAL**: Tera template syntax and registration
- [`contributing/error-handling.md`](contributing/error-handling.md) - Error handling principles
- [`contributing/commit-process.md`](contributing/commit-process.md) - Commit process and pre-commit checks
- [`contributing/testing/unit-testing.md`](contributing/testing/unit-testing.md) - Unit testing conventions

</details>

<details>
<summary><strong>📋 Architectural Decision Records (30+ ADRs) (Click to Expand)</strong></summary>

Index at [`decisions/README.md`](decisions/README.md) - All architectural decisions documented with context, rationale, and consequences

**Recent Notable Decisions:**

- Cloud-Init SSH Port Configuration with Reboot
- Single Docker Image for Sequential E2E Command Testing
- Register Command SSH Port Override
- Migration to AGENTS.md Standard
- ReentrantMutex Pattern for UserOutput
- Environment Variable Prefix (`TORRUST_TD_`)
- Command State Return Pattern
- Actionable Error Messages
- LXD VMs over Containers
- Tera Minimal Templating Strategy

</details>

<details>
<summary><strong>🧪 E2E Testing Documentation (8 documents) (Click to Expand)</strong></summary>

Architecture, running tests, manual testing, troubleshooting, test suites, contributing, advanced techniques

**Key Documents:**

- [`e2e-testing/README.md`](e2e-testing/README.md) - Overview and quick start
- [`e2e-testing/running-tests.md`](e2e-testing/running-tests.md) - How to run automated tests
- [`e2e-testing/manual-testing.md`](e2e-testing/manual-testing.md) - Manual testing guide
- [`e2e-testing/troubleshooting.md`](e2e-testing/troubleshooting.md) - Common issues and solutions
- [`e2e-testing/architecture.md`](e2e-testing/architecture.md) - Testing architecture and design

</details>

<details>
<summary><strong>✨ Feature Development (7+ features) (Click to Expand)</strong></summary>

Feature specs, templates, 5 active features

**Active Features:**

- Hetzner Provider Support
- Import Existing Instances
- Hybrid Command Architecture
- Linter Auto-fix
- Linter Parallel Execution (deferred)
- Environment State Management (refactoring)
- Progress Reporting in Application Layer

See [`features/README.md`](features/README.md) for complete list and templates.

</details>

<details>
<summary><strong>📖 User Documentation (Click to Expand)</strong></summary>

Complete user guide with command documentation, provider guides, quick start, logging, template customization

**Key Documents:**

- [`user-guide/README.md`](user-guide/README.md) - Complete user guide
- [`user-guide/quick-start.md`](user-guide/quick-start.md) - Quick start guide
- [`user-guide/commands/`](user-guide/commands/) - Individual command documentation (8 commands)
- [`user-guide/providers/`](user-guide/providers/) - Provider-specific guides (LXD, Hetzner)

</details>

<details>
<summary><strong>🛠️ Technology Documentation (4 documents) (Click to Expand)</strong></summary>

Ansible, LXD, OpenTofu, SSH keys - installation, setup, and usage

- [`tech-stack/ansible.md`](tech-stack/ansible.md)
- [`tech-stack/lxd.md`](tech-stack/lxd.md)
- [`tech-stack/opentofu.md`](tech-stack/opentofu.md)
- [`tech-stack/ssh-keys.md`](tech-stack/ssh-keys.md)

</details>

<details>
<summary><strong>🔧 Technical Deep Dives (2 documents) (Click to Expand)</strong></summary>

- [`technical/template-system-architecture.md`](technical/template-system-architecture.md) - **CRITICAL**: Template system architecture and Project Generator pattern
- [`technical/type-erasure-pattern.md`](technical/type-erasure-pattern.md) - Type erasure pattern for Environment states

</details>

<details>
<summary><strong>🔬 Research Notes (Click to Expand)</strong></summary>

Ansible testing strategy, Docker vs LXD, E2E testing, UX patterns, MVVM analysis, presentation layer organization

**Key Documents:**

- [`research/ansible-testing-strategy.md`](research/ansible-testing-strategy.md) - Comprehensive Ansible testing strategy
- [`research/docker-vs-lxd-ansible-testing.md`](research/docker-vs-lxd-ansible-testing.md) - Technology comparison
- [`research/UX/`](research/UX/) - UX research documents (console output patterns, logging strategy)
- [`research/mvvm-pattern-analysis/`](research/mvvm-pattern-analysis/) - MVVM pattern research

</details>

<details>
<summary><strong>🔄 Other Documentation Categories (Click to Expand)</strong></summary>

**Refactoring Plans:**

- [`refactors/README.md`](refactors/README.md) - Process, templates, active/completed refactorings

**Implementation Plans:**

- [`implementation-plans/README.md`](implementation-plans/README.md) - Step-by-step plans for complex changes

**Issue Documentation:**

- [`issues/`](issues/) - Templates for epics, issues, and specifications

**CI/CD Troubleshooting:**

- [`github-actions-issues/README.md`](github-actions-issues/README.md) - GitHub Actions runner issues

**Code Analysis:**

- [`analysis/presentation-layer/`](analysis/presentation-layer/) - Presentation layer structure analysis

</details>

---

## 📊 Statistics

- **Total Documentation Files**: 160+ markdown files
- **Major Categories**: 13 top-level directories
- **ADRs**: 30+ architectural decisions documented
- **Active Features**: 5 features in various stages
- **Test Documentation**: Comprehensive E2E testing guides
- **Contribution Guides**: 18 contributor-focused documents
