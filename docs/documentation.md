# Documentation Organization Guide

This guide explains how documentation is organized in this project to help contributors understand where to place different types of documentation.

## 📁 Documentation Structure

```text
docs/
├── contributing/        # Contribution guidelines and workflows
│   ├── README.md        # Contribution guide overview
│   ├── branching.md     # Branching conventions
│   ├── commit-process.md # Commit process and pre-commit checks
│   └── linting.md       # Linting tools and conventions
├── github-actions-issues/ # GitHub Actions runner issue documentation
│   ├── README.md        # Issue documentation index
│   └── docker-apt-cache-issue.md # Docker installation APT cache problems
├── tech-stack/          # Generic technology documentation
│   ├── ansible.md       # Ansible installation, setup, and basic usage
│   ├── lxd.md          # LXD system containers overview and setup
│   └── opentofu.md     # OpenTofu/Terraform installation and usage
├── research/           # Research notes and exploration docs
├── decisions/          # Architecture Decision Records (ADRs)
│   └── meson-removal.md # Decision to remove Meson build system
└── *.md               # Project-specific documentation
```

## 📝 Documentation Categories

### 🤝 Contribution Documentation (`docs/contributing/`)

**Purpose**: Guidelines and conventions for project contributors.

**What belongs here**:

- Branching conventions and naming standards
- Commit message standards and pre-commit processes
- Code quality standards and linting setup
- Development environment setup
- Pull request and review processes
- Testing guidelines and requirements

**Examples**:

- `branching.md` - Branch naming conventions
- `commit-process.md` - Conventional commits and pre-commit checks
- `linting.md` - Linting tools, configuration, and usage

### 🚀 GitHub Actions Issues (`docs/github-actions-issues/`)

**Purpose**: Documentation of recurring issues and solutions specific to GitHub Actions runners.

**What belongs here**:

- Package installation failures in containerized environments
- Networking issues specific to GitHub Actions runners
- Container/VM provisioning problems in CI environments
- APT cache and repository availability issues
- Solutions and workarounds for CI-specific problems
- Debugging techniques for GitHub Actions environments

**Examples**:

- `docker-apt-cache-issue.md` - Docker installation APT cache problems and solutions
- Network connectivity issues between containers and VMs in GitHub Actions
- Permission and security context problems in CI environments

### 🔧 Tech Stack Documentation (`docs/tech-stack/`)

**Purpose**: Generic documentation about tools and technologies used in the project.

**What belongs here**:

- Installation guides for tools (Ansible, OpenTofu, LXD, etc.)
- Basic usage examples and common commands
- General troubleshooting for the technology itself
- Tool-agnostic best practices
- Technology comparisons and explanations

**What does NOT belong here**:

- Project-specific configurations
- Custom scripts or automation specific to this project
- Integration details between this project and the tools

**Examples**:

- `ansible.md` - How to install Ansible, basic playbook structure, common commands
- `lxd.md` - LXD installation, container management, troubleshooting
- `opentofu.md` - OpenTofu installation, basic commands, state management

### 🔬 Research Documentation (`docs/research/`)

**Purpose**: Exploration, analysis, and decision-making documentation.

**What belongs here**:

- Technology research and comparisons
- Proof-of-concept findings
- Experimental approaches and their outcomes
- Preliminary investigations and findings

### 🎯 Architecture Decisions (`docs/decisions/`)

**Purpose**: Records of significant architectural and technical decisions.

**What belongs here**:

- Architecture Decision Records (ADRs)
- Technology adoption or removal decisions
- Process and workflow changes
- Design pattern choices and rationales
- Tool and dependency decisions

**Examples**:

- `meson-removal.md` - Decision to remove Meson build system and rationale

### 📋 Project Documentation (`docs/*.md`)

**Purpose**: Project-specific documentation and guides.

**What belongs here**:

- Project overview and architecture
- Specific implementation details
- Integration guides between technologies
- Project-specific troubleshooting
- User guides for this specific project

**Examples**:

- `vm-providers.md` - Comparison of VM providers for this project
- `structured-logging-implementation-plan.md` - Implementation plan for hierarchical logging with tracing spans
- Project-specific usage patterns and workflows

### 📁 Configuration Documentation (`templates/*/README.md`)

**Purpose**: Documentation for specific configurations within the project.

**What belongs here**:

- How to use the specific configuration
- Configuration-specific setup steps
- Customization options for that configuration
- Troubleshooting specific to that configuration

**Examples**:

- `templates/tofu/lxd/README.md` - How to use the LXD OpenTofu configuration
- `docs/contributing/templates/ansible.md` - How to use the Ansible playbooks

## 🎯 Guidelines for Contributors

### When adding new documentation

1. **Generic tool documentation** → `docs/tech-stack/`

   - If it could be useful in other projects using the same tool

2. **GitHub Actions runner issues** → `docs/github-actions-issues/`

   - If documenting CI-specific problems and solutions
   - When tracking recurring GitHub Actions runner issues

3. **Project-specific documentation** → `docs/`

   - If it's specific to how this project works

4. **Configuration documentation** → `templates/*/README.md`

   - If it's about a specific configuration or setup

5. **Research and exploration** → `docs/research/`

   - If it's about investigating or comparing approaches

6. **Architecture decisions** → `docs/decisions/`
   - If documenting a significant technical or architectural decision
   - When removing or adding major dependencies
   - For process or workflow changes that affect contributors

### Cross-referencing

- Always use relative paths when linking between documents
- Tech stack docs should link to project-specific implementations
- Configuration docs should link to relevant tech stack documentation
- Use descriptive link text that explains what the reader will find

### Example cross-references

```markdown
<!-- From config documentation to tech stack -->

For general LXD setup, see the [LXD documentation](../../docs/tech-stack/lxd.md).

<!-- From project docs to tech stack -->

Install OpenTofu following the [OpenTofu setup guide](tech-stack/opentofu.md).

<!-- From tech stack to project usage -->

For project-specific usage, see the [LXD configuration guide](../templates/tofu/lxd/README.md).
```

## 🔄 Maintaining Documentation

### When updating tools

1. **Update tech stack docs** for generic tool changes
2. **Update project docs** for project-specific impacts
3. **Update configuration docs** for configuration-specific changes
4. **Check all cross-references** remain valid

### When adding new tools

1. **Create tech stack documentation** for the tool itself
2. **Document project integration** in project docs
3. **Create configuration documentation** if there are specific configurations
4. **Update this guide** if new categories are needed

### When making architectural decisions

1. **Document significant decisions** in `docs/decisions/`
2. **Use descriptive filenames** (e.g., `tool-name-adoption.md`, `framework-removal.md`)
3. **Include context and reasoning** for future contributors
4. **Document when to reconsider** the decision

### Decision document format

Decision documents should include:

- **Status** (adopted, removed, superseded)
- **Context** that led to the decision
- **Problems** or requirements that drove the decision
- **Decision** made and alternatives considered
- **Consequences** both positive and negative
- **When to reconsider** the decision

This organization ensures documentation is:

- **Easy to find** - Clear categories for different types of information
- **Maintainable** - Generic docs don't get duplicated across configurations
- **Scalable** - New tools and configurations have clear places to go
- **Useful** - Readers can find both generic help and project-specific guidance
