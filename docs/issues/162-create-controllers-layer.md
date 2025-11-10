# Create Controllers Layer

**Issue**: #162  
**Parent Epic**: #154 - Presentation Layer Reorganization  
**Related**: [Presentation Layer Reorganization Refactor Plan](../refactors/plans/presentation-layer-reorganization.md#proposal-3-create-controllers-layer)

## Overview

Transform the `src/presentation/commands/` directory into a Controllers Layer using standard MVC terminology and integrate it fully with the existing ExecutionContext and Container for dependency injection. This proposal establishes the third layer of the four-layer presentation architecture.

## Goals

- [ ] Rename `commands/` to `controllers/` with standard terminology
- [ ] Integrate controllers with ExecutionContext for dependency access
- [ ] Remove factory.rs pattern in favor of Container dependency injection
- [ ] Update function signatures to use ExecutionContext instead of direct UserOutput
- [ ] Simplify handler function names to follow controller patterns
- [ ] Maintain full test coverage during the transition

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation  
**Module Path**: `src/presentation/controllers/`  
**Pattern**: Controller Layer (MVC pattern)

### Module Structure Requirements

- [ ] Follow controller naming conventions (controllers/, not commands/)
- [ ] Each controller module manages one command domain (create/, destroy/)
- [ ] Use ExecutionContext for all service dependencies
- [ ] Handler functions follow simplified naming (`handle` instead of `handle_create_command`)

### Architectural Constraints

- [ ] Controllers must not contain business logic (delegate to application layer)
- [ ] All dependencies accessed through ExecutionContext wrapper
- [ ] Error handling follows project conventions with CommandError types
- [ ] Router integration through existing dispatch layer

### Anti-Patterns to Avoid

- ❌ Direct service instantiation in controllers
- ❌ Business logic in presentation layer
- ❌ Factory pattern duplication of Container
- ❌ Inconsistent naming between controller modules

## Specifications

### Current State Analysis

**Existing Structure**:

```text
src/presentation/commands/
├── constants.rs
├── context.rs          # Will be deprecated/removed
├── create/
│   ├── handler.rs
│   ├── mod.rs
│   ├── errors.rs
│   └── subcommands/
├── destroy/
│   ├── handler.rs
│   ├── mod.rs
│   └── errors.rs
├── factory.rs          # Will be removed
├── mod.rs
└── tests/
```

**Target Structure**:

```text
src/presentation/controllers/
├── constants.rs        # Moved from commands/
├── create/
│   ├── handler.rs      # Updated to use ExecutionContext
│   ├── mod.rs
│   ├── errors.rs
│   └── subcommands/
├── destroy/
│   ├── handler.rs      # Updated to use ExecutionContext
│   ├── mod.rs
│   └── errors.rs
├── mod.rs              # Updated documentation and exports
└── tests/             # Moved and updated
```

**Target Structure**:

```text
src/presentation/controllers/
```

### ExecutionContext Integration

**Current Handler Signatures**:

```rust
// create/handler.rs
pub fn handle_create_command(
    action: CreateAction,
    working_dir: &Path,
    user_output: &Arc<Mutex<UserOutput>>,
) -> Result<(), CreateSubcommandError>

// destroy/handler.rs
pub fn handle_destroy_command(
    environment: &str,
    working_dir: &Path,
    user_output: &Arc<Mutex<UserOutput>>,
) -> Result<(), DestroySubcommandError>
```

**Target Handler Signatures**:

```rust
// create/handler.rs
pub fn handle(
    action: CreateAction,
    working_dir: &Path,
    context: &ExecutionContext,
) -> Result<(), CreateSubcommandError>

// destroy/handler.rs
pub fn handle(
    environment: &str,
    working_dir: &Path,
    context: &ExecutionContext,
) -> Result<(), DestroySubcommandError>
```

### Router Integration

**Current Router Calls**:

```rust
// dispatch/router.rs
Commands::Create { action } => {
    create::handle_create_command(action, working_dir, &context.user_output())?;
    Ok(())
}
Commands::Destroy { environment } => {
    destroy::handle_destroy_command(&environment, working_dir, &context.user_output())?;
    Ok(())
}
```

**Target Router Calls**:

```rust
// dispatch/router.rs
use crate::presentation::controllers::{create, destroy};

Commands::Create { action } => {
    create::handle(action, working_dir, context)?;
    Ok(())
}
Commands::Destroy { environment } => {
    destroy::handle(&environment, working_dir, context)?;
    Ok(())
}
```

## Implementation Plan

### Phase 1: Preparation and Structure (30 min)

- [ ] Create `src/presentation/controllers/` directory
- [ ] Copy entire `commands/` directory contents to `controllers/`
- [ ] Update `src/presentation/mod.rs` to declare `controllers` module
- [ ] Verify all tests still pass with duplicated structure

### Phase 2: Handler Signature Updates (45 min)

- [ ] Update `controllers/create/handler.rs`:
  - [ ] Change function name from `handle_create_command` to `handle`
  - [ ] Replace `user_output: &Arc<Mutex<UserOutput>>` with `context: &ExecutionContext`
  - [ ] Update function body to use `context.user_output()` instead of direct `user_output`
- [ ] Update `controllers/destroy/handler.rs`:
  - [ ] Change function name from `handle_destroy_command` to `handle`
  - [ ] Replace `user_output: &Arc<Mutex<UserOutput>>` with `context: &ExecutionContext`
  - [ ] Update function body to use `context.user_output()` instead of direct `user_output`
- [ ] Update subcommand handlers to receive and pass through ExecutionContext

### Phase 3: Router Integration (30 min)

- [ ] Update `src/presentation/dispatch/router.rs`:
  - [ ] Change import from `crate::presentation::commands` to `crate::presentation::controllers`
  - [ ] Update function calls to use new `handle` names
  - [ ] Pass full `context` instead of `context.user_output()`
- [ ] Verify router compiles and routes correctly

### Phase 4: Module Documentation and Exports (30 min)

- [ ] Update `controllers/mod.rs`:
  - [ ] Update module documentation to reflect controller terminology
  - [ ] Remove references to old commands naming
  - [ ] Update re-exports to match new handler function names
- [ ] Update `src/presentation/mod.rs`:
  - [ ] Add documentation explaining controllers layer
  - [ ] Update re-exports from commands to controllers

### Phase 5: Remove Factory Pattern (30 min)

- [ ] Remove `controllers/factory.rs` (duplicate of Container pattern)
- [ ] Remove `controllers/context.rs` (superseded by dispatch/context.rs)
- [ ] Remove any imports or references to factory pattern
- [ ] Ensure no circular dependencies or unused code remains

### Phase 6: Test Updates and Cleanup (45 min)

- [ ] Update test files in `controllers/tests/`:
  - [ ] Update imports to use new controller module paths
  - [ ] Update test helper functions to use ExecutionContext
  - [ ] Verify all tests pass with new structure
- [ ] Remove old `commands/` directory completely
- [ ] Update any remaining imports across the codebase
- [ ] Run full test suite to ensure no regressions

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Structural Changes**:

- [ ] `src/presentation/commands/` directory no longer exists
- [ ] `src/presentation/controllers/` directory exists with all command handlers
- [ ] `controllers/factory.rs` has been removed
- [ ] All controller handlers use ExecutionContext instead of direct UserOutput
- [ ] Handler functions use simplified naming (`handle` instead of `handle_create_command`)

**Integration Verification**:

- [ ] Router in `dispatch/router.rs` correctly calls new controller functions
- [ ] All controller handlers receive ExecutionContext and access services through it
- [ ] No direct service instantiation in controller code
- [ ] All imports updated to use `presentation::controllers` paths

**Documentation and Testing**:

- [ ] Module documentation explains controllers layer purpose and MVC alignment
- [ ] All tests updated and passing
- [ ] No references to old "commands" terminology in documentation
- [ ] Import paths consistently use "controllers" terminology

**Container Integration**:

- [ ] Controllers access all services through ExecutionContext
- [ ] No duplicate dependency injection patterns (factory removed)
- [ ] Container integration works correctly through dispatch layer
- [ ] Lazy loading of services works as expected

## Related Documentation

- [Presentation Layer Reorganization Refactor Plan](../refactors/plans/presentation-layer-reorganization.md) - Overall refactoring strategy
- [Epic #154: Presentation Layer Reorganization](154-epic-presentation-layer-reorganization.md) - Parent epic tracking
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md) - Architecture principles
- [Module Organization](../contributing/module-organization.md) - Code organization patterns

## Notes

### Why "Controllers" Instead of "Commands"?

- **Standard Terminology**: Controllers is the standard term in MVC patterns
- **Clear Responsibility**: Controllers handle request routing and coordinate responses
- **Industry Alignment**: Familiar terminology for developers from web frameworks
- **Separation of Concerns**: Clear distinction from business logic (commands) vs presentation logic (controllers)

### ExecutionContext Benefits

- **Clean Dependencies**: Controllers get typed access to services without Container complexity
- **Future-Proof**: Easy to add new services without changing controller signatures
- **Thread Safety**: Proper concurrent access to shared services
- **Testing**: Easy to inject test doubles through Container

### Factory Pattern Removal Rationale

The `commands/factory.rs` duplicates the Container pattern from `bootstrap/container.rs`:

- **Container**: Centralized dependency injection with lazy loading
- **Factory**: Redundant pattern that creates the same services
- **ExecutionContext**: Provides clean access to Container services

Removing the factory simplifies the architecture and eliminates duplicate dependency management.
