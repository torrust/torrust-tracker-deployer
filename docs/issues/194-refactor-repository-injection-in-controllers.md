# Refactor Repository Injection in Presentation Layer Controllers

**Issue**: #194
**Parent Epic**: N/A (Standalone refactoring task)
**Related**:

- [docs/codebase-architecture.md](../codebase-architecture.md)
- [docs/development-principles.md](../development-principles.md)
- [src/bootstrap/container.rs](../../src/bootstrap/container.rs)

## Overview

Refactor presentation layer controllers to inject the `EnvironmentRepository` trait directly instead of `RepositoryFactory`, treating the repository as a global dependency managed by the application container. This simplifies controller construction, reduces inconsistencies, and aligns with the dependency injection pattern established in the bootstrap container.

## Background

Currently, presentation layer controllers inconsistently handle repository dependencies:

- **Inconsistent Injection**: Some controllers inject `RepositoryFactory`, while `DestroyCommandController` already injects `Arc<dyn EnvironmentRepository>` directly
- **Unnecessary Complexity**: Controllers instantiate repositories with `repository_factory.create(data_dir)`, but the repository doesn't need the environment name at construction time - it only needs the base data directory
- **Redundant Dependencies**: `ConfigureCommandController` has both `repository: Arc<dyn EnvironmentRepository>` AND `repository_factory: Arc<RepositoryFactory>`, which is inconsistent

The key insight is that `EnvironmentRepository` is actually a **global dependency**, not an environment-specific one:

```rust
// Repository builds the final path using environment name AFTER instantiation
fn environment_file_path(&self, name: &EnvironmentName) -> PathBuf {
    self.base_dir.join(name.as_str()).join("environment.json")
}
```

Since the repository only needs the base `data/` directory (not `data/{ENV_NAME}`), it can be instantiated once at application startup and reused across all controllers.

## Goals

- [ ] Add `EnvironmentRepository` as a container-managed service
- [ ] Standardize repository injection across all controllers
- [ ] Remove redundant `RepositoryFactory` usage from controllers
- [ ] Evaluate if `RepositoryFactory` is still needed after refactoring
- [ ] Ensure all tests pass after changes
- [ ] Maintain backward compatibility with existing command handlers

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation + Infrastructure
**Module Paths**:

- `src/bootstrap/container.rs` (Container enhancement)
- `src/presentation/controllers/*/handler.rs` (Controller refactoring)

**Pattern**: Dependency Injection via Service Container

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Respect dependency flow rules (Presentation → Application → Domain ← Infrastructure)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Repository remains in Infrastructure layer
- [ ] Controllers in Presentation layer only orchestrate, no business logic
- [ ] Container manages global service lifecycle
- [ ] No environment-specific state in container (only base data directory)
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))

### Anti-Patterns to Avoid

- ❌ Mixing concerns across layers
- ❌ Instantiating repositories multiple times with different configurations
- ❌ Storing environment-specific state in the container
- ❌ Breaking existing command handler APIs

## Specifications

### Current State Analysis

**Controllers with `RepositoryFactory` injection:**

```rust
// src/presentation/controllers/create/subcommands/environment/handler.rs
pub struct CreateEnvironmentCommandController {
    repository_factory: Arc<RepositoryFactory>,
    clock: Arc<dyn Clock>,
    progress: ProgressReporter,
}

// src/presentation/controllers/provision/handler.rs
pub struct ProvisionCommandController {
    repository: Arc<dyn EnvironmentRepository>,
    repository_factory: Arc<RepositoryFactory>,  // REDUNDANT!
    clock: Arc<dyn Clock>,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    progress: ProgressReporter,
}

// src/presentation/controllers/configure/handler.rs
pub struct ConfigureCommandController {
    repository: Arc<dyn EnvironmentRepository>,
    repository_factory: Arc<RepositoryFactory>,  // REDUNDANT!
    clock: Arc<dyn Clock>,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    progress: ProgressReporter,
}

// src/presentation/controllers/test/handler.rs
struct TestCommandController {
    repository: Arc<dyn EnvironmentRepository>,
    progress: ProgressReporter,
}
```

