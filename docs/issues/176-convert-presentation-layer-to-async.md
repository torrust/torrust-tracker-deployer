# Convert Presentation Layer to Async

**Issue**: #176
**Parent Epic**: #2 - Scaffolding for main app
**Related**:

- [docs/codebase-architecture.md](../codebase-architecture.md)
- [docs/contributing/module-organization.md](../contributing/module-organization.md)
- [docs/development-principles.md](../development-principles.md)

## Overview

Convert the presentation layer (controllers and command dispatch) from synchronous to asynchronous execution to eliminate the need for runtime creation in handler functions. Currently, the presentation layer is synchronous while the application layer uses async operations, requiring awkward runtime creation patterns like:

```rust
// Current ugly pattern in presentation layer
let runtime = tokio::runtime::Runtime::new().map_err(|e| {
    ProvisionSubcommandError::RepositoryAccessFailed {
        data_dir: "runtime".to_string(),
        reason: format!("Failed to create tokio runtime: {e}"),
    }
})?;

let provisioned = runtime.block_on(handler.execute(env_name))?;
```

This refactoring will make the entire execution stack async from `main.rs` down through dispatch, routing, and controllers, providing cleaner code and better alignment with async infrastructure operations (SSH, file I/O, network calls).

## Goals

- [ ] Eliminate runtime creation in presentation layer handlers
- [ ] Make command execution flow async from main entry point
- [ ] Improve code maintainability and architectural consistency
- [ ] Enable future async optimizations (parallel operations, better task scheduling)
- [ ] Follow idiomatic Rust async patterns used by successful CLI tools

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/`, `src/main.rs`, `src/bootstrap/app.rs`
**Pattern**: CLI Subcommand Handlers, Command Router, Application Bootstrap

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Presentation layer should cleanly delegate to application layer without runtime gymnastics
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Single tokio runtime created at application entry point (`main.rs`)
- [ ] All presentation layer handlers become async functions
- [ ] Command router becomes async
- [ ] Bootstrap layer handles async initialization
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Maintain clear separation: Presentation → Application → Domain → Infrastructure

### Anti-Patterns to Avoid

- ❌ Creating multiple runtimes in different parts of the application
- ❌ Mixing sync and async execution within the same layer
- ❌ Using `block_on` in presentation layer handlers
- ❌ Losing error context during async conversions

## Specifications

### Current Architecture Problem

**Synchronous Presentation Layer**:

- `src/main.rs`: `fn main()` - Synchronous entry point
- `src/bootstrap/app.rs`: `pub fn run()` - Synchronous bootstrap
- `src/presentation/dispatch/router.rs`: `pub fn route_command()` - Synchronous router
- Controller handlers: `pub fn handle()` / `pub fn handle_*_command()` - All synchronous

**Async Application Layer**:

- `ProvisionCommandHandler::execute()` - Async
- `ConfigureCommandHandler::execute()` - Async (likely)
- `TestCommandHandler::execute()` - Async
- Template rendering steps - Async (use `tokio::fs`)
- SSH operations - Async (network I/O)

**Mismatch Pattern**:

```rust
// Presentation layer (sync) calling application layer (async)
pub fn handle_provision_command(...) -> Result<...> {
    let runtime = tokio::runtime::Runtime::new()?; // ❌ Bad pattern
    let result = runtime.block_on(async_handler.execute())?;
    Ok(result)
}
```

### Target Architecture

**Async Throughout Stack**:

```rust
// main.rs
#[tokio::main]
async fn main() {
    bootstrap::app::run().await;
}

// bootstrap/app.rs
pub async fn run() {
    // ... initialization ...
    presentation::dispatch::route_command(command, working_dir, &context).await?;
}

// presentation/dispatch/router.rs
pub async fn route_command(...) -> Result<(), CommandError> {
    match command {
        Commands::Provision { environment } => {
            provision::handle(&environment, working_dir, context).await?;
        }
        // ... other commands ...
    }
}

// presentation/controllers/provision/handler.rs
pub async fn handle(...) -> Result<...> {
    handle_provision_command(...).await
}

pub async fn handle_provision_command(...) -> Result<...> {
    let handler = ProvisionCommandHandler::new(...);
    handler.execute(env_name).await  // ✅ Clean async call
}
```

### Files Requiring Changes

**Core Entry Points**:

1. `src/main.rs` - Add `#[tokio::main]` attribute
2. `src/bootstrap/app.rs` - Convert `run()` to async

**Dispatch Layer**:

1. `src/presentation/dispatch/router.rs` - Convert `route_command()` to async

**Controller Handlers** (all in `src/presentation/controllers/`):

1. `provision/handler.rs`:

   - `handle()` → async
   - `handle_provision_command()` → async
   - `ProvisionCommandController::execute()` → async (and all its methods)

