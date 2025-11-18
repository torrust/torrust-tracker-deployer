# Implement Configure CLI Command

**Issue**: #180
**Parent Epic**: #2 - [Scaffolding for main app](https://github.com/torrust/torrust-tracker-deployer/issues/2)
**Related**: [Roadmap Task 1.7](../roadmap.md), [ConfigureCommandHandler](../../src/application/command_handlers/configure/)

## Overview

Implement the CLI presentation layer for the `configure` command, exposing the existing `ConfigureCommandHandler` application layer logic to end users. This enables users to transition deployment environments from "Provisioned" state to "Configured" state via the CLI interface.

## Goals

- [ ] Expose `ConfigureCommandHandler` through a CLI subcommand
- [ ] Follow the same architectural pattern as the `provision` command
- [ ] Provide user-friendly progress reporting during configuration
- [ ] Return actionable error messages with help guidance
- [ ] Enable full deployment workflow testing via public CLI interface

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/controllers/configure/`
**Pattern**: CLI Subcommand Controller

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Match structure of `provision` controller: `mod.rs`, `errors.rs`, `handler.rs`, `tests/`
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] No business logic in presentation layer (delegate to `ConfigureCommandHandler`)
- [ ] Use `ExecutionContext` pattern for dependency injection
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] All errors implement `.help()` with actionable guidance

### Anti-Patterns to Avoid

- ❌ Implementing configuration logic in presentation layer
- ❌ Direct infrastructure calls (use application layer)
- ❌ Generic error messages without help guidance

## Specifications

### Current State

**Application Layer** (already implemented):

- `ConfigureCommandHandler` in `src/application/command_handlers/configure/`
- Handles Docker and Docker Compose installation
- Implements state transitions: `Provisioned` → `Configuring` → `Configured`
- Fully tested and working

**Missing Presentation Layer**:

- CLI command definition in `Commands` enum
- Presentation controller in `src/presentation/controllers/configure/`
- Router integration
- Error types with `.help()` methods

### CLI Command Definition

Location: `src/presentation/input/cli/commands.rs`

```rust
/// Configure a provisioned deployment environment
///
/// This command configures the infrastructure of a provisioned deployment
/// environment. It will:
/// - Install Docker engine
/// - Install Docker Compose
/// - Configure system services
///
/// The environment must be in "Provisioned" state (use 'provision' command first).
Configure {
    /// Name of the environment to configure
    ///
    /// The environment name must match an existing environment that was
    /// previously provisioned and is in "Provisioned" state.
    environment: String,
},
```

### Presentation Controller Structure

Create `src/presentation/controllers/configure/` with:

**Files**:

1. `mod.rs` - Module documentation and exports
2. `errors.rs` - `ConfigureSubcommandError` with `.help()` methods
3. `handler.rs` - `handle()`, `handle_configure_command()`, controller implementation
4. `tests/mod.rs` - Integration tests

**Reference**: Follow exact pattern from `src/presentation/controllers/provision/`

### Error Types

Location: `src/presentation/controllers/configure/errors.rs`

```rust
#[derive(Debug, Error)]
pub enum ConfigureSubcommandError {
    #[error("Invalid environment name: {0}")]
    InvalidEnvironmentName(#[from] EnvironmentNameError),

    #[error("Repository error: {0}")]
    RepositoryError(#[from] EnvironmentRepositoryError),

    #[error("Configuration failed: {0}")]
    ConfigureHandlerError(#[from] ConfigureCommandHandlerError),
}

impl ConfigureSubcommandError {
    pub fn help(&self) -> String {
        // Actionable help messages for each error variant
    }
}
```

### Router Integration

Location: `src/presentation/dispatch/router.rs`

```rust
Commands::Configure { environment } => {
    configure::handle(&environment, working_dir, context).await?;
}
```

Location: `src/presentation/errors.rs`

```rust
#[error("Configure command failed: {0}")]
Configure(Box<ConfigureSubcommandError>),
```

## Implementation Plan

### Phase 1: CLI Command Definition (15 min)

- [ ] Add `Configure` variant to `Commands` enum in `src/presentation/input/cli/commands.rs`
- [ ] Add comprehensive documentation following provision pattern
- [ ] Verify CLI parsing with `--help`

**Files Modified**: `src/presentation/input/cli/commands.rs`

### Phase 2: Presentation Controller (1-2 hours)

- [ ] Create `src/presentation/controllers/configure/` directory
- [ ] Implement `mod.rs` with module documentation and exports
- [ ] Implement `errors.rs` with `ConfigureSubcommandError` and `.help()` methods
- [ ] Implement `handler.rs`:
  - [ ] `handle()` function (ExecutionContext-based API)
  - [ ] `handle_configure_command()` function (direct DI API)
  - [ ] `ConfigureCommandController` struct with progress reporting
  - [ ] State validation (must be in "Provisioned" state)
  - [ ] Integration with `ConfigureCommandHandler`

**Files Created**:

- `src/presentation/controllers/configure/mod.rs`
- `src/presentation/controllers/configure/errors.rs`
- `src/presentation/controllers/configure/handler.rs`

### Phase 3: Integration (30 min)

- [ ] Add configure controller to router in `src/presentation/dispatch/router.rs`
- [ ] Add configure error variant to `CommandError` in `src/presentation/errors.rs`
- [ ] Update `src/presentation/controllers/mod.rs` to export configure module
- [ ] Verify compilation and module visibility

**Files Modified**:

- `src/presentation/dispatch/router.rs`
- `src/presentation/errors.rs`
- `src/presentation/controllers/mod.rs`

### Phase 4: Testing (1-2 hours)

- [ ] Create `src/presentation/controllers/configure/tests/mod.rs`
- [ ] Add integration tests:
  - [ ] Successful configuration flow
  - [ ] Invalid environment name handling
  - [ ] Environment not found handling
  - [ ] Invalid state handling (not provisioned)
  - [ ] Configuration failure handling
- [ ] Verify progress reporting output
- [ ] Test error messages and help output

**Files Created**: `src/presentation/controllers/configure/tests/mod.rs`

### Phase 5: Manual End-to-End Testing (30 min)

Test the complete deployment workflow using only the public CLI interface:

```bash
# 1. Generate template
torrust-tracker-deployer create template test-config.json

# 2. Edit configuration (use testing keys from fixtures/)
vim test-config.json

# 3. Create environment
torrust-tracker-deployer create environment -f test-config.json

# 4. Provision infrastructure
torrust-tracker-deployer provision test-env

# 5. Configure environment (NEW COMMAND)
torrust-tracker-deployer configure test-env

# 6. Verify via SSH
ssh -i fixtures/testing_rsa ubuntu@<ip> docker --version
ssh -i fixtures/testing_rsa ubuntu@<ip> docker compose version

# 7. Cleanup
torrust-tracker-deployer destroy test-env
```

**Verify**:

- [ ] All commands execute successfully
- [ ] Progress reporting is clear and helpful
- [ ] Docker and Docker Compose are installed
- [ ] State transitions work correctly
- [ ] Error messages are actionable

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Functional Requirements**:

- [ ] `torrust-tracker-deployer configure <name>` command is available
- [ ] Command validates environment is in "Provisioned" state
- [ ] Configuration workflow executes (Docker, Docker Compose installation)
- [ ] Environment transitions to "Configured" state on success
- [ ] Environment transitions to "ConfigureFailed" state on error
- [ ] State changes are persisted to repository

**User Experience**:

- [ ] Progress reporting shows clear step-by-step feedback
- [ ] Error messages include actionable help via `.help()` method
- [ ] Command appears in main CLI `--help` output
- [ ] Command has comprehensive documentation in `--help configure`
- [ ] Success message indicates next steps

**Code Quality**:

- [ ] Follows existing presentation layer patterns (matches provision controller)
- [ ] Module organization matches provision command structure
- [ ] Documentation is comprehensive with examples
- [ ] Error types implement Display and `.help()` traits
- [ ] No clippy warnings or rustfmt issues

**Testing**:

- [ ] Unit tests pass
- [ ] Integration tests cover happy path and error cases
- [ ] Manual E2E test confirms full deployment workflow works

**Architecture Compliance**:

- [ ] Presentation layer only (no business logic)
- [ ] Uses `ExecutionContext` pattern
- [ ] Integrates with existing `ConfigureCommandHandler`
- [ ] Error handling provides traceability and actionability

## Related Documentation

- [Roadmap](../roadmap.md) - Task 1.7
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Module Organization](../contributing/module-organization.md)
- [Codebase Architecture](../codebase-architecture.md)
- [Provision Controller Reference](../../src/presentation/controllers/provision/)
- [Configure Command Handler](../../src/application/command_handlers/configure/)

## Notes

### Implementation Approach

This is a straightforward presentation layer task:

- **No new business logic** - All configuration logic exists in `ConfigureCommandHandler`
- **Follow existing pattern** - Copy structure from `provision` controller
- **Focus on UX** - Clear progress reporting and actionable error messages
- **Enable E2E workflow** - Users can now test full deployment: create → provision → configure

### Estimated Time

Total: 4-6 hours

- Phase 1: 15 min
- Phase 2: 1-2 hours
- Phase 3: 30 min
- Phase 4: 1-2 hours
- Phase 5: 30 min

### Questions/Clarifications

None at this time. The implementation path is clear:

1. Copy the provision controller pattern
2. Adapt it for configure command
3. Wire it into the CLI routing
4. Test the complete workflow
