# Refactor ConfigureCommandHandler to Use EnvironmentName Instead of Environment<Provisioned>

**Issue**: #178
**Parent Epic**: #TBD - [Implement Configure Console Subcommand]
**Related**:

- [Issue #174](https://github.com/torrust/torrust-tracker-deployer/issues/174) - Provision Console Subcommand
- [docs/codebase-architecture.md](../codebase-architecture.md) - DDD Architecture
- [docs/contributing/ddd-layer-placement.md](../contributing/ddd-layer-placement.md) - Layer Placement Guide

## Overview

Refactor the `ConfigureCommandHandler` in the Application layer to accept `EnvironmentName` instead of `Environment<Provisioned>` as its input parameter. This change aligns the configure command handler with the provision command handler's interface pattern, simplifying the implementation of presentation layer controllers by reducing the number of Application layer dependencies that need to be constructed.

Currently, `ConfigureCommandHandler::execute` accepts `Environment<Provisioned>`, which requires the presentation layer controller to:

1. Load the environment from the repository
2. Verify it's in the correct state (`Provisioned`)
3. Pass the fully-typed environment to the handler

The `ProvisionCommandHandler` already follows the simpler pattern of accepting just `EnvironmentName` and handling environment loading and state verification internally. This refactor brings consistency to command handler interfaces and prepares for implementing the configure console subcommand.

## Goals

- [ ] Update `ConfigureCommandHandler::execute` signature to accept `&EnvironmentName` instead of `Environment<Provisioned>`
- [ ] Move environment loading logic from presentation layer to `ConfigureCommandHandler`
- [ ] Move state verification logic from presentation layer to `ConfigureCommandHandler`
- [ ] Ensure error handling remains comprehensive and user-friendly
- [ ] Maintain backward compatibility with existing tests
- [ ] Update all integration and E2E tests to use new signature

## 🏗️ Architecture Requirements

**DDD Layer**: Application
**Module Path**: `src/application/command_handlers/configure/`
**Pattern**: Command Handler

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Respect dependency flow rules (Application layer can depend on Domain and Infrastructure)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Command handler must handle environment loading and state verification internally
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Must return explicit enum errors (not `anyhow::Error`) for better pattern matching
- [ ] Errors must be actionable with specific fix instructions
- [ ] Maintain type-state pattern for environment lifecycle management
- [ ] Repository operations should be non-failing (log persistence errors but don't fail command)

### Anti-Patterns to Avoid

- ❌ Passing complex Application layer types through presentation layer
- ❌ Forcing presentation layer to handle Application layer concerns (state verification, loading)
- ❌ Using `anyhow::Error` for domain errors
- ❌ Failing command execution on repository persistence errors (state should remain valid in memory)

## Specifications

### Current Signature (Before Refactor)

```rust
// src/application/command_handlers/configure/handler.rs
impl ConfigureCommandHandler {
    pub fn execute(
        &self,
        environment: Environment<Provisioned>,
    ) -> Result<Environment<Configured>, ConfigureCommandHandlerError> {
        // ... implementation
    }
}
```

**Issues with current approach:**

1. Presentation layer must construct `Environment<Provisioned>` from repository
2. Presentation layer must verify environment state before calling handler
3. Presentation layer needs access to typed repository (`TypedEnvironmentRepository`)
4. Increases coupling between presentation and application layers

### Target Signature (After Refactor)

```rust
// src/application/command_handlers/configure/handler.rs
impl ConfigureCommandHandler {
    pub fn execute(
        &self,
        env_name: &EnvironmentName,
    ) -> Result<Environment<Configured>, ConfigureCommandHandlerError> {
        // 1. Load environment from repository (returns AnyEnvironmentState)
        // 2. Verify environment exists
        // 3. Verify environment is in Provisioned state (downcast)
        // 4. Execute configuration workflow
        // 5. Return configured environment
    }
}
```

**Benefits of new approach:**

1. Presentation layer only needs to pass `EnvironmentName` (simple value object)
2. Application layer handles all environment loading and state verification
3. Consistent interface with `ProvisionCommandHandler`
4. Simplifies future presentation layer controller implementation
5. Reduces dependencies presentation layer needs to manage

### Error Handling

Update `ConfigureCommandHandlerError` to include new error variants for environment loading:

```rust
// src/application/command_handlers/configure/errors.rs
#[derive(Debug, Error)]
pub enum ConfigureCommandHandlerError {
    #[error("Environment not found")]
    EnvironmentNotFound,

    #[error("Environment is in invalid state for configuration. Expected: Provisioned, Found: {found}")]
    InvalidEnvironmentState { found: String },

    #[error("Failed to load environment from repository")]
    StatePersistence(#[from] RepositoryError),

    // ... existing error variants
}
```

### Implementation Pattern Reference

Follow the same pattern used in `ProvisionCommandHandler::execute`:

```rust
// Reference: src/application/command_handlers/provision/handler.rs (lines 95-120)
pub async fn execute(
    &self,
    env_name: &EnvironmentName,
) -> Result<Environment<Provisioned>, ProvisionCommandHandlerError> {
    // 1. Load the environment from storage (returns AnyEnvironmentState - type-erased)
    let any_env = self
        .repository
        .inner()
        .load(env_name)
        .map_err(ProvisionCommandHandlerError::StatePersistence)?;

    // 2. Check if environment exists
    let any_env = any_env.ok_or_else(|| {
        ProvisionCommandHandlerError::StatePersistence(RepositoryError::NotFound)
    })?;

    // 3. Try to downcast to Created state
    let environment = any_env
        .downcast::<Created>()
        .ok_or_else(|| ProvisionCommandHandlerError::InvalidState {
            expected: "Created".to_string(),
            found: any_env.state_name().to_string(),
        })?;

    // 4. Continue with provisioning workflow...
}
```

## Implementation Plan

### Phase 1: Update Command Handler Signature (estimated: 30 minutes)

- [ ] Update `ConfigureCommandHandler::execute` signature to accept `&EnvironmentName`
- [ ] Add environment loading logic at the start of `execute` method
- [ ] Add state verification logic (downcast to `Provisioned` state)
- [ ] Update error handling to include new error variants
- [ ] Ensure instrumentation and logging reflect new signature

### Phase 2: Update Error Types (estimated: 15 minutes)

- [ ] Add `EnvironmentNotFound` error variant to `ConfigureCommandHandlerError`
- [ ] Add `InvalidEnvironmentState` error variant with detailed state information
- [ ] Ensure error messages are actionable and user-friendly
- [ ] Add context to errors for traceability

### Phase 3: Update Unit Tests (estimated: 30 minutes)

- [ ] Update unit tests in `src/application/command_handlers/configure/handler.rs`
- [ ] Add test cases for environment not found scenario
- [ ] Add test cases for invalid environment state (e.g., trying to configure an already-configured environment)
- [ ] Verify error messages are clear and actionable

### Phase 4: Update Integration Tests (estimated: 30 minutes)

- [ ] Update E2E tests in `tests/` directory that use `ConfigureCommandHandler`
- [ ] Update test fixtures if needed
- [ ] Verify all tests pass with new signature

### Phase 5: Update Documentation (estimated: 15 minutes)

- [ ] Update command handler documentation comments
- [ ] Update any relevant guides in `docs/` directory
- [ ] Update examples if they reference the old signature

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
  - [ ] No unused dependencies (`cargo machete`)
  - [ ] All linters pass (markdown, yaml, toml, clippy, rustfmt, shellcheck)
  - [ ] All unit tests pass (`cargo test`)
  - [ ] Documentation builds successfully (`cargo doc`)
  - [ ] All E2E tests pass

**Functional Requirements**:

- [ ] `ConfigureCommandHandler::execute` accepts `&EnvironmentName` as parameter
- [ ] Environment loading is handled internally by the command handler
- [ ] State verification (Provisioned state) is handled internally
- [ ] Appropriate errors are returned for:
  - [ ] Environment not found
  - [ ] Environment in wrong state (not Provisioned)
  - [ ] Repository access failures
- [ ] Error messages are clear, actionable, and follow project conventions
- [ ] Type-state pattern is preserved (Environment<Provisioned> → Environment<Configured>)
- [ ] Repository persistence failures are logged but don't fail the command

**Code Quality**:

- [ ] Follows the pattern established by `ProvisionCommandHandler`
- [ ] Maintains separation of concerns (Application vs Presentation layer)
- [ ] Error handling uses explicit enum types (not `anyhow::Error`)
- [ ] Code is well-documented with clear examples
- [ ] Instrumentation (`tracing`) is properly configured
- [ ] Module organization follows project conventions

**Testing**:

- [ ] All existing unit tests updated and passing
- [ ] New test cases added for error scenarios:
  - [ ] Environment not found
  - [ ] Invalid environment state
- [ ] Integration tests updated and passing
- [ ] E2E tests updated and passing
- [ ] Test coverage maintained or improved

**Documentation**:

- [ ] Command handler documentation updated
- [ ] Method documentation includes examples
- [ ] Error variant documentation is clear and actionable
- [ ] Architecture documentation updated if needed

## Related Documentation

- [Codebase Architecture](../codebase-architecture.md) - DDD layers and dependencies
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md) - Which code belongs where
- [Error Handling Guide](../contributing/error-handling.md) - Error handling principles
- [Module Organization](../contributing/module-organization.md) - Code organization patterns
- [Provision Console Subcommand - Issue #174](https://github.com/torrust/torrust-tracker-deployer/issues/174) - Related presentation layer work
- [Provision Command Handler](../../src/application/command_handlers/provision/handler.rs) - Reference implementation

## Notes

### Why This Refactor is Necessary

This refactor is a **preparatory step** before implementing the configure console subcommand in the presentation layer. Without this change, the presentation layer controller would need to:

1. Inject `TypedEnvironmentRepository` into the controller
2. Load the environment and handle type-erased `AnyEnvironmentState`
3. Downcast to `Environment<Provisioned>` and handle errors
4. Construct and inject all the services that `ConfigureCommandHandler` needs

This creates unnecessary complexity and coupling. By moving environment loading into the command handler, the presentation layer controller can simply:

```rust
// Simplified presentation layer (future implementation)
pub fn handle_configure_subcommand(
    env_name: &EnvironmentName,
    context: &ExecutionContext,
) -> Result<Environment<Configured>, ConfigureSubcommandError> {
    let handler = context.container.configure_command_handler();
    handler.execute(env_name)?
}
```

### Consistency Across Command Handlers

After this refactor, both major command handlers will have consistent interfaces:

- `ProvisionCommandHandler::execute(&self, env_name: &EnvironmentName)` ✅ Already implemented
- `ConfigureCommandHandler::execute(&self, env_name: &EnvironmentName)` ⬅️ This refactor

This consistency makes the codebase more maintainable and predictable.

### Non-Breaking Change Strategy

This is an internal refactor that only affects:

- The Application layer (`ConfigureCommandHandler`)
- Test code that uses the handler

No external APIs or user-facing features are affected. The change is backward-compatible from a user perspective.
