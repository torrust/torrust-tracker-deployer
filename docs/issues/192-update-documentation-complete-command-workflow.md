# Update Documentation: Complete Command Workflow Available

**Issue**: [#192](https://github.com/torrust/torrust-tracker-deployer/issues/192)
**Parent Epic**: [#2 - Scaffolding for main app](https://github.com/torrust/torrust-tracker-deployer/issues/2)
**Related**:

- [Issue #180 - Configure command (UI layer)](https://github.com/torrust/torrust-tracker-deployer/issues/180)
- [Issue #188 - Test command (UI layer)](https://github.com/torrust/torrust-tracker-deployer/issues/188)

## Overview

Update all project documentation to reflect that the complete command workflow is now available to end-users. With the recent completion of the `configure` and `test` commands, users can now execute the full deployment lifecycle from environment creation to verification. This task involves updating existing documentation and creating new user-facing guides.

## Goals

- [x] Update roadmap to mark tasks 1.7 and 1.8 as completed ✅
- [x] Update existing documentation to reflect all available commands ✅
- [x] Create comprehensive user guide for individual commands ✅
- [x] Create quick-start guide showing complete deployment workflow ✅
- [x] Ensure documentation consistency across all files ✅

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

### Phase 1: Update Roadmap ✅ Completed

- [x] Open `docs/roadmap.md`
- [x] Mark task 1.7 as completed (`[x]`, ✅ emoji, "Completed" text)
- [x] Mark task 1.8 as completed (`[x]`, ✅ emoji, "Completed" text)
- [x] Verify formatting matches other completed tasks (1.1, 1.2, 1.4, 1.5, 1.6)
- [x] Commit: `docs: [#192] update roadmap to mark configure and test commands as completed`

### Phase 2: Update README.md ✅ Completed

- [x] Review current "Features" or "Commands" section
- [x] Update to list all 6 available commands
- [x] Add workflow example showing command sequence
- [x] Update any outdated command references
- [x] Verify markdown linting passes
- [x] Commit: Initial commit included README updates

### Phase 3: Update docs/codebase-architecture.md ✅ Completed

- [x] Review for command references and examples
- [x] Update presentation layer examples to show complete command set
- [x] Update any architecture diagrams if needed
- [x] Ensure consistency with current implementation
- [x] Verify markdown linting passes
- [x] Commit: `docs: [#192] update codebase architecture documentation`

### Phase 4: Update docs/console-commands.md ✅ Completed

- [x] Review existing command documentation
- [x] Add comprehensive `configure` command documentation
- [x] Add comprehensive `test` command documentation
- [x] Ensure all commands have:
  - [x] Purpose and description
  - [x] Command syntax
  - [x] Arguments and options
  - [x] Usage examples
  - [x] State transitions
- [x] Verify consistency across all command docs
- [x] Verify markdown linting passes
- [x] Commit: `docs: [#192] update console commands reference`

### Phase 5: Create docs/user-guide/commands/ ✅ Completed

> **Note**: Created one file per first-level command (not per subcommand), consolidating `create template` and `create environment` into single `create.md` file.

- [x] Create directory: `docs/user-guide/commands/`
- [x] Create `README.md` with overview of all commands
- [x] Create `create.md` with detailed guide (covers both template and environment subcommands)
- [x] Create `provision.md` with detailed guide
- [x] Create `configure.md` with detailed guide
- [x] Create `test.md` with detailed guide
- [x] Use existing `destroy.md` (already present)
- [x] Each guide includes:
  - [x] Purpose and use case
  - [x] Prerequisites
  - [x] Command syntax
  - [x] Step-by-step instructions
  - [x] Examples with output
  - [x] Troubleshooting
  - [x] State transitions
- [x] Verify markdown linting passes for all files
- [x] Commit: `docs: [#192] Create user guide for provision, configure, and test commands`

### Phase 6: Create docs/user-guide/quick-start.md ✅ Completed

> **Important**: Fixed documentation to use config files **outside** `data/` folder (user config vs internal app data). Updated all examples to use proper JSON structure and correct CLI flags (`--env-file` not `--from-file`).

- [x] Create `docs/user-guide/quick-start.md`
- [x] Add introduction and prerequisites
- [x] Document Step 1: Generate configuration template
- [x] Document Step 2: Customize configuration
- [x] Document Step 3: Create environment
- [x] Document Step 4: Provision infrastructure
- [x] Document Step 5: Configure infrastructure
- [x] Document Step 6: Test deployment
- [x] Document Step 7: Destroy environment
- [x] Include actual command examples (verified end-to-end)
- [x] Add troubleshooting section
- [x] Add "Next Steps" section
- [x] Verify markdown linting passes
- [x] Commit: `docs: [#192] Create quick-start guide with correct config file paths`

### Phase 7: Update docs/user-guide/README.md ✅ Completed

- [x] Update user guide index to link to new documents
- [x] Add links to `commands/` directory
- [x] Add link to `quick-start.md`
- [x] Ensure navigation is clear for end-users
- [x] Verify markdown linting passes
- [x] Commit: Will be included in final commit

### Phase 8: Final Quality Checks ✅ Completed

- [x] Run `./scripts/pre-commit.sh` to verify all linters pass
- [x] Check all markdown files for consistency
- [x] Verify all internal links work
- [x] Verify all command examples are accurate
- [x] Review all diffs to ensure only intended changes
- [x] Test command examples end-to-end (workflow verified successfully)

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [x] Pre-commit checks pass: `./scripts/pre-commit.sh` ✅
- [x] All markdown files pass linting (markdownlint) ✅
- [x] All internal links are functional ✅
- [x] All command examples are accurate ✅

**Roadmap Updates**:

- [x] Task 1.7 marked as completed (`[x]`, ✅, "Completed") ✅
- [x] Task 1.8 marked as completed (`[x]`, ✅, "Completed") ✅
- [x] Formatting consistent with other completed tasks ✅

**Existing Documentation Updates**:

- [x] `README.md` lists all 6 available commands ✅
- [x] `README.md` includes workflow example ✅
- [x] `docs/codebase-architecture.md` updated with current commands ✅
- [x] `docs/console-commands.md` includes `configure` command ✅
- [x] `docs/console-commands.md` includes `test` command ✅
- [x] All command descriptions are consistent across files ✅

**New Documentation Created**:

- [x] `docs/user-guide/commands/README.md` exists ✅
- [x] `docs/user-guide/commands/create.md` exists (consolidated template + environment) ✅
- [x] `docs/user-guide/commands/provision.md` exists ✅
- [x] `docs/user-guide/commands/configure.md` exists ✅
- [x] `docs/user-guide/commands/test.md` exists ✅
- [x] `docs/user-guide/commands/destroy.md` exists (was already present) ✅
- [x] Each command guide includes:
  - [x] Purpose and use case ✅
  - [x] Prerequisites ✅
  - [x] Command syntax ✅
  - [x] Examples with output ✅
  - [x] Troubleshooting ✅
  - [x] State transitions ✅
- [x] `docs/user-guide/quick-start.md` exists ✅
- [x] Quick-start guide covers complete workflow (7 steps) ✅
- [x] Quick-start guide includes troubleshooting ✅
- [x] `docs/user-guide/README.md` links to new documentation ✅

**Content Quality**:

- [x] All commands documented with consistent format ✅
- [x] Examples are practical and realistic (using test SSH keys from fixtures/) ✅
- [x] State transitions clearly explained ✅
- [x] Troubleshooting covers common issues ✅
- [x] Documentation is user-friendly for end-users ✅
- [x] Technical accuracy verified against implementation (workflow tested end-to-end) ✅

**Additional Achievements**:

- [x] Fixed config file path strategy (user config files outside `data/` folder) ✅
- [x] Corrected CLI flag documentation (`--env-file` not `--from-file`) ✅
- [x] Verified complete workflow end-to-end ✅
- [x] Added proper JSON structure examples with correct field names ✅

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