**Controller already using direct repository injection (correct pattern):**

```rust
// src/presentation/controllers/destroy/handler.rs
pub struct DestroyCommandController {
    repository: Arc<dyn EnvironmentRepository>,
    clock: Arc<dyn Clock>,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    progress: ProgressReporter,
}
```

### Current Repository Instantiation Pattern

```rust
// Controllers currently do this:
let data_dir = working_dir.join("data");
let repository = repository_factory.create(data_dir);
```

**Key Observation**: The `data_dir` is always `{working_dir}/data` - a predictable path that doesn't depend on runtime environment name. This makes the repository a perfect candidate for container management.

### Target State

**Container enhancement:**

```rust
// src/bootstrap/container.rs
pub struct Container {
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    repository_factory: Arc<RepositoryFactory>,  // Potentially removable
    repository: Arc<dyn EnvironmentRepository>,   // NEW
    clock: Arc<dyn Clock>,
}

impl Container {
    pub fn new(verbosity_level: VerbosityLevel, working_dir: PathBuf) -> Self {
        let user_output = Arc::new(ReentrantMutex::new(RefCell::new(UserOutput::new(verbosity_level))));
        let repository_factory = Arc::new(RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT));

        // Create repository once for the entire application
        let data_dir = working_dir.join("data");
        let repository = repository_factory.create(data_dir);

        let clock: Arc<dyn Clock> = Arc::new(SystemClock);

        Self {
            user_output,
            repository_factory,  // Evaluate if still needed
            repository,
            clock,
        }
    }

    pub fn repository(&self) -> Arc<dyn EnvironmentRepository> {
        Arc::clone(&self.repository)
    }
}
```

**Updated controllers (all follow same pattern):**

```rust
pub struct ProvisionCommandController {
    repository: Arc<dyn EnvironmentRepository>,
    clock: Arc<dyn Clock>,
    user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    progress: ProgressReporter,
}

impl ProvisionCommandController {
    pub fn new(
        repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn Clock>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        let progress = ProgressReporter::new(user_output.clone(), PROVISION_WORKFLOW_STEPS);

        Self {
            repository,
            clock,
            user_output,
            progress,
        }
    }
}
```

### Working Directory Challenge & Solution

**CRITICAL ISSUE**: The container is currently instantiated without knowledge of the working directory:

```rust
// src/presentation/dispatch/context.rs
impl ExecutionContext {
    pub fn new(container: Arc<Container>) -> Self {
        Self { container }
    }
}

// Current container initialization (in main.rs or similar)
let container = Arc::new(Container::new(VerbosityLevel::Normal));
```

However, controllers receive `working_dir` at runtime:

```rust
pub async fn handle(
    environment_name: &str,
    working_dir: &std::path::Path,  // Runtime parameter!
    context: &ExecutionContext,
) -> Result<Environment<Provisioned>, ProvisionSubcommandError>
```

**CHOSEN SOLUTION**: Pass `working_dir` to `Container::new()` during initialization.

The working directory is known at application startup since it's either:

- Default: current directory (`./`)
- Specified via CLI flag: `--working-dir /path/to/dir`

This means we can pass it to `Container::new()` during bootstrap. The container signature will change from:

```rust
// Before
Container::new(VerbosityLevel::Normal)

// After
Container::new(VerbosityLevel::Normal, working_dir)
```

**Future Enhancement**: Eventually we might pass the entire `Config` struct (from `src/config/mod.rs`) instead of individual parameters, but for now adding `working_dir` as a parameter is the right incremental step.

## Implementation Plan

### Phase 1: Container Enhancement (2 hours)

- [ ] Task 1.1: Modify `Container::new()` to accept `working_dir: PathBuf` parameter
  - Note: In the future, we might pass the entire `Config` struct instead of individual parameters