2. `destroy/handler.rs`:

   - `handle()` → async
   - `handle_destroy_command()` → async
   - `DestroyCommandController::execute()` → async (and all its methods)

3. `create/mod.rs` - `route_command()` → async

4. `create/subcommands/environment/handler.rs`:

   - `handle()` → async
   - `handle_environment_creation_command()` → async
   - `CreateEnvironmentController::execute()` → async (and all its methods)

5. `create/subcommands/template/handler.rs`:
   - `handle()` → async
   - `handle_template_creation_command()` → async
   - `CreateTemplateController::execute()` → async (and all its methods)

**Future Commands** (when implemented):

- `configure/handler.rs` - Already likely needs async
- `test/handler.rs` - Already async in application layer
- `release/handler.rs` - Future command

### Testing Considerations

**Unit Tests**:

- Update test functions to use `#[tokio::test]` instead of `#[test]`
- No need for runtime creation in tests
- Async assertions work naturally

**Integration/E2E Tests**:

- Already using tokio runtime for async operations
- Should continue to work with async presentation layer
- May simplify some test setup code

### Dependencies

**Current Dependencies**:

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

**No New Dependencies Required** - Tokio is already included with the necessary features.

## Implementation Plan

### Phase 0: Document Architectural Decision (15 minutes)

- [ ] Create ADR in `docs/decisions/` following [docs/decisions/README.md](../decisions/README.md)
  - [ ] Document the decision to use async throughout the presentation layer
  - [ ] Include context: sync/async boundary problem with runtime creation
  - [ ] Document the decision: make entire stack async from main.rs down
  - [ ] List consequences: cleaner code, better performance, future-proof
  - [ ] Document alternatives considered: convert application layer to sync
  - [ ] Reference related decisions and resources
  - [ ] Add entry to decision index in `docs/decisions/README.md`

### Phase 1: Entry Point and Bootstrap (30 minutes)

- [ ] Update `src/main.rs`:

  - [ ] Add `#[tokio::main]` attribute to `main()`
  - [ ] Make `main()` async
  - [ ] Update call to `bootstrap::app::run().await`

- [ ] Update `src/bootstrap/app.rs`:
  - [ ] Make `run()` function async
  - [ ] Update call to `route_command(...).await`
  - [ ] Update all internal async calls

### Phase 2: Command Router (15 minutes)

- [ ] Update `src/presentation/dispatch/router.rs`:
  - [ ] Make `route_command()` function async
  - [ ] Add `.await` to all handler calls in match arms
  - [ ] Update documentation examples

### Phase 3: Provision Controller (45 minutes)

- [ ] Update `src/presentation/controllers/provision/handler.rs`:
  - [ ] Make `handle()` async
  - [ ] Make `handle_provision_command()` async
  - [ ] Make `ProvisionCommandController::execute()` async
  - [ ] Make all controller methods async (validation, creation, provisioning, completion)
  - [ ] **Remove runtime creation code** - replace `runtime.block_on()` with direct `.await`
  - [ ] Update documentation examples
  - [ ] Update unit tests to use `#[tokio::test]`

### Phase 4: Destroy Controller (30 minutes)

- [ ] Update `src/presentation/controllers/destroy/handler.rs`:
  - [ ] Make `handle()` async
  - [ ] Make `handle_destroy_command()` async
  - [ ] Make `DestroyCommandController::execute()` async (if applicable)
  - [ ] Make all controller methods async
  - [ ] Update documentation examples
  - [ ] Update unit tests to use `#[tokio::test]`

### Phase 5: Create Command Router and Controllers (1 hour)

- [ ] Update `src/presentation/controllers/create/mod.rs`:

  - [ ] Make `route_command()` async
  - [ ] Add `.await` to subcommand handler calls

- [ ] Update `src/presentation/controllers/create/subcommands/environment/handler.rs`:

  - [ ] Make `handle()` async
  - [ ] Make `handle_environment_creation_command()` async
  - [ ] Make `CreateEnvironmentController::execute()` async
  - [ ] Make all controller methods async
  - [ ] Update documentation examples
  - [ ] Update unit tests to use `#[tokio::test]`

- [ ] Update `src/presentation/controllers/create/subcommands/template/handler.rs`:
  - [ ] Make `handle()` async
  - [ ] Make `handle_template_creation_command()` async
  - [ ] Make `CreateTemplateController::execute()` async
  - [ ] Make all controller methods async
  - [ ] Update documentation examples
  - [ ] Update unit tests to use `#[tokio::test]`

### Phase 6: Testing and Validation (45 minutes)

- [ ] Run all unit tests: `cargo test`
- [ ] Run E2E tests:
  - [ ] `cargo run --bin e2e-provision-and-destroy-tests`
  - [ ] `cargo run --bin e2e-config-tests`
  - [ ] `cargo run --bin e2e-tests-full` (local only)
