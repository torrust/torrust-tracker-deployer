# Update Documentation: Complete Command Workflow Available

**Issue**: [#192](https://github.com/torrust/torrust-tracker-deployer/issues/192)
**Parent Epic**: [#2 - Scaffolding for main app](https://github.com/torrust/torrust-tracker-deployer/issues/2)
**Related**:

- [Issue #180 - Configure command (UI layer)](https://github.com/torrust/torrust-tracker-deployer/issues/180)
- [Issue #188 - Test command (UI layer)](https://github.com/torrust/torrust-tracker-deployer/issues/188)

## Overview

Update all project documentation to reflect that the complete command workflow is now available to end-users. With the recent completion of the `configure` and `test` commands, users can now execute the full deployment lifecycle from environment creation to verification. This task involves updating existing documentation and creating new user-facing guides.

## Goals

- [ ] Update roadmap to mark tasks 1.7 and 1.8 as completed
- [ ] Update existing documentation to reflect all available commands
- [ ] Create comprehensive user guide for individual commands
- [ ] Create quick-start guide showing complete deployment workflow
- [ ] Ensure documentation consistency across all files

## 🏗️ Architecture Requirements

**DDD Layer**: N/A (Documentation only)
**Module Path**: N/A
**Pattern**: Documentation update

### Module Structure Requirements

N/A - This is a documentation-only change

### Architectural Constraints

N/A - No code changes required

### Anti-Patterns to Avoid

- ❌ Inconsistent command descriptions across documentation files
- ❌ Outdated information about available commands
- ❌ Missing commands in user-facing documentation
- ❌ Incomplete workflow examples
- ❌ Breaking markdown linting rules

## Specifications

### Complete Command Workflow

The application now supports the complete deployment lifecycle:

1. **`create template`** - Generate environment configuration template
2. **`create environment`** - Create deployment environment from configuration
3. **`provision`** - Provision VM infrastructure (LXD instances)
4. **`configure`** - Configure provisioned infrastructure (Docker, Docker Compose)
5. **`test`** - Verify deployment infrastructure
6. **`destroy`** - Tear down deployment environment

### Documentation Files to Update

#### 1. README.md

**Current state**: May not list all available commands or show the complete workflow.

**Required updates**:

- Update "Features" or "Commands" section to list all 6 commands
- Add quick example showing the basic workflow
- Update any outdated command references

#### 2. docs/codebase-architecture.md

**Current state**: Architecture documentation may reference old command structure.

**Required updates**:

- Update any command examples to use current commands
- Ensure presentation layer examples show the complete command set
- Update any diagrams or architecture descriptions

#### 3. docs/console-commands.md

**Current state**: Console commands documentation may be incomplete.

**Required updates**:

- Add `configure` command documentation
- Add `test` command documentation
- Ensure all commands are documented with:
  - Purpose and usage
  - Arguments and options
  - Example invocations
  - State transitions (e.g., "Created" → "Provisioned" → "Configured")

#### 4. docs/user-guide/commands/ (NEW)

**Create new directory structure**:

```text
docs/user-guide/commands/
├── README.md              # Overview of all commands
├── create-template.md     # Detailed guide for create template
├── create-environment.md  # Detailed guide for create environment
├── provision.md           # Detailed guide for provision
├── configure.md           # Detailed guide for configure
├── test.md                # Detailed guide for test
└── destroy.md             # Detailed guide for destroy
```

**Content for each command guide**:

- Command purpose and use case
- Prerequisites (required state, dependencies)
- Command syntax and arguments
- Step-by-step usage instructions
- Examples with sample output
- Common errors and troubleshooting
- State transition information
- Related commands

#### 5. docs/user-guide/quick-start.md (NEW)

**Create comprehensive quick-start guide**:

- **Title**: "Quick Start: Local Deployment Environment"
- **Audience**: End-users wanting to deploy locally with LXD
- **Content**:
  1. Prerequisites (LXD installed, dependencies)
  2. Step 1: Generate configuration template
  3. Step 2: Customize configuration
  4. Step 3: Create environment
  5. Step 4: Provision infrastructure
  6. Step 5: Configure infrastructure
  7. Step 6: Test deployment
  8. Step 7: Destroy environment (cleanup)
- Include actual command examples with sample output
- Include troubleshooting section
- Include "Next Steps" section

#### 6. docs/roadmap.md

**Current state**: Tasks 1.7 and 1.8 marked as incomplete:

```markdown
- [ ] **1.7** Create command `torrust-tracker-deployer configure` to configure provisioned infrastructure (UI layer only) - [Issue #180](https://github.com/torrust/torrust-tracker-deployer/issues/180)
  - **Note:** The App layer ConfigureCommand is already implemented, this task focuses on the console subcommand interface
  - Implementation should call the existing ConfigureCommandHandler business logic
  - Handle user input, validation, and output presentation
  - Enables transition from "provisioned" to "configured" state via CLI
- [ ] **1.8** Create command `torrust-tracker-deployer test` to verify deployment infrastructure (UI layer only) - [Issue #188](https://github.com/torrust/torrust-tracker-deployer/issues/188)
  - **Note:** The App layer TestCommandHandler is already implemented, this task focuses on the console subcommand interface
  - Implementation should call the existing TestCommandHandler business logic
  - Handle user input, validation, and output presentation
  - Enables verification of deployment state via CLI (cloud-init, Docker, Docker Compose)
```

### Desired State

Both tasks should be marked as completed with the ✅ emoji, following the same format as other completed tasks (see tasks 1.1, 1.2, 1.4, 1.5, 1.6 as examples):

```markdown
- [x] **1.7** Create command `torrust-tracker-deployer configure` to configure provisioned infrastructure (UI layer only) ✅ Completed - [Issue #180](https://github.com/torrust/torrust-tracker-deployer/issues/180)
  - **Note:** The App layer ConfigureCommand is already implemented, this task focuses on the console subcommand interface
  - Implementation should call the existing ConfigureCommandHandler business logic
  - Handle user input, validation, and output presentation
  - Enables transition from "provisioned" to "configured" state via CLI
- [x] **1.8** Create command `torrust-tracker-deployer test` to verify deployment infrastructure (UI layer only) ✅ Completed - [Issue #188](https://github.com/torrust/torrust-tracker-deployer/issues/188)
  - **Note:** The App layer TestCommandHandler is already implemented, this task focuses on the console subcommand interface
  - Implementation should call the existing TestCommandHandler business logic
  - Handle user input, validation, and output presentation
  - Enables verification of deployment state via CLI (cloud-init, Docker, Docker Compose)
```

### Evidence of Completion

Both commands are fully implemented:

**Configure Command:**

- CLI definition: `src/presentation/input/cli/commands.rs` (line 56-70)
- Controller: `src/presentation/controllers/configure/`
- Handler: `src/presentation/controllers/configure/handler.rs`
- Errors: `src/presentation/controllers/configure/errors.rs`
- Tests: `src/presentation/controllers/configure/tests/`

**Test Command:**

- CLI definition: `src/presentation/input/cli/commands.rs` (line 72-88)
- Controller: `src/presentation/controllers/test/`
- Handler: `src/presentation/controllers/test/handler.rs`
- Errors: `src/presentation/controllers/test/errors.rs`
- Tests: `src/presentation/controllers/test/tests/`

## Implementation Plan

### Phase 1: Update Roadmap (10 minutes)

- [ ] Open `docs/roadmap.md`
- [ ] Mark task 1.7 as completed (`[x]`, ✅ emoji, "Completed" text)
- [ ] Mark task 1.8 as completed (`[x]`, ✅ emoji, "Completed" text)
- [ ] Verify formatting matches other completed tasks (1.1, 1.2, 1.4, 1.5, 1.6)
- [ ] Commit: `docs: [#XXX] update roadmap to mark configure and test commands as completed`

### Phase 2: Update README.md (15 minutes)

- [ ] Review current "Features" or "Commands" section
- [ ] Update to list all 6 available commands
- [ ] Add workflow example showing command sequence
- [ ] Update any outdated command references
- [ ] Verify markdown linting passes
- [ ] Commit: `docs: [#XXX] update README with complete command workflow`

### Phase 3: Update docs/codebase-architecture.md (15 minutes)

- [ ] Review for command references and examples
- [ ] Update presentation layer examples to show complete command set
- [ ] Update any architecture diagrams if needed
- [ ] Ensure consistency with current implementation
- [ ] Verify markdown linting passes
- [ ] Commit: `docs: [#XXX] update codebase architecture with current commands`

### Phase 4: Update docs/console-commands.md (20 minutes)

- [ ] Review existing command documentation
- [ ] Add comprehensive `configure` command documentation
- [ ] Add comprehensive `test` command documentation
- [ ] Ensure all commands have:
  - [ ] Purpose and description
  - [ ] Command syntax
  - [ ] Arguments and options
  - [ ] Usage examples
  - [ ] State transitions
- [ ] Verify consistency across all command docs
- [ ] Verify markdown linting passes
- [ ] Commit: `docs: [#XXX] add configure and test command documentation`

### Phase 5: Create docs/user-guide/commands/ (45 minutes)

- [ ] Create directory: `docs/user-guide/commands/`
- [ ] Create `README.md` with overview of all commands
- [ ] Create `create-template.md` with detailed guide
- [ ] Create `create-environment.md` with detailed guide
- [ ] Create `provision.md` with detailed guide
- [ ] Create `configure.md` with detailed guide
- [ ] Create `test.md` with detailed guide
- [ ] Create `destroy.md` with detailed guide
- [ ] Each guide should include:
  - [ ] Purpose and use case
  - [ ] Prerequisites
  - [ ] Command syntax
  - [ ] Step-by-step instructions
  - [ ] Examples with output
  - [ ] Troubleshooting
  - [ ] State transitions
- [ ] Verify markdown linting passes for all files
- [ ] Commit: `docs: [#XXX] add comprehensive user guide for all commands`

### Phase 6: Create docs/user-guide/quick-start.md (30 minutes)

- [ ] Create `docs/user-guide/quick-start.md`
- [ ] Add introduction and prerequisites
- [ ] Document Step 1: Generate configuration template
- [ ] Document Step 2: Customize configuration
- [ ] Document Step 3: Create environment
- [ ] Document Step 4: Provision infrastructure
- [ ] Document Step 5: Configure infrastructure
- [ ] Document Step 6: Test deployment
- [ ] Document Step 7: Destroy environment
- [ ] Include actual command examples
- [ ] Add troubleshooting section
- [ ] Add "Next Steps" section
- [ ] Verify markdown linting passes
- [ ] Commit: `docs: [#XXX] add quick-start guide for local deployment`

### Phase 7: Update docs/user-guide/README.md (10 minutes)

- [ ] Update user guide index to link to new documents
- [ ] Add links to `commands/` directory
- [ ] Add link to `quick-start.md`
- [ ] Ensure navigation is clear for end-users
- [ ] Verify markdown linting passes
- [ ] Commit: `docs: [#XXX] update user guide index with new documentation`

### Phase 8: Final Quality Checks (15 minutes)

- [ ] Run `./scripts/pre-commit.sh` to verify all linters pass
- [ ] Check all markdown files for consistency
- [ ] Verify all internal links work
- [ ] Verify all command examples are accurate
- [ ] Review all diffs to ensure only intended changes
- [ ] Test command examples if possible

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All markdown files pass linting (markdownlint)
- [ ] All internal links are functional
- [ ] All command examples are accurate

**Roadmap Updates**:

- [ ] Task 1.7 marked as completed (`[x]`, ✅, "Completed")
- [ ] Task 1.8 marked as completed (`[x]`, ✅, "Completed")
- [ ] Formatting consistent with other completed tasks

**Existing Documentation Updates**:

- [ ] `README.md` lists all 6 available commands
- [ ] `README.md` includes workflow example
- [ ] `docs/codebase-architecture.md` updated with current commands
- [ ] `docs/console-commands.md` includes `configure` command
- [ ] `docs/console-commands.md` includes `test` command
- [ ] All command descriptions are consistent across files

**New Documentation Created**:

- [ ] `docs/user-guide/commands/README.md` exists
- [ ] `docs/user-guide/commands/create-template.md` exists
- [ ] `docs/user-guide/commands/create-environment.md` exists
- [ ] `docs/user-guide/commands/provision.md` exists
- [ ] `docs/user-guide/commands/configure.md` exists
- [ ] `docs/user-guide/commands/test.md` exists
- [ ] `docs/user-guide/commands/destroy.md` exists
- [ ] Each command guide includes:
  - [ ] Purpose and use case
  - [ ] Prerequisites
  - [ ] Command syntax
  - [ ] Examples with output
  - [ ] Troubleshooting
  - [ ] State transitions
- [ ] `docs/user-guide/quick-start.md` exists
- [ ] Quick-start guide covers complete workflow (7 steps)
- [ ] Quick-start guide includes troubleshooting
- [ ] `docs/user-guide/README.md` links to new documentation

**Content Quality**:

- [ ] All commands documented with consistent format
- [ ] Examples are practical and realistic
- [ ] State transitions clearly explained
- [ ] Troubleshooting covers common issues
- [ ] Documentation is user-friendly for end-users
- [ ] Technical accuracy verified against implementation

## Related Documentation

**Project Documentation**:

- [docs/roadmap.md](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md) - Project roadmap
- [README.md](https://github.com/torrust/torrust-tracker-deployer/blob/main/README.md) - Main project README
- [docs/codebase-architecture.md](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/codebase-architecture.md) - Architecture documentation
- [docs/console-commands.md](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/console-commands.md) - Console commands reference
- [docs/user-guide/README.md](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/user-guide/README.md) - User guide index

**Contributing Guides**:

- [docs/contributing/roadmap-issues.md](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/roadmap-issues.md) - Roadmap issues guide
- [docs/contributing/commit-process.md](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/commit-process.md) - Commit conventions
- [docs/contributing/github-markdown-pitfalls.md](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/github-markdown-pitfalls.md) - Markdown best practices

**Implementation Reference**:

- [src/presentation/input/cli/commands.rs](https://github.com/torrust/torrust-tracker-deployer/blob/main/src/presentation/input/cli/commands.rs) - CLI command definitions
- [src/presentation/controllers/](https://github.com/torrust/torrust-tracker-deployer/tree/main/src/presentation/controllers) - All command controllers
- [src/presentation/controllers/create/](https://github.com/torrust/torrust-tracker-deployer/tree/main/src/presentation/controllers/create) - Create command
- [src/presentation/controllers/provision/](https://github.com/torrust/torrust-tracker-deployer/tree/main/src/presentation/controllers/provision) - Provision command
- [src/presentation/controllers/configure/](https://github.com/torrust/torrust-tracker-deployer/tree/main/src/presentation/controllers/configure) - Configure command
- [src/presentation/controllers/test/](https://github.com/torrust/torrust-tracker-deployer/tree/main/src/presentation/controllers/test) - Test command
- [src/presentation/controllers/destroy/](https://github.com/torrust/torrust-tracker-deployer/tree/main/src/presentation/controllers/destroy) - Destroy command

**Related Issues**:

- [Issue #180 - Configure command (UI layer)](https://github.com/torrust/torrust-tracker-deployer/issues/180)
- [Issue #188 - Test command (UI layer)](https://github.com/torrust/torrust-tracker-deployer/issues/188)

## Notes

### Why This Update Is Important

This comprehensive documentation update is crucial because:

1. **Complete Workflow Available**: All core plumbing commands are now implemented, enabling users to perform complete deployment workflows
2. **User Onboarding**: New users need clear documentation to understand how to use the tool
3. **Documentation Accuracy**: Outdated documentation leads to confusion and support burden
4. **Project Maturity**: Complete documentation signals that the core functionality is stable
5. **Contributor Clarity**: Clear documentation helps contributors understand the current state

### Documentation Strategy

This update follows a layered documentation approach:

- **README.md**: High-level overview for quick understanding
- **docs/console-commands.md**: Technical reference for all commands
- **docs/user-guide/commands/**: Detailed guides for each command
- **docs/user-guide/quick-start.md**: Tutorial for complete workflow
- **docs/codebase-architecture.md**: Technical architecture for contributors

### Command Workflow State Transitions

Understanding state transitions is critical for documentation:

```text
[No Environment] --create environment--> [Created]
[Created] --provision--> [Provisioned]
[Provisioned] --configure--> [Configured]
[Configured] --test--> [Verified]
[Any State] --destroy--> [No Environment]
```

### Example Quick-Start Workflow

The quick-start guide should demonstrate:

```bash
# Step 1: Generate template
torrust-tracker-deployer create template my-env.json

# Step 2: Edit my-env.json with your settings

# Step 3: Create environment
torrust-tracker-deployer create environment -f my-env.json

# Step 4: Provision infrastructure
torrust-tracker-deployer provision my-environment

# Step 5: Configure infrastructure
torrust-tracker-deployer configure my-environment

# Step 6: Test deployment
torrust-tracker-deployer test my-environment

# Step 7: Cleanup
torrust-tracker-deployer destroy my-environment
```

### Consistency Requirements

When updating documentation:

- Use consistent terminology across all files
- Use the same command examples where possible
- Maintain consistent formatting and structure
- Cross-reference related documentation
- Keep technical depth appropriate for the audience

## Estimated Time

~2.5 hours total (comprehensive documentation update across multiple files and creation of new guides)