- [ ] Task 1.2: Add `repository: Arc<dyn EnvironmentRepository>` field to `Container`
- [ ] Task 1.3: Instantiate repository in `Container::new()` using `working_dir.join("data")`
- [ ] Task 1.4: Add `repository()` accessor method to `Container`
- [ ] Task 1.5: Update `Container` tests to pass working_dir
- [ ] Task 1.6: Update `ExecutionContext` creation sites to pass working_dir to container

### Phase 2: Update Controller Constructors (3 hours)

Update each controller to inject repository directly instead of factory:

- [ ] Task 2.1: Update `CreateEnvironmentCommandController`:
  - Remove `repository_factory` field
  - Add `repository: Arc<dyn EnvironmentRepository>` field
  - Update constructor signature
  - Remove repository instantiation logic
- [ ] Task 2.2: Update `ProvisionCommandController`:
  - Remove `repository_factory` field (already has `repository`)
  - Update constructor to inject repository from container
  - Remove repository instantiation logic
- [ ] Task 2.3: Update `ConfigureCommandController`:
  - Remove `repository_factory` field (already has `repository`)
  - Update constructor to inject repository from container
  - Remove repository instantiation logic
- [ ] Task 2.4: Update `TestCommandController`:
  - Verify it already uses repository directly (no changes needed)
- [ ] Task 2.5: Update `DestroyCommandController`:
  - Verify it already uses repository directly (no changes needed)
- [ ] Task 2.6: Update `CreateTemplateCommandController`:
  - Check if it needs repository injection

### Phase 3: Update Handler Functions (2 hours)

Update `handle()` and `handle_*_command()` functions:

- [ ] Task 3.1: Update handler functions to use `context.repository()` instead of `context.repository_factory()`
- [ ] Task 3.2: Remove `working_dir.join("data")` logic from handlers
- [ ] Task 3.3: Update controller instantiation to pass repository from context
- [ ] Task 3.4: Verify working_dir is still passed for other purposes (e.g., template paths)

### Phase 4: Evaluate RepositoryFactory Necessity (1 hour)

- [ ] Task 4.1: Search codebase for remaining `RepositoryFactory` usage
- [ ] Task 4.2: If only used by Container, keep it as implementation detail
- [ ] Task 4.3: If unused elsewhere, evaluate removing from Container public API
- [ ] Task 4.4: Update documentation if factory is removed or made private
- [ ] Task 4.5: Remove `repository_factory()` accessor from Container if no longer needed

### Phase 5: Main Application Bootstrap (1 hour)

- [ ] Task 5.1: Update main application entry point to determine working_dir early
- [ ] Task 5.2: Pass working_dir to `Container::new()`
- [ ] Task 5.3: Update CLI argument parsing if needed (working_dir as global option)

### Phase 6: Testing & Validation (2 hours)

- [ ] Task 6.1: Update unit tests for controllers with new constructor signatures
- [ ] Task 6.2: Update integration tests if needed
- [ ] Task 6.3: Run full test suite: `cargo test`
- [ ] Task 6.4: Run E2E tests: `cargo run --bin e2e-provision-and-destroy-tests`
- [ ] Task 6.5: Run E2E tests: `cargo run --bin e2e-config-tests`

### Phase 7: Quality Checks (1 hour)

- [ ] Task 7.1: Run linters: `cargo run --bin linter all`
- [ ] Task 7.2: Check for unused dependencies: `cargo machete`
- [ ] Task 7.3: Run pre-commit verification: `./scripts/pre-commit.sh`
- [ ] Task 7.4: Fix any linting or compilation errors

### Phase 8: Documentation Updates (1 hour)

- [ ] Task 8.1: Update Container documentation with working_dir parameter
- [ ] Task 8.2: Update controller documentation to reflect repository injection
- [ ] Task 8.3: Add code examples showing new constructor patterns
- [ ] Task 8.4: Update architectural documentation if needed

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Functional Requirements**:

