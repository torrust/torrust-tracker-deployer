# Refactor Controllers: Remove Intermediary Layer and Use DI Container

**Issue**: #196
**Parent Epic**: #X - [Refactoring]
**Related**:

## Overview

Refactor the presentation layer controllers to remove the functional wrapper (`handle_X_command`) and instantiate controllers directly. Furthermore, move the instantiation logic to the Dependency Injection container (`src/bootstrap/container.rs`) to decouple the `main.rs` and `handler.rs` from manual dependency wiring.

## Goals

- [ ] Remove `handle_X_command` functions in all controllers.
- [ ] Instantiate controllers using `Controller::new(...)` pattern.
- [ ] Move controller factory methods to `Container` struct.
- [ ] Update `ExecutionContext` or call sites to retrieve controllers from the container.

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/controllers/`
**Pattern**: Controller

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] Respect dependency flow rules (dependencies flow toward domain)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../docs/contributing/module-organization.md))

### Architectural Constraints

- [ ] No business logic in presentation layer
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Testing strategy aligns with layer responsibilities

### Anti-Patterns to Avoid

- ❌ Mixing concerns across layers
- ❌ Domain layer depending on infrastructure
- ❌ Monolithic modules with multiple responsibilities

## Specifications

### 1. Remove Intermediary Layer

The current implementation uses a functional wrapper `handle_X_command` that instantiates the controller and calls `execute`. This should be replaced by direct instantiation of the controller struct.

**Before:**

```rust
pub async fn handle_create_command(...) -> Result<()> {
    CreateCommandController::new(...).execute(...).await
}
```

**After:**

```rust
// In main.rs or dispatch
let controller = container.create_create_command_controller();
controller.execute(...).await?;
```

### 2. Move Instantiation to Container

The `Container` struct in `src/bootstrap/container.rs` should be responsible for creating controller instances, injecting the necessary dependencies (repository, user_output, etc.).

```rust
impl Container {
    pub fn create_create_command_controller(&self) -> CreateCommandController {
        CreateCommandController::new(
            self.repository.clone(),
            self.user_output.clone(),
        )
    }
}
```

## Implementation Plan

- [ ] Refactor `create` controller.
- [ ] Refactor `provision` controller.
- [ ] Refactor `destroy` controller.
- [ ] Refactor `configure` controller.
- [ ] Refactor `test` controller.
- [ ] Update `Container` to include factory methods.
- [ ] Update `main.rs` / `router.rs` to use `Container` for controller creation.

## Acceptance Criteria

- [ ] All `handle_X_command` functions are removed.
- [ ] Controllers are instantiated via `Container`.
- [ ] All tests pass.
- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
