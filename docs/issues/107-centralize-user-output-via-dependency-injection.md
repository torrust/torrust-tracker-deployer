# Centralize UserOutput via Dependency Injection

**Issue**: [#107](https://github.com/torrust/torrust-tracker-deployer/issues/107)
**Parent Epic**: [#102](https://github.com/torrust/torrust-tracker-deployer/issues/102) - User Output Architecture Improvements
**Related**:

- [#103](https://github.com/torrust/torrust-tracker-deployer/issues/103) - Extract Verbosity Filtering Logic
- [Refactoring Plan](../refactors/plans/user-output-architecture-improvements.md)

## Overview

Currently, the `UserOutput` type is instantiated in multiple places throughout the presentation layer, which can lead to inconsistent user-facing messages with different verbosity levels, styles, or formatting. This task centralizes `UserOutput` instantiation in the application bootstrap phase and passes it down through the function call chain using dependency injection via `Arc<UserOutput>`.

**Problem**: Decentralized construction of `UserOutput` creates inconsistency risk:

1. `src/presentation/commands/mod.rs` (line 129) - Error handler creates its own instance
2. `src/presentation/commands/create/subcommands/template.rs` (line 38) - Template generation creates its own instance
3. `src/presentation/commands/context.rs` (line 122) - Command context creates its own instance

**Solution**: Bootstrap a single `UserOutput` instance during application initialization and pass it down through the command execution chain using `Arc<UserOutput>` for shared ownership.

## Goals

- [ ] Ensure consistent user-facing output across the entire application
- [ ] Centralize `UserOutput` configuration in application bootstrap
- [ ] Enable future extensibility (e.g., different output sinks, testing mocks)
- [ ] Follow dependency injection best practices
- [ ] Maintain clean architecture boundaries (presentation layer concerns)

## 🏗️ Architecture Requirements

**DDD Layer**: Infrastructure (Container), Presentation (usage)
**Module Path**:

- Container: `src/bootstrap/container.rs` (new module)
- Bootstrap integration: `src/bootstrap/app.rs` (existing)
- Presentation layer: Multiple files (passing `Arc<UserOutput>` through call chain)

**Pattern**: Dependency Injection Container + Function Parameter Threading

### Module Structure Requirements

- [ ] Create `Container` type in `src/bootstrap/container.rs` for centralized service initialization
- [ ] Bootstrap container in `src/bootstrap/app.rs` after logging initialization
- [ ] Thread `Arc<UserOutput>` through presentation layer function signatures
- [ ] Update existing `UserOutput` construction sites to use injected dependency
- [ ] Respect layering: Container in bootstrap, usage in presentation

### Architectural Constraints

- [ ] **No global state** - Use `Arc<UserOutput>` for shared ownership, not global variables
- [ ] **Thread-safe** - `UserOutput` must be thread-safe (already is with `Arc`)
- [ ] **Bootstrap only** - Container initialization happens once during app startup
- [ ] **Dependency flow** - Bootstrap → Presentation layer only (no reverse dependency)
- [ ] **Maintain existing behavior** - User-facing output behavior must remain unchanged

### Anti-Patterns to Avoid

- ❌ **Global mutable state** - Don't use `static mut` or `lazy_static!` for `UserOutput`
- ❌ **Service locator pattern** - Don't create a global registry for dependency lookup
- ❌ **Hidden dependencies** - All functions should explicitly declare `Arc<UserOutput>` parameter
- ❌ **Layer violations** - Don't pass container into domain or application layers

## Specifications

### 1. Bootstrap Container Module

Create a new module `src/bootstrap/container.rs` to hold application-wide services:

````rust
//! Application Service Container
//!
//! This module provides centralized initialization of application-wide services
//! that need consistent configuration across the entire application.

use std::sync::Arc;

use crate::presentation::user_output::UserOutput;
use crate::presentation::commands::constants::DEFAULT_VERBOSITY;

/// Application service container
///
/// Holds shared services initialized during application bootstrap.
/// Services are wrapped in `Arc` for thread-safe shared ownership across
/// the application.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::bootstrap::container::Container;
///
/// let container = Container::new();
/// let user_output = container.user_output();
/// ```
#[derive(Clone)]
pub struct Container {
    user_output: Arc<UserOutput>,
}

impl Container {
    /// Create a new container with initialized services
    ///
    /// Uses `DEFAULT_VERBOSITY` for user output. In the future, this may
    /// accept a verbosity parameter from CLI flags.
    pub fn new() -> Self {
        let user_output = Arc::new(UserOutput::new(DEFAULT_VERBOSITY));

        Self { user_output }
    }
}

    /// Get shared reference to user output service
    ///
    /// Returns an `Arc<UserOutput>` that can be cheaply cloned and shared
    /// across threads and function calls.
    #[must_use]
    pub fn user_output(&self) -> Arc<UserOutput> {
        Arc::clone(&self.user_output)
    }
}
````

**Module Registration**: Update `src/bootstrap/mod.rs` to include:

```rust
pub mod container;

// Re-export for convenience
pub use container::Container;
```

### 2. Bootstrap Integration in `app.rs`

Modify `src/bootstrap/app.rs` to create the container after logging initialization:

```rust
pub fn run() {
    let cli = presentation::Cli::parse();

    let logging_config = cli.global.logging_config();

    bootstrap::logging::init_subscriber(logging_config);

    info!(
        app = "torrust-tracker-deployer",
        version = env!("CARGO_PKG_VERSION"),
        log_dir = %cli.global.log_dir.display(),
        log_file_format = ?cli.global.log_file_format,
        log_stderr_format = ?cli.global.log_stderr_format,
        log_output = ?cli.global.log_output,
        "Application started"
    );

    // 🆕 Create container with default user output
    // Note: Uses DEFAULT_VERBOSITY for now. CLI verbosity flags will be added in a future task.
    let container = bootstrap::Container::new();

    match cli.command {
        Some(command) => {
            if let Err(e) = presentation::execute(command, &cli.global.working_dir, container.user_output()) {
                presentation::handle_error(&e, container.user_output());
                std::process::exit(1);
            }
        }
        None => {
            bootstrap::help::display_getting_started();
        }
    }

    info!("Application finished");
}
```

### 3. Update Function Signatures in Presentation Layer

#### 3.1. Update `presentation::execute`

**File**: `src/presentation/commands/mod.rs`

```rust
pub fn execute(
    command: Commands,
    working_dir: &std::path::Path,
    user_output: Arc<UserOutput>, // 🆕 Add user_output parameter
) -> Result<(), CommandError> {
    match command {
        Commands::Create { action } => {
            create::handle_create_command(action, working_dir, user_output)?;
            Ok(())
        }
        Commands::Destroy { environment } => {
            destroy::handle_destroy_command(&environment, working_dir, user_output)?;
            Ok(())
        }
    }
}
```

#### 3.2. Update `presentation::handle_error`

**File**: `src/presentation/commands/mod.rs`

```rust
pub fn handle_error(error: &CommandError, user_output: Arc<UserOutput>) {
    let help_text = error.help();

    user_output.error(&format!("{error}"));
    user_output.blank_line();
    user_output.info_block("For detailed troubleshooting:", &[help_text]);
}
```

#### 3.3. Update Command Handlers

**File**: `src/presentation/commands/create/handler.rs`

```rust
pub fn handle_create_command(
    action: CreateAction,
    working_dir: &Path,
    user_output: Arc<UserOutput>, // 🆕 Add user_output parameter
) -> Result<(), CreateSubcommandError> {
    match action {
        CreateAction::Environment { env_file } => {
            subcommands::handle_environment_creation(&env_file, working_dir, user_output)
        }
        CreateAction::Template { output_path } => {
            let template_path = output_path.unwrap_or_else(CreateAction::default_template_path);
            subcommands::handle_template_generation(&template_path, user_output)
        }
    }
}
```

**File**: `src/presentation/commands/destroy/handler.rs`

```rust
pub fn handle_destroy_command(
    environment_name: &str,
    working_dir: &Path,
    user_output: Arc<UserOutput>, // 🆕 Add user_output parameter
) -> Result<(), DestroySubcommandError> {
    // Use injected user_output instead of creating new instance
    // ... existing implementation
}
```

#### 3.4. Update Subcommand Handlers

**File**: `src/presentation/commands/create/subcommands/template.rs`

```rust
pub fn handle_template_generation(
    output_path: &Path,
    user_output: Arc<UserOutput>, // 🆕 Add user_output parameter
) -> Result<(), CreateSubcommandError> {
    // Remove: let mut user_output = UserOutput::new(DEFAULT_VERBOSITY);
    // Use injected user_output instead

    user_output.progress("Generating configuration template...");

    tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            EnvironmentCreationConfig::generate_template_file(output_path)
                .await
                .map_err(|source| CreateSubcommandError::TemplateGenerationFailed { source })
        })?;

    user_output.success(&format!(
        "Template generated successfully at '{}'",
        output_path.display()
    ));

    Ok(())
}
```

**File**: `src/presentation/commands/create/subcommands/environment.rs`

```rust
pub fn handle_environment_creation(
    env_file: &Path,
    working_dir: &Path,
    user_output: Arc<UserOutput>, // 🆕 Add user_output parameter
) -> Result<(), CreateSubcommandError> {
    // Use injected user_output throughout the function
    // ... existing implementation
}
```

### 4. Update Command Context

**File**: `src/presentation/commands/context.rs`

Update `CommandContext` to inject `UserOutput` via constructor instead of creating it internally. This follows the same pattern as other services (`repository`, `clock`) and enables future injection of those services as well.

```rust
pub struct CommandContext {
    pub repository: Arc<dyn EnvironmentRepository>,
    pub clock: Arc<dyn Clock>,
    pub user_output: Arc<UserOutput>, // 🔄 Changed: Now injected via constructor as Arc
}

impl CommandContext {
    /// Create a new command context with injected services
    ///
    /// # Arguments
    ///
    /// * `working_dir` - Working directory for environment storage
    /// * `user_output` - User output service for command feedback
    pub fn new(working_dir: PathBuf, user_output: Arc<UserOutput>) -> Self {
        let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        let repository = repository_factory.create(working_dir);
        let clock = Arc::new(SystemClock);

        Self {
            repository,
            clock,
            user_output,
        }
    }
}
```

**Design Rationale**:

- **Consistent pattern**: All services are now handled in the constructor (some created, some injected)
- **Future-proof**: Other services (`repository_factory`, `clock`) can be injected later following the same pattern
- **Field storage**: Keep `user_output` as a field since it's used across multiple methods in the context
- **Arc wrapper**: Use `Arc<UserOutput>` for cheap cloning and thread-safe sharing

## Implementation Plan

### Phase 1: Create Bootstrap Container (1-2 hours)

- [ ] Create `src/bootstrap/container.rs` with `Container` type
- [ ] Add `Arc<UserOutput>` field to `Container`
- [ ] Implement `new()` and `user_output()` methods
- [ ] Update `src/bootstrap/mod.rs` to export `Container`
- [ ] Add unit tests for `Container` construction and `user_output()` access

### Phase 2: Integrate Container in Bootstrap (1 hour)

- [ ] Create `Container` in `src/bootstrap/app.rs` after logging initialization
- [ ] Pass `container.user_output()` to `presentation::execute()`
- [ ] Pass `container.user_output()` to `presentation::handle_error()`

### Phase 3: Update Presentation Layer Signatures (2-3 hours)

- [ ] Update `presentation::execute()` signature to accept `Arc<UserOutput>`
- [ ] Update `presentation::handle_error()` signature to accept `Arc<UserOutput>`
- [ ] Update `create::handle_create_command()` signature
- [ ] Update `destroy::handle_destroy_command()` signature
- [ ] Update all subcommand handlers in `create/subcommands/` and `destroy/subcommands/`

### Phase 4: Remove Local UserOutput Constructions (1-2 hours)

- [ ] Remove `UserOutput::new()` call in `presentation::handle_error()`
- [ ] Remove `UserOutput::new()` call in `create/subcommands/template.rs`
- [ ] Update `CommandContext::new()` to accept `user_output: Arc<UserOutput>` parameter
- [ ] Update all `CommandContext::new()` call sites to pass injected `user_output`
- [ ] Search codebase for remaining `UserOutput::new()` calls and update them

### Phase 5: Update Tests (2-3 hours)

- [ ] Update all presentation layer tests to create and pass `Arc<UserOutput>`
- [ ] Update command handler tests
- [ ] Update subcommand tests
- [ ] Verify E2E tests still pass with new signatures
- [ ] Add integration test verifying consistent output configuration

### Phase 6: Documentation and Cleanup (1 hour)

- [ ] Update module documentation in `src/bootstrap/container.rs`
- [ ] Update function documentation for all changed signatures
- [ ] Add architectural note to `docs/codebase-architecture.md` about centralized services
- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] All linters pass (markdown, yaml, toml, clippy, rustfmt, shellcheck)
- [ ] All unit tests pass
- [ ] All E2E tests pass

**Architecture Checks**:

- [ ] `Container` type exists in `src/bootstrap/container.rs`
- [ ] `Container` is bootstrapped in `src/bootstrap/app.rs` after logging initialization
- [ ] No `UserOutput::new()` calls exist in presentation layer (except in tests)
- [ ] All presentation layer functions accept `Arc<UserOutput>` parameter
- [ ] `CommandContext` requires `Arc<UserOutput>` in constructor and stores it as a field
- [ ] `CommandContext::new()` accepts `user_output: Arc<UserOutput>` parameter
- [ ] No global mutable state used for `UserOutput`
- [ ] No service locator pattern used

**Behavior Checks**:

- [ ] User-facing output behavior remains unchanged (no visual differences)
- [ ] Default verbosity level (`DEFAULT_VERBOSITY`) is used consistently
- [ ] Error messages still display correctly with help text
- [ ] Progress messages still display correctly
- [ ] All commands produce consistent output style

**Testing Checks**:

- [ ] Container construction is tested
- [ ] Presentation layer functions are tested with injected `UserOutput`
- [ ] Integration test verifies consistent output configuration
- [ ] E2E tests pass without modification

**Documentation Checks**:

- [ ] Module documentation explains container purpose and usage
- [ ] Function documentation updated for new signatures
- [ ] Architecture documentation mentions centralized service pattern

## Related Documentation

- [Development Principles](../development-principles.md) - Observability, testability, user-friendliness
- [Codebase Architecture](../codebase-architecture.md) - DDD layers and dependency flow
- [Module Organization](../contributing/module-organization.md) - Code organization conventions
- [Error Handling Guide](../contributing/error-handling.md) - Error handling patterns
- [User Output Architecture Improvements](../refactors/plans/user-output-architecture-improvements.md) - Parent refactoring plan
- [Extract Verbosity Filtering Logic (#103)](./103-extract-verbosity-filtering-logic.md) - Related refactoring

## Notes

### Why Arc<UserOutput>?

- **Thread Safety**: `Arc` enables thread-safe shared ownership
- **Cheap Cloning**: Cloning `Arc<UserOutput>` is just incrementing a reference count
- **No Lifetime Annotations**: Avoids complex lifetime annotations in function signatures
- **Future-Proof**: Enables async command handlers without borrowing issues

### Alternative: Pass UserOutput by Reference

We could pass `&UserOutput` instead of `Arc<UserOutput>`, but this would:

- ❌ Require lifetime annotations throughout the call chain
- ❌ Complicate async code (references across await points)
- ❌ Make testing more verbose (lifetime management in test fixtures)

Using `Arc` is the pragmatic choice for this use case.

### Relationship to Issue #103

Issue #103 (Extract Verbosity Filtering Logic) is complementary:

- **This issue (centralization)**: Ensures consistent `UserOutput` configuration
- **Issue #103 (verbosity filtering)**: Improves internal implementation of `UserOutput`

Both can be implemented independently and in any order.

### Future Extensions

Once `UserOutput` is centralized via dependency injection, future enhancements become easier:

- **CLI Verbosity Flags**: Add `-v/--verbose` flags to CLI and pass user-selected verbosity to `Container::new()`
- **Testing**: Inject mock `UserOutput` for capturing messages in tests
- **Output Sinks**: Support different output destinations (files, network)
- **Output Styles**: Switch between emoji, plain text, JSON output
- **Dynamic Configuration**: Change verbosity or style at runtime

These extensions are **NOT** in scope for this issue but are enabled by this architectural change.

**Note**: This task uses `DEFAULT_VERBOSITY` constant. CLI verbosity flags (`-v/--verbose`) will be added in a separate future task that will:

1. Add verbosity flags to `GlobalArgs` in CLI
2. Modify `Container::new()` to accept `VerbosityLevel` parameter
3. Pass CLI-selected verbosity from `app.rs` bootstrap