- [ ] Container has `repository: Arc<dyn EnvironmentRepository>` field
- [ ] Container accepts `working_dir: PathBuf` in `new()` method
- [ ] Container has `repository()` accessor method
- [ ] All controllers inject `Arc<dyn EnvironmentRepository>` directly
- [ ] No controllers have both `repository` and `repository_factory` fields
- [ ] Repository is instantiated exactly once per application lifecycle
- [ ] All controllers use consistent dependency injection pattern

**Code Quality**:

- [ ] No duplicate repository instantiation logic across controllers
- [ ] Repository instantiation centralized in Container
- [ ] Controller constructors have consistent signatures
- [ ] All imports are necessary (no unused imports after refactoring)

**Testing**:

- [ ] All unit tests pass: `cargo test`
- [ ] All E2E tests pass:
  - [ ] `cargo run --bin e2e-provision-and-destroy-tests`
  - [ ] `cargo run --bin e2e-config-tests`
- [ ] Controller tests updated with new constructor signatures
- [ ] Container tests updated to pass working_dir

**Backward Compatibility**:

- [ ] Existing command behavior unchanged
- [ ] Public command handler APIs maintain same functionality
- [ ] CLI interface unchanged (unless working_dir becomes global flag)

**Documentation**:

- [ ] Container documentation updated
- [ ] Controller documentation updated
- [ ] Code examples reflect new patterns
- [ ] Breaking changes (if any) documented

**Optional Goals**:

- [ ] `RepositoryFactory` removed from Container public API if no longer needed
- [ ] `repository_factory()` accessor method removed if unused
- [ ] working_dir becomes global CLI option (if beneficial)

## Related Documentation

- [docs/codebase-architecture.md](../codebase-architecture.md) - DDD architecture and layer responsibilities
- [docs/development-principles.md](../development-principles.md) - Dependency injection principles
- [docs/contributing/ddd-layer-placement.md](../contributing/ddd-layer-placement.md) - Layer placement guide
- [docs/contributing/module-organization.md](../contributing/module-organization.md) - Module organization conventions
- [docs/contributing/error-handling.md](../contributing/error-handling.md) - Error handling patterns
- [src/bootstrap/container.rs](../../src/bootstrap/container.rs) - Application service container
- [src/infrastructure/persistence/repository_factory.rs](../../src/infrastructure/persistence/repository_factory.rs) - Repository factory implementation
- [src/infrastructure/persistence/filesystem/file_environment_repository.rs](../../src/infrastructure/persistence/filesystem/file_environment_repository.rs) - File repository implementation

## Notes

### Why This Refactoring Matters

1. **Reduces Complexity**: Controllers no longer need to know how to construct repositories
2. **Ensures Consistency**: Single repository instance with consistent configuration (lock timeout)
3. **Simplifies Testing**: Easier to inject mock repositories in tests
4. **Aligns with Architecture**: Container manages global dependencies, not controllers
5. **Eliminates Redundancy**: Removes duplicate `repository` + `repository_factory` fields

### Performance Impact

✅ **Positive**: Repository instantiated once instead of per-controller
✅ **Positive**: Less Arc cloning during controller construction
⚠️ **Neutral**: Working directory determined at startup (already the case)

### Migration Risk Assessment

**Low Risk**:

- Changes are internal to presentation layer
- Command handler public APIs remain unchanged
- Tests will catch any integration issues

**Medium Risk**:

- Container initialization signature changes (impacts main.rs)
- Working directory must be known at startup (requires careful handling)

### Future Enhancements

After this refactoring, consider:

1. **Remove RepositoryFactory from public API**: If only used internally by Container
2. **Pass Config struct to Container**: Instead of individual parameters (working_dir, verbosity_level), pass the entire `Config` struct from `src/config/mod.rs` for better scalability
3. **Multiple Repository Support**: If we need different storage backends (DB, cloud)
4. **Repository Configuration**: If we need runtime configuration changes
5. **Working Directory Validation**: Add checks at container initialization