- [ ] Test all commands manually:
  - [ ] `cargo run -- create environment test-env`
  - [ ] `cargo run -- provision test-env`
  - [ ] `cargo run -- destroy test-env`
- [ ] Verify no runtime creation patterns remain in presentation layer
- [ ] Check for proper error propagation through async stack

### Phase 7: Documentation Updates (30 minutes)

- [ ] Update README examples if they show sync usage
- [ ] Update user guide if it references synchronous behavior
- [ ] Update module documentation in affected files
- [ ] Add note about async architecture in `docs/codebase-architecture.md` if needed

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
  - All linters pass (clippy, rustfmt, etc.)
  - All unit tests pass
  - All E2E tests pass
  - Documentation builds successfully

**Documentation Criteria**:

- [ ] ADR created in `docs/decisions/` documenting the async presentation layer decision
- [ ] ADR entry added to decision index in `docs/decisions/README.md`

**Architecture Criteria**:

- [ ] `src/main.rs` uses `#[tokio::main]` and async main function
- [ ] `src/bootstrap/app.rs::run()` is async
- [ ] `src/presentation/dispatch/router.rs::route_command()` is async
- [ ] All controller `handle()` functions are async
- [ ] All controller `handle_*_command()` functions are async
- [ ] All `Controller::execute()` methods are async
- [ ] **Zero** runtime creation patterns in presentation layer (no `Runtime::new()`, no `block_on()`)
- [ ] Error handling maintains context through async boundaries
- [ ] All async calls properly use `.await`

**Testing Criteria**:

- [ ] All unit tests updated to `#[tokio::test]` where needed
- [ ] All E2E test suites pass
- [ ] Manual testing confirms all commands work correctly
- [ ] No degradation in error messages or user experience

**Code Quality Criteria**:

- [ ] Documentation examples updated to show async usage
- [ ] No unused imports or dead code introduced
- [ ] Function signatures are idiomatic async Rust
- [ ] Error types remain unchanged (same error handling, just async propagation)

**Verification Commands**:

```bash
# Verify no runtime creation in presentation layer
grep -r "Runtime::new()" src/presentation/
# Should return no results

# Verify no block_on in presentation layer
grep -r "block_on" src/presentation/
# Should return no results

# Verify all handlers are async
grep -r "pub fn handle" src/presentation/controllers/ | grep -v "async"
# Should return minimal results (only helper functions, not main handlers)
```

## Related Documentation

- [docs/codebase-architecture.md](../codebase-architecture.md) - DDD layer architecture
- [docs/contributing/module-organization.md](../contributing/module-organization.md) - Module organization patterns
- [docs/development-principles.md](../development-principles.md) - Core development principles
- [docs/contributing/error-handling.md](../contributing/error-handling.md) - Error handling conventions
- [Tokio Documentation](https://tokio.rs/tokio/tutorial) - Tokio async runtime guide

## Notes

### Why Async Throughout?

**Current Pain Points**:

1. Ugly runtime creation code in presentation layer
2. Mixing sync/async boundaries creates confusion
3. Cannot leverage async benefits in presentation layer
4. Testing is more complex with runtime management

**Benefits of Full Async Stack**:

1. **Cleaner Code**: No runtime gymnastics, direct `.await` calls
2. **Better Performance**: Single runtime, proper task scheduling
3. **Future-Proof**: Easy to add parallel operations later
4. **Idiomatic**: Follows patterns from successful Rust CLI tools (ripgrep, cargo, etc.)
5. **Easier Testing**: `#[tokio::test]` is simpler than manual runtime creation

### Alternative Considered: Convert to Sync

We could convert the application layer to sync by:

- Replace `tokio::fs` with `std::fs`
- Replace async SSH operations with blocking calls
- Remove all async/await

**Why We Rejected This**:

- Network I/O (SSH) is inherently async - making it sync blocks threads
- File I/O benefits from async in larger operations
- Blocks future performance optimizations
- Goes against Rust ecosystem trends for I/O-heavy applications

### Impact on Future Development

**Enables**:

- Parallel provisioning of multiple environments
- Concurrent template rendering
- Better progress reporting during long operations
- Streaming output from external commands

**Does Not Break**:

- Existing application layer code (already async)
- Domain layer (remains sync, no I/O)
- Infrastructure adapters (already async where needed)

### Estimated Total Time

**Total**: ~4 hours 15 minutes

- Phase 0: 15 minutes (ADR creation)
- Phase 1: 30 minutes
- Phase 2: 15 minutes
- Phase 3: 45 minutes
- Phase 4: 30 minutes
- Phase 5: 1 hour
- Phase 6: 45 minutes
- Phase 7: 30 minutes

**Recommended Approach**: Start with Phase 0 (ADR) to document the decision, then implement in order, testing after each phase to catch issues early.
