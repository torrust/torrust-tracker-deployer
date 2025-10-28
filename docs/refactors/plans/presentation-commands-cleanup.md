# Presentation Commands Module Refactoring

**GitHub EPIC**: [#63](https://github.com/torrust/torrust-tracker-deployer/issues/63)

## 📋 Overview

This refactoring plan addresses code quality, consistency, and maintainability issues in the `src/presentation/commands` module. The goal is to eliminate duplication, improve abstraction, enhance testability, and ensure consistent patterns across all command handlers.

**Target Files:**

- `src/presentation/commands/mod.rs`
- `src/presentation/commands/create/mod.rs`
- `src/presentation/commands/create/subcommand.rs` (372 lines)
- `src/presentation/commands/create/config_loader.rs` (289 lines)
- `src/presentation/commands/create/errors.rs` (250 lines)
- `src/presentation/commands/destroy/mod.rs`
- `src/presentation/commands/destroy/command.rs` (120 lines)
- `src/presentation/commands/destroy/errors.rs` (285 lines)
- Test files in both `create/tests/` and `destroy/tests/`

**Scope:**

- Eliminate code duplication across command handlers
- Create consistent naming conventions
- Extract shared abstractions for common patterns
- Improve function size and complexity (SRP adherence)
- Enhance testability with shared test utilities
- Consolidate hard-coded configuration values

## 📊 Progress Tracking

**Total Active Proposals**: 11
**Total Postponed**: 0
**Total Discarded**: 1 (Proposal 4 merged into Proposal 3)
**Completed**: 0/11
**In Progress**: 0/11
**Not Started**: 11/11

### By Type

- **Quick Wins**: 0/5 completed (0%)
- **Structural Improvements**: 0/4 completed (0%)
- **Advanced Patterns**: 0/2 completed (0%)

### Discarded Proposals

#### Proposal 4: Extract User Output Helper (Merged into Proposal 3)

**Reason for Discard**: This proposal was made redundant by enhancing Proposal 3 to include `UserOutput` in the `CommandContext`. When all commands get `UserOutput` from the context, there's no need for a separate `create_default()` wrapper. The `report_error()` utility function was preserved and integrated into Proposal 3.

### Postponed Proposals

None yet.

## 🎯 Key Problems Identified

### 1. Inconsistent Naming and Structure

**Description**: The `create` module uses `subcommand.rs` while `destroy` uses `command.rs`. Both handle the same kind of operations but use different file naming conventions.

**Impact**: Reduces discoverability and creates cognitive load for developers navigating the codebase.

### 2. Code Duplication

**Description**: Duplicate initialization patterns across commands:

- `UserOutput::new(VerbosityLevel::Normal)` repeated in both handlers
- `RepositoryFactory::new(Duration::from_secs(30))` duplicated
- `Arc::new(SystemClock)` duplicated
- Similar error mapping patterns: `output.error(&error.to_string())`

**Impact**: Violates DRY principle, makes changes harder to maintain, increases risk of inconsistencies.

### 3. Missing Abstractions

**Description**: No shared types for:

- Command dependencies (repository, clock, output)
- User output patterns (progress, success, error reporting)
- Command handler factory for consistent initialization

**Impact**: Forces duplication, reduces testability, makes future commands harder to implement consistently.

### 4. Complex Handler Functions

**Description**: The `handle_environment_creation()` function in `create/subcommand.rs` is 110+ lines and handles multiple responsibilities:

1. User output initialization
2. Configuration loading
3. Repository setup
4. Clock initialization
5. Command handler creation
6. Command execution
7. Success/error reporting

**Impact**: Violates SRP, reduces testability, hard to understand and modify.

### 5. Hard-coded Configuration

**Description**: Magic numbers and fixed values scattered throughout:

- Timeout: `Duration::from_secs(30)` (appears in both commands)
- Verbosity: `VerbosityLevel::Normal` (hard-coded)
- No centralized configuration

**Impact**: Makes behavior changes require code modifications in multiple places.

### 6. Mixed Implementation in Create Subcommand Handler

**Description**: While `handle_create_command()` correctly acts as a dispatcher routing to subcommands (template generation and environment creation are related - templates define environment configurations), the current implementation mixes the subcommand logic with the routing logic in a single file (`subcommand.rs`). Both subcommand implementations (`handle_template_generation()` and `handle_environment_creation()`) are in the same file as the dispatcher.

**Why subcommands**: Template generation and environment creation are grouped under the `create` command because they're highly coupled - the template generates the configuration file needed to create an environment. This is a valid use of subcommands.

**The actual problem**: The lack of clear module separation makes it harder to:

- Navigate the codebase (110+ line environment creation function mixed with 30+ line template generation)
- Test each subcommand independently
- Understand the boundaries between the router and subcommand handlers

**Impact**: Reduces code organization clarity, makes the single file too large (372 lines), and mixes different levels of abstraction.

### 7. Inconsistent Test Structure

**Description**: `create` module has comprehensive test infrastructure (`fixtures.rs`, `template.rs`, `integration.rs`) while `destroy` only has `integration.rs`. No shared test utilities.

**Impact**: Duplication in test setup, inconsistent test coverage, harder to write new tests.

## 🚀 Proposals

---

### Proposal 1: Organize Subcommands in Dedicated Folder Structure and Standardize Module Organization

**GitHub Issue**: [#64](https://github.com/torrust/torrust-tracker-deployer/issues/64)

**Status**: ⏳ Not Started  
**Type**: 🎯 Quick Win  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P0  
**Depends On**: None

#### Problem

##### Part 1: Inconsistent Naming and Structure

The naming inconsistency (`subcommand.rs` vs `command.rs`) exists because commands have different structures:

- **Create command**: Has subcommands (`environment`, `template`) - uses `subcommand.rs`
- **Destroy command**: Single command (no subcommands) - uses `command.rs`

While this reflects the actual structure, the naming doesn't make this distinction clear. Additionally, for commands with subcommands like `create`, the subcommand implementations are mixed in a single file.

```rust
// Current structure - unclear distinction
src/presentation/commands/create/subcommand.rs  // Has subcommands but flat structure
src/presentation/commands/destroy/command.rs    // Single command
```

##### Part 2: No Standardized Module Organization Pattern

Module organization differs between `create` and `destroy` without a documented standard:

```text
create/
  ├── mod.rs              ← Re-exports
  ├── subcommand.rs       ← Main handler (but mixed concerns)
  ├── config_loader.rs    ← Create-specific
  ├── errors.rs
  └── tests/

destroy/
  ├── mod.rs              ← Re-exports
  ├── command.rs          ← Main handler (inconsistent name)
  ├── errors.rs
  └── tests/
```

There's no documented pattern for future commands to follow.

#### Proposed Solution

Create a clear distinction using folder structure and document it as the standard pattern:

**For commands with subcommands** (like `create`):

```rust
src/presentation/commands/create/
  ├── mod.rs                          // Re-exports
  ├── handler.rs                      // Main router that delegates to subcommands
  ├── errors.rs                       // Command-specific errors
  ├── {helpers}.rs                    // Optional command-specific helpers
  ├── subcommands/                    // 🆕 Dedicated subcommands folder
  │   ├── mod.rs                      // Subcommands module
  │   ├── environment.rs              // Environment creation subcommand
  │   └── template.rs                 // Template generation subcommand
  └── tests/
      ├── mod.rs
      ├── integration.rs
      └── {subcommand}_tests.rs
```

**For single commands** (like `destroy`):

```rust
src/presentation/commands/destroy/
  ├── mod.rs                          // Re-exports
  ├── handler.rs                      // Direct implementation (no subcommands)
  ├── errors.rs                       // Command-specific errors
  └── tests/
      ├── mod.rs
      └── integration.rs
```

The `handler.rs` in create would become a simple router:

```rust
// In src/presentation/commands/create/handler.rs

use crate::presentation::cli::commands::CreateAction;
use super::errors::CreateSubcommandError;
use super::subcommands;

/// Handle the create command by routing to appropriate subcommand
pub fn handle_create_command(
    action: CreateAction,
    working_dir: &Path,
) -> Result<(), CreateSubcommandError> {
    match action {
        CreateAction::Environment { env_file } => {
            subcommands::environment::handle(&env_file, working_dir)
        }
        CreateAction::Template { output_path } => {
            let template_path = output_path.unwrap_or_else(CreateAction::default_template_path);
            subcommands::template::handle(&template_path)
        }
    }
}
```

#### Rationale

- **Clarity**: Folder structure makes it immediately obvious which commands have subcommands
- **Scalability**: Easy to add new subcommands to `create` or other commands
- **Consistency**: All command handlers use `handler.rs`, but structure differs based on needs
- **Separation of Concerns**: Each subcommand has its own focused module
- **Discoverability**: Developers can see at a glance which commands are composite vs simple
- **Future-proof**: When `provision` or `configure` commands get subcommands, the pattern is clear
- **Documentation**: Establishes a standard pattern for all future commands
- **Maintainability**: Consistent structure across the codebase reduces cognitive load

#### Benefits

- ✅ Clear visual distinction between simple and composite commands
- ✅ Each subcommand has focused, single-responsibility module
- ✅ Consistent `handler.rs` naming for all commands
- ✅ Easy to add new subcommands without cluttering main files
- ✅ Improved code navigation and discoverability
- ✅ Pattern is documented and clear for future commands with subcommands
- ✅ Predictable structure for all commands
- ✅ Easy to navigate codebase

#### Implementation Checklist

**Phase 1: Restructure destroy command (simple rename):**

- [ ] Rename `src/presentation/commands/destroy/command.rs` to `handler.rs`
- [ ] Update `destroy/mod.rs` to use `pub mod handler;`
- [ ] Update re-exports in `destroy/mod.rs`
- [ ] Verify imports and tests still work

**Phase 2: Restructure create command (with subcommands folder):**

- [ ] Create `src/presentation/commands/create/subcommands/` directory
- [ ] Create `src/presentation/commands/create/subcommands/mod.rs`
- [ ] Extract environment creation to `subcommands/environment.rs`
- [ ] Extract template generation to `subcommands/template.rs`
- [ ] Rename `create/subcommand.rs` to `handler.rs`
- [ ] Refactor `handler.rs` to be a simple router delegating to subcommands
- [ ] Update `create/mod.rs` to include subcommands module
- [ ] Update re-exports in `create/mod.rs`

**Phase 3: Document the standard pattern:**

- [ ] Add module structure patterns to `docs/contributing/module-organization.md`:
  - [ ] Pattern for commands with subcommands (use `subcommands/` folder)
  - [ ] Pattern for simple commands (flat structure with `handler.rs`)
  - [ ] Examples of when to use each pattern
  - [ ] Migration guide for converting simple commands to composite

**Phase 4: Testing and validation:**

- [ ] Update imports in all test files
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues
- [ ] Verify documentation builds correctly

#### Testing Strategy

Run existing tests to ensure no behavioral changes:

```bash
cargo test presentation::commands
cargo doc --no-deps --package torrust-tracker-deployer-lib
```

---

### Proposal 2: Extract Command Configuration Constants

**GitHub Issue**: [#65](https://github.com/torrust/torrust-tracker-deployer/issues/65)

**Status**: ⏳ Not Started  
**Type**: 🎯 Quick Win  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: None

#### Problem

Hard-coded magic numbers scattered across handlers:

```rust
// In create/subcommand.rs
let repository_factory = RepositoryFactory::new(Duration::from_secs(30));  // ❌ Magic number
let mut output = UserOutput::new(VerbosityLevel::Normal);                   // ❌ Hard-coded

// In destroy/command.rs
let repository_factory = RepositoryFactory::new(Duration::from_secs(30));  // ❌ Duplicate
let mut output = UserOutput::new(VerbosityLevel::Normal);                   // ❌ Duplicate
```

#### Proposed Solution

Create a constants module at `src/presentation/commands/constants.rs`:

```rust
//! Constants for command handlers
//!
//! Centralized configuration values used across command handlers.

use std::time::Duration;
use crate::presentation::user_output::VerbosityLevel;

/// Default timeout for file lock operations in repository
pub const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_secs(30);

/// Default verbosity level for user output
pub const DEFAULT_VERBOSITY: VerbosityLevel = VerbosityLevel::Normal;

// Future additions:
// pub const MAX_RETRY_ATTEMPTS: u32 = 3;
// pub const RETRY_DELAY: Duration = Duration::from_secs(5);
```

Then use these constants in handlers:

```rust
use crate::presentation::commands::constants::{DEFAULT_LOCK_TIMEOUT, DEFAULT_VERBOSITY};

let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);  // ✅ Clear
let mut output = UserOutput::new(DEFAULT_VERBOSITY);                     // ✅ Clear
```

#### Rationale

- **Single Source of Truth**: All configuration in one place
- **Maintainability**: Changes to timeouts/verbosity only need one edit
- **Discoverability**: Developers know where to find configuration values
- **Documentation**: Constants can be documented with why these values were chosen

#### Benefits

- ✅ Eliminates duplicate magic numbers
- ✅ Makes configuration explicit and discoverable
- ✅ Easier to adjust behavior without code changes (future: move to config file)
- ✅ Better documentation for why specific values are used

#### Implementation Checklist

- [ ] Create `src/presentation/commands/constants.rs`
- [ ] Define `DEFAULT_LOCK_TIMEOUT` constant
- [ ] Define `DEFAULT_VERBOSITY` constant
- [ ] Add module documentation explaining purpose
- [ ] Update `src/presentation/commands/mod.rs` to include `pub mod constants;`
- [ ] Replace `Duration::from_secs(30)` in create handler with constant
- [ ] Replace `Duration::from_secs(30)` in destroy handler with constant
- [ ] Replace `VerbosityLevel::Normal` in create handler with constant
- [ ] Replace `VerbosityLevel::Normal` in destroy handler with constant
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

No behavioral changes, but verify:

```bash
cargo test presentation::commands
cargo clippy -- -D warnings
```

#### Future Work Note

> **Note**: This proposal is a tactical fix to eliminate duplicate magic numbers. However, the broader issue of dependency injection for command handlers should be addressed in a future refactoring.
>
> Currently, each handler manually constructs its dependencies (repository, clock, output), which creates duplication and coupling. The project has a dependency injection container at `src/bootstrap/container.rs`, but it's not yet integrated with command handlers because:
>
> 1. We don't know upfront which services each command will need
> 2. The container would need to lazily initialize services on-demand (possibly using `Option` types)
> 3. We're waiting to have more commands implemented before designing a comprehensive solution
>
> This deserves its own refactor plan once we have more commands and can identify common patterns. See Proposal 3 for the intermediate step (CommandContext struct), and consider creating a separate refactor plan for full dependency injection integration after implementing a few more commands.

---

### Proposal 3: Extract Shared Command Dependencies and Output (Merged with former Proposal 4)

**GitHub Issue**: [#66](https://github.com/torrust/torrust-tracker-deployer/issues/66)

**Status**: ⏳ Not Started  
**Type**: 🎯 Quick Win  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P0  
**Depends On**: Proposal 2

#### Problem

Both command handlers manually create the same set of dependencies with duplicate code:

```rust
// In create handler
let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
let repository = repository_factory.create(working_dir.to_path_buf());
let clock: Arc<dyn Clock> = Arc::new(SystemClock);
let mut output = UserOutput::new(VerbosityLevel::Normal);  // Also duplicated!

// In destroy handler (exact duplicate)
let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
let repository = repository_factory.create(working_dir.to_path_buf());
let clock = Arc::new(crate::shared::SystemClock);
let mut output = UserOutput::new(VerbosityLevel::Normal);  // Also duplicated!
```

#### Proposed Solution

Create a `CommandContext` struct in `src/presentation/commands/context.rs` that includes **all** shared dependencies including `UserOutput`:

````rust
//! Command Execution Context
//!
//! Provides shared dependencies for all command handlers.

use std::path::PathBuf;
use std::sync::Arc;

use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::infrastructure::persistence::EnvironmentRepository;
use crate::presentation::user_output::UserOutput;
use crate::shared::{Clock, SystemClock};

use super::constants::{DEFAULT_LOCK_TIMEOUT, DEFAULT_VERBOSITY};

/// Context containing shared dependencies for command execution
///
/// This struct provides a consistent way to initialize and pass
/// dependencies to command handlers, reducing duplication and
/// ensuring consistent configuration across all commands.
pub struct CommandContext {
    /// Environment repository for persistence
    repository: Arc<dyn EnvironmentRepository>,

    /// Clock for timing operations
    clock: Arc<dyn Clock>,

    /// User output for consistent messaging
    output: UserOutput,
}

impl CommandContext {
    /// Create a new command context with default dependencies
    ///
    /// # Arguments
    ///
    /// * `working_dir` - Base directory for environment data storage
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::path::Path;
    /// use torrust_tracker_deployer_lib::presentation::commands::CommandContext;
    ///
    /// let mut context = CommandContext::new(Path::new("."));
    /// context.output().progress("Starting operation...");
    /// ```
    pub fn new(working_dir: PathBuf) -> Self {
        let repository_factory = RepositoryFactory::new(DEFAULT_LOCK_TIMEOUT);
        let repository = repository_factory.create(working_dir);
        let clock = Arc::new(SystemClock);
        let output = UserOutput::new(DEFAULT_VERBOSITY);

        Self { repository, clock, output }
    }

    /// Get a reference to the environment repository
    pub fn repository(&self) -> &Arc<dyn EnvironmentRepository> {
        &self.repository
    }

    /// Get a reference to the clock
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Get a mutable reference to the user output
    pub fn output(&mut self) -> &mut UserOutput {
        &mut self.output
    }
}

#[cfg(test)]
impl CommandContext {
    /// Create a test context with mock dependencies
    pub fn new_for_testing(
        repository: Arc<dyn EnvironmentRepository>,
        clock: Arc<dyn Clock>,
        output: UserOutput,
    ) -> Self {
        Self { repository, clock, output }
    }
}

/// Report an error to the user with standardized formatting
///
/// This utility function provides consistent error reporting across
/// all command handlers.
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::commands::context;
///
/// let mut ctx = context::CommandContext::new(PathBuf::from("."));
/// let error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
/// context::report_error(ctx.output(), &error);
/// ```
pub fn report_error(output: &mut UserOutput, error: &dyn std::error::Error) {
    output.error(&error.to_string());
}
````

Then use it in handlers:

```rust
// In create handler - much cleaner!
let mut context = CommandContext::new(working_dir.to_path_buf());

context.output().progress("Loading configuration...");

let command_handler = CreateCommandHandler::new(
    context.repository().clone(),
    context.clock().clone(),
);

// Use the context's output throughout
context.output().success("Environment created!");

// In destroy handler - same pattern!
let mut context = CommandContext::new(working_dir.to_path_buf());

context.output().progress("Destroying environment...");

let command_handler = DestroyCommandHandler::new(
    context.repository().clone(),
    context.clock().clone(),
);
```

#### Rationale

- **DRY**: Eliminates duplicate dependency initialization (repository, clock, **and output**)
- **Consistency**: All commands use same dependency setup and output instance
- **Testability**: Single place to inject mocks for testing
- **Maintainability**: Adding new dependencies only requires updating one struct
- **Clarity**: Makes dependencies explicit and discoverable
- **Single Output Instance**: All commands use the same `UserOutput`, ensuring consistent verbosity and formatting

#### Benefits

- ✅ Eliminates 8+ lines of duplicate code per command handler
- ✅ Makes testing easier with `new_for_testing()` constructor
- ✅ Future commands automatically get consistent dependency setup
- ✅ Single place to modify if dependency initialization changes
- ✅ No need for separate output helper module - it's built into the context
- ✅ `report_error()` utility still available for consistent error formatting
- ✅ All output goes through the same instance, ensuring consistency

#### Implementation Checklist

- [ ] Create `src/presentation/commands/context.rs`
- [ ] Define `CommandContext` struct with repository, clock, **and output**
- [ ] Implement `new()` constructor using constants for all three dependencies
- [ ] Add accessor methods `repository()`, `clock()`, and `output()`
- [ ] Add `new_for_testing()` for test support (include output parameter)
- [ ] Add `report_error()` utility function for consistent error formatting
- [ ] Add comprehensive documentation with examples
- [ ] Update `src/presentation/commands/mod.rs` to include `pub mod context;`
- [ ] Update create handler to use `CommandContext::new()` and `context.output()`
- [ ] Update destroy handler to use `CommandContext::new()` and `context.output()`
- [ ] Remove duplicate initialization code from both handlers (repository, clock, **and output**)
- [ ] Update tests to verify same behavior
- [ ] Run all tests to ensure no regressions
- [ ] Run linter and fix any issues

**Note**: This proposal merges the former Proposal 4's functionality. By including `UserOutput` in the context, we eliminate the need for a separate `create_default()` wrapper while keeping the useful `report_error()` utility.

#### Testing Strategy

Create unit tests for `CommandContext`:

```rust
#[test]
fn it_should_create_context_with_all_dependencies() {
    let temp_dir = TempDir::new().unwrap();
    let mut context = CommandContext::new(temp_dir.path().to_path_buf());

    assert!(Arc::strong_count(context.repository()) >= 1);
    assert!(Arc::strong_count(context.clock()) >= 1);

    // Verify output is accessible and functional
    context.output().progress("Test message");
}

#[test]
fn it_should_provide_consistent_output_instance() {
    let temp_dir = TempDir::new().unwrap();
    let mut context = CommandContext::new(temp_dir.path().to_path_buf());

    // All output calls use the same instance
    context.output().progress("Step 1");
    context.output().success("Step 2");
}
```

Test the `report_error()` utility:

```rust
#[test]
fn it_should_report_errors_consistently() {
    let mut context = CommandContext::new(PathBuf::from("."));
    let error = std::io::Error::new(std::io::ErrorKind::NotFound, "test error");

    // Should not panic
    context::report_error(context.output(), &error);
}
```

Verify existing command tests still pass.

---

### Proposal 4: Split Template Generation from Environment Creation

**GitHub Issue**: [#67](https://github.com/torrust/torrust-tracker-deployer/issues/67)

**Status**: ⏳ Not Started  
**Type**: 🎯 Quick Win  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P0  
**Depends On**: Proposal 1

#### Problem

The `handle_create_command()` function handles two completely different concerns:

```rust
pub fn handle_create_command(action: CreateAction, working_dir: &Path)
    -> Result<(), CreateSubcommandError>
{
    match action {
        CreateAction::Environment { env_file } => {
            // Complex domain operation (100+ lines)
            handle_environment_creation(&env_file, working_dir)
        }
        CreateAction::Template { output_path } => {
            // Simple file operation (30 lines)
            handle_template_generation(&template_path)
        }
    }
}
```

This makes the file large and mixes different levels of abstraction (routing vs implementation).

#### Proposed Solution

Separate the subcommand implementations into their own modules while keeping the simple router in `handler.rs`:

**Move template subcommand to** `subcommands/template.rs`:

```rust
// In src/presentation/commands/create/subcommands/template.rs

//! Template Generation Subcommand
//!
//! Handles generation of environment configuration templates.

use std::path::Path;
use crate::domain::config::EnvironmentCreationConfig;
use crate::presentation::user_output::{UserOutput, VerbosityLevel};
use super::super::errors::CreateSubcommandError;
use super::super::super::constants::DEFAULT_VERBOSITY;

/// Generate a configuration template file
///
/// Creates a template JSON file with placeholder values that users
/// can edit to create their environment configurations.
///
/// # Arguments
///
/// * `output_path` - Path where the template file should be created
///
/// # Returns
///
/// Returns `Ok(())` on success, or a `CreateSubcommandError` on failure.
pub fn handle(output_path: &Path)
    -> Result<(), CreateSubcommandError>
{
    let mut output = UserOutput::new(DEFAULT_VERBOSITY);

    output.progress("Generating configuration template...");

    // Generate template (async operation)
    tokio::runtime::Runtime::new()
        .expect("Failed to create tokio runtime")
        .block_on(async {
            EnvironmentCreationConfig::generate_template_file(output_path)
                .await
                .map_err(CreateSubcommandError::TemplateGenerationFailed)
        })?;

    display_success_message(&mut output, output_path);

    Ok(())
}

fn display_success_message(output: &mut UserOutput, output_path: &Path) {
    output.success(&format!(
        "Configuration template generated: {}",
        output_path.display()
    ));

    println!();
    println!("Next steps:");
    println!("1. Edit the template file and replace placeholder values:");
    println!("   - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name");
    println!("   - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key");
    println!("   - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key");
    println!("2. Review default values (username: 'torrust', port: 22)");
    println!("3. Create the environment:");
    println!("   torrust-tracker-deployer create environment --env-file {}",
        output_path.display());
}
```

Similarly, split environment creation into `environment_handler.rs`:

```rust
// In src/presentation/commands/create/environment_handler.rs

//! Environment Creation Handler
//!
//! Handles creation of deployment environments from configuration files.

use std::path::Path;
// ... imports ...

/// Handle environment creation from configuration file
pub fn handle_environment_creation(
    env_file: &Path,
    working_dir: &Path,
) -> Result<(), CreateSubcommandError> {
    // Current implementation from subcommand.rs
    // Will be further refactored in Phase 1
}
```

Update the router in `handler.rs`:

```rust
// In src/presentation/commands/create/handler.rs (formerly subcommand.rs)

use super::environment_handler;
use super::template_handler;

pub fn handle_create_command(
    action: CreateAction,
    working_dir: &Path,
) -> Result<(), CreateSubcommandError> {
    match action {
        CreateAction::Environment { env_file } => {
            environment_handler::handle_environment_creation(&env_file, working_dir)
        }
        CreateAction::Template { output_path } => {
            let template_path = output_path.unwrap_or_else(CreateAction::default_template_path);
            template_handler::handle_template_generation(&template_path)
        }
    }
}
```

#### Rationale

- **SRP**: Each module has one clear responsibility
- **Clarity**: Function names clearly indicate what they do
- **Maintainability**: Changes to template generation don't affect environment creation
- **Testability**: Each concern can be tested independently
- **Organization**: Follows module organization conventions (see docs/contributing/module-organization.md)

#### Benefits

- ✅ Router stays focused on dispatching (single responsibility)
- ✅ Each subcommand has its own focused module
- ✅ Easier to test each subcommand independently
- ✅ Aligns with Proposal 1's subcommands folder structure
- ✅ Reduces file size from 372 lines to manageable, focused modules
- ✅ Prepares for future refactoring of environment creation (Proposal 5)

#### Implementation Checklist

- [ ] Create `src/presentation/commands/create/subcommands/` directory (if not done in Proposal 1)
- [ ] Create `src/presentation/commands/create/subcommands/mod.rs`
- [ ] Extract template generation to `subcommands/template.rs` with `handle()` function
- [ ] Extract environment creation to `subcommands/environment.rs` with `handle()` function
- [ ] Extract `display_success_message()` helper to template subcommand
- [ ] Add comprehensive documentation to each subcommand module
- [ ] Update `create/mod.rs` to include subcommands module
- [ ] Update `handler.rs` to import and delegate to `subcommands::template::handle()` and `subcommands::environment::handle()`
- [ ] Update re-exports if needed
- [ ] Move or duplicate relevant tests to subcommand modules
- [ ] Verify all tests pass
- [ ] Run linter and fix issues

**Note**: This proposal uses the `subcommands/` folder structure from Proposal 1, ensuring consistency with the standardized module structure.

#### Testing Strategy

Create unit tests for each subcommand:

```bash
cargo test create::subcommands::template
cargo test create::subcommands::environment
cargo test create::handler  # Router tests
```

#### Results (if completed)

TBD

---

### Proposal 5: Eliminate Direct Console Output Bypassing UserOutput

**GitHub Issue**: [#68](https://github.com/torrust/torrust-tracker-deployer/issues/68)

**Status**: ⏳ Not Started  
**Type**: 🎯 Quick Win  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P0  
**Depends On**: Proposal 3 (CommandContext with UserOutput)

#### Problem

The presentation layer directly calls `println!()` and other console output macros, bypassing the `UserOutput` service abstraction:

```rust
// In template generation subcommand - WRONG!
fn display_success_message(output: &mut UserOutput, output_path: &Path) {
    output.success(&format!(
        "Configuration template generated: {}",
        output_path.display()
    ));

    println!();  // ❌ Bypasses UserOutput
    println!("Next steps:");  // ❌ Bypasses UserOutput
    println!("1. Edit the template file and replace placeholder values:");  // ❌ Bypasses UserOutput
    println!("   - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name");  // ❌ Bypasses UserOutput
    println!("   - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key");  // ❌ Bypasses UserOutput
    println!("   - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key");  // ❌ Bypasses UserOutput
    println!("2. Review default values (username: 'torrust', port: 22)");  // ❌ Bypasses UserOutput
    println!("3. Create the environment:");  // ❌ Bypasses UserOutput
    println!("   torrust-tracker-deployer create environment --env-file {}",  // ❌ Bypasses UserOutput
        output_path.display());
}
```

**Why this is a problem**:

- **Violates abstraction**: The `UserOutput` service exists to centralize and control all user-facing output
- **Inconsistent formatting**: Direct `println!()` calls bypass output formatting rules
- **No verbosity control**: Can't suppress or filter these messages based on verbosity level
- **Hard to test**: Can't capture or verify output in tests
- **No output redirection**: Can't redirect output to files, different streams, or alternative formats
- **Maintainability**: Changes to output format require finding all scattered `println!()` calls

#### Proposed Solution

**Option 1: Extend UserOutput with structured message methods** (Recommended)

Add methods to `UserOutput` for complex, multi-line messages:

```rust
// In src/presentation/user_output.rs

impl UserOutput {
    /// Display a multi-line information block
    pub fn info_block(&mut self, title: &str, lines: &[&str]) {
        if self.verbosity >= VerbosityLevel::Normal {
            println!();
            println!("{}", title);
            for line in lines {
                println!("{}", line);
            }
        }
    }

    /// Display a numbered list of steps
    pub fn steps(&mut self, title: &str, steps: &[&str]) {
        if self.verbosity >= VerbosityLevel::Normal {
            println!();
            println!("{}", title);
            for (idx, step) in steps.iter().enumerate() {
                println!("{}. {}", idx + 1, step);
            }
        }
    }

    /// Display a blank line (for spacing)
    pub fn blank_line(&mut self) {
        if self.verbosity >= VerbosityLevel::Normal {
            println!();
        }
    }
}
```

Then refactor the template handler:

```rust
fn display_success_message(output: &mut UserOutput, output_path: &Path) {
    output.success(&format!(
        "Configuration template generated: {}",
        output_path.display()
    ));

    output.blank_line();

    output.steps("Next steps:", &[
        "Edit the template file and replace placeholder values:\n\
         - REPLACE_WITH_ENVIRONMENT_NAME: Choose a unique environment name\n\
         - REPLACE_WITH_SSH_PRIVATE_KEY_PATH: Path to your SSH private key\n\
         - REPLACE_WITH_SSH_PUBLIC_KEY_PATH: Path to your SSH public key",
        "Review default values (username: 'torrust', port: 22)",
        &format!("Create the environment:\n\
                  torrust-tracker-deployer create environment --env-file {}",
                 output_path.display()),
    ]);
}
```

**Option 2: Create view components for complex output** (Future enhancement)

For very complex output (tables, multi-section help), create dedicated view components:

```rust
// In src/presentation/views/next_steps.rs

pub struct NextStepsView {
    template_path: PathBuf,
}

impl NextStepsView {
    pub fn new(template_path: PathBuf) -> Self {
        Self { template_path }
    }

    pub fn render(&self, output: &mut UserOutput) {
        output.blank_line();
        output.steps("Next steps:", &[
            "Edit the template file...",
            "Review default values...",
            &format!("Create environment: {}", self.template_path.display()),
        ]);
    }
}
```

#### Rationale

- **Single Responsibility**: `UserOutput` is the single point for all user-facing output
- **Consistent Control**: All output respects verbosity levels, formatting rules, and redirection
- **Testability**: Can mock or capture output for testing
- **Maintainability**: Output format changes happen in one place
- **Extensibility**: Easy to add new output formats (JSON, structured logging, etc.)
- **Abstraction**: Presentation layer doesn't know about console specifics

#### Benefits

- ✅ All output goes through `UserOutput` service
- ✅ Consistent formatting and verbosity control
- ✅ Testable output (can verify messages in tests)
- ✅ Supports future output redirection (files, JSON, etc.)
- ✅ Easy to change output format globally
- ✅ Aligns with project architecture principles

#### Implementation Checklist

##### Phase 1: Extend UserOutput API

- [ ] Add `blank_line()` method to `UserOutput`
- [ ] Add `steps()` method for numbered lists
- [ ] Add `info_block()` method for multi-line info blocks
- [ ] Add tests for new methods
- [ ] Document new methods with examples

##### Phase 2: Refactor template subcommand

- [ ] Identify all `println!()` calls in `create/subcommands/template.rs`
- [ ] Replace with appropriate `UserOutput` methods
- [ ] Verify output format is preserved
- [ ] Update tests to verify output

##### Phase 3: Audit all presentation layer files

- [ ] Search entire `src/presentation/` for `println!()`, `print!()`, `eprintln!()`, `eprint!()`
- [ ] Create inventory of direct output calls
- [ ] Replace each with appropriate `UserOutput` method
- [ ] Add new `UserOutput` methods as needed

##### Phase 4: Validation

- [ ] Run all tests to ensure output is correct
- [ ] Manual testing with different verbosity levels
- [ ] Verify output redirection works (if implemented)
- [ ] Run linter and fix issues

#### Testing Strategy

**Unit tests for new UserOutput methods**:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_display_steps_with_numbering() {
        let mut output = UserOutput::new(VerbosityLevel::Normal);

        // Capture output somehow (need to add capture mechanism)
        output.steps("Test steps:", &["First step", "Second step"]);

        // Verify output contains "1. First step" and "2. Second step"
    }

    #[test]
    fn it_should_suppress_steps_when_quiet() {
        let mut output = UserOutput::new(VerbosityLevel::Quiet);

        output.steps("Test steps:", &["First step"]);

        // Verify no output
    }
}
```

**Integration tests**:

```bash
cargo test presentation::commands::create::subcommands::template
```

#### Future Enhancements

- Add output capture mechanism for testing
- Create view components for very complex output (tables, etc.)
- Support alternative output formats (JSON, YAML)
- Add output redirection to files

---

### Proposal 6: Refactor `handle_environment_creation()` Using Steps Pattern

**GitHub Issue**: [#69](https://github.com/torrust/torrust-tracker-deployer/issues/69)

**Status**: ⏳ Not Started  
**Type**: 🔨 Structural Improvement  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵🔵 High  
**Priority**: P1  
**Depends On**: Proposal 4

#### Problem

The `handle_environment_creation()` function is 110+ lines and does too much:

```rust
fn handle_environment_creation(env_file: &Path, working_dir: &Path)
    -> Result<(), CreateSubcommandError>
{
    // 1. Create user output
    // 2. Load configuration
    // 3. Validate configuration
    // 4. Create repository
    // 5. Create clock
    // 6. Create command handler
    // 7. Execute command
    // 8. Display results
    // All in one function! 110+ lines
}
```

This violates SRP and makes the function hard to test and understand.

#### Proposed Solution

Break down into focused step functions:

```rust
//! Environment Creation Handler
//!
//! Orchestrates environment creation through discrete steps.

use std::path::Path;
use crate::application::command_handlers::create::CreateCommandHandler;
use crate::domain::config::EnvironmentCreationConfig;
use crate::domain::environment::Environment;
use crate::presentation::user_output::UserOutput;
use super::config_loader::ConfigLoader;
use super::errors::CreateSubcommandError;
use super::super::context::CommandContext;
use super::super::output;

/// Handle environment creation from configuration file
///
/// Orchestrates the environment creation workflow by coordinating
/// discrete steps with clear responsibilities.
pub fn handle_environment_creation(
    env_file: &Path,
    working_dir: &Path,
) -> Result<(), CreateSubcommandError> {
    let mut output = output::create_default();

    // Step 1: Load configuration
    let config = load_configuration(&mut output, env_file)?;

    // Step 2: Create dependencies
    let context = CommandContext::new(working_dir.to_path_buf());

    // Step 3: Execute command
    let environment = execute_create_command(&mut output, config, &context)?;

    // Step 4: Display results
    display_creation_results(&mut output, &environment);

    Ok(())
}

/// Load and validate configuration from file
fn load_configuration(
    output: &mut UserOutput,
    env_file: &Path,
) -> Result<EnvironmentCreationConfig, CreateSubcommandError> {
    output.progress(&format!(
        "Loading configuration from '{}'...",
        env_file.display()
    ));

    let loader = ConfigLoader;
    loader.load_from_file(env_file).inspect_err(|err| {
        output::report_error(output, err);
    })
}

/// Execute the create command handler
fn execute_create_command(
    output: &mut UserOutput,
    config: EnvironmentCreationConfig,
    context: &CommandContext,
) -> Result<Environment, CreateSubcommandError> {
    output.progress(&format!(
        "Creating environment '{}'...",
        config.environment.name
    ));

    let command_handler = CreateCommandHandler::new(
        context.repository().clone(),
        context.clock().clone(),
    );

    output.progress("Validating configuration and creating environment...");

    command_handler.execute(config).map_err(|err| {
        let error = CreateSubcommandError::CommandFailed(err);
        output::report_error(output, &error);
        error
    })
}

/// Display successful creation results
fn display_creation_results(output: &mut UserOutput, environment: &Environment) {
    output.success(&format!(
        "Environment '{}' created successfully",
        environment.name().as_str()
    ));

    output.result(&format!(
        "Instance name: {}",
        environment.instance_name().as_str()
    ));
    output.result(&format!(
        "Data directory: {}",
        environment.data_dir().display()
    ));
    output.result(&format!(
        "Build directory: {}",
        environment.build_dir().display()
    ));
}
```

#### Rationale

- **SRP**: Each function has one clear responsibility
- **Readability**: Main flow is clear, details are hidden in focused functions
- **Testability**: Each step can be tested independently
- **Maintainability**: Changes to one step don't affect others
- **Organization**: Follows top-down organization with helpers at bottom

#### Benefits

- ✅ Main function reduced from 110+ to ~25 lines
- ✅ Each step is self-contained and testable
- ✅ Clear orchestration flow
- ✅ Easier to modify individual steps
- ✅ Better error handling with clear error context

#### Implementation Checklist

- [ ] Create helper function `load_configuration()`
- [ ] Create helper function `execute_create_command()`
- [ ] Create helper function `display_creation_results()`
- [ ] Refactor main `handle_environment_creation()` to orchestrate steps
- [ ] Add comprehensive documentation to each function
- [ ] Write unit tests for each helper function
- [ ] Update integration tests to verify end-to-end behavior
- [ ] Verify all existing tests still pass
- [ ] Run linter and fix issues

#### Testing Strategy

Test each step independently:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn it_should_load_valid_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());
        let mut output = output::create_default();

        let result = load_configuration(&mut output, &config_path);
        assert!(result.is_ok());
    }

    #[test]
    fn it_should_reject_invalid_configuration() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_invalid_config(temp_dir.path());
        let mut output = output::create_default();

        let result = load_configuration(&mut output, &config_path);
        assert!(result.is_err());
    }

    // More tests for execute_create_command, display_creation_results...
}
```

---

### Proposal 7: Create Shared Test Utilities Module

**GitHub Issue**: [#70](https://github.com/torrust/torrust-tracker-deployer/issues/70)

**Status**: ⏳ Not Started  
**Type**: 🔨 Structural Improvement  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: None

#### Problem

Test utilities are duplicated and inconsistent:

- `create/tests/` has comprehensive fixtures module
- `destroy/tests/` lacks test helpers
- Common patterns like creating temp directories, generating test configs repeated

#### Proposed Solution

Create shared test utilities at `src/presentation/commands/tests/mod.rs`:

```rust
//! Shared test utilities for command handlers
//!
//! Provides common test helpers, fixtures, and utilities used across
//! all command handler tests.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;

use crate::infrastructure::persistence::repository_factory::RepositoryFactory;
use crate::shared::{Clock, MockClock};
use crate::testing::MockEnvironmentRepository;

/// Test context with temp directory and common test dependencies
pub struct TestContext {
    _temp_dir: TempDir,
    pub working_dir: PathBuf,
    pub repository: Arc<MockEnvironmentRepository>,
    pub clock: Arc<MockClock>,
}

impl TestContext {
    /// Create a new test context with fresh temp directory
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let working_dir = temp_dir.path().to_path_buf();

        let repository_factory = RepositoryFactory::new(std::time::Duration::from_secs(30));
        let repository = Arc::new(repository_factory.create(working_dir.clone()));

        let clock = Arc::new(MockClock::new(chrono::Utc::now()));

        Self {
            _temp_dir: temp_dir,
            working_dir,
            repository,
            clock,
        }
    }

    /// Get the working directory path
    pub fn working_dir(&self) -> &Path {
        &self.working_dir
    }
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a valid test configuration file
pub fn create_test_config(path: &Path, env_name: &str) -> PathBuf {
    let config_path = path.join("test-config.json");

    let project_root = env!("CARGO_MANIFEST_DIR");
    let private_key_path = format!("{project_root}/fixtures/testing_rsa");
    let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

    let config_json = format!(
        r#"{{
            "environment": {{
                "name": "{env_name}"
            }},
            "ssh_credentials": {{
                "private_key_path": "{private_key_path}",
                "public_key_path": "{public_key_path}"
            }}
        }}"#
    );

    std::fs::write(&config_path, config_json)
        .expect("Failed to write test config");

    config_path
}

/// Create an invalid JSON configuration file for error testing
pub fn create_invalid_json_config(path: &Path) -> PathBuf {
    let config_path = path.join("invalid.json");
    std::fs::write(&config_path, r#"{"invalid json"#)
        .expect("Failed to write invalid config");
    config_path
}

/// Create a configuration with invalid environment name
pub fn create_config_with_invalid_name(path: &Path) -> PathBuf {
    let config_path = path.join("invalid-name.json");

    let project_root = env!("CARGO_MANIFEST_DIR");
    let private_key_path = format!("{project_root}/fixtures/testing_rsa");
    let public_key_path = format!("{project_root}/fixtures/testing_rsa.pub");

    let config_json = format!(
        r#"{{
            "environment": {{
                "name": "invalid_name_with_underscores"
            }},
            "ssh_credentials": {{
                "private_key_path": "{private_key_path}",
                "public_key_path": "{public_key_path}"
            }}
        }}"#
    );

    std::fs::write(&config_path, config_json)
        .expect("Failed to write test config");

    config_path
}
```

Then use in tests:

```rust
// In create/tests/integration.rs
use crate::presentation::commands::tests::{TestContext, create_test_config};

#[test]
fn it_should_create_environment_from_valid_config() {
    let context = TestContext::new();
    let config_path = create_test_config(context.working_dir(), "test-env");

    let result = handle_environment_creation(&config_path, context.working_dir());
    assert!(result.is_ok());
}
```

#### Rationale

- **DRY**: Eliminates duplicate test utilities
- **Consistency**: Same helpers for all command tests
- **Maintainability**: Changes to test patterns happen in one place
- **Discoverability**: New tests know where to find utilities

#### Benefits

- ✅ Reduces test code duplication
- ✅ Consistent test patterns across commands
- ✅ Easier to write new tests
- ✅ Shared improvements benefit all tests

#### Implementation Checklist

- [ ] Create `src/presentation/commands/tests/mod.rs`
- [ ] Implement `TestContext` struct with temp dir management
- [ ] Migrate `create_test_config()` from fixtures.rs
- [ ] Migrate `create_invalid_json_config()` from fixtures.rs
- [ ] Migrate `create_config_with_invalid_name()` from fixtures.rs
- [ ] Add comprehensive documentation
- [ ] Update `create/tests/` to use shared utilities
- [ ] Update `destroy/tests/` to use shared utilities
- [ ] Remove duplicate code from old test files
- [ ] Verify all tests pass
- [ ] Run linter

#### Testing Strategy

Run all presentation command tests:

```bash
cargo test presentation::commands
```

---

### Proposal 8: Improve Error Consistency Between Commands

**GitHub Issue**: [#71](https://github.com/torrust/torrust-tracker-deployer/issues/71)

**Status**: ⏳ Not Started  
**Type**: 🔨 Structural Improvement  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: None

#### Problem

Error structures differ between commands without clear reason:

```rust
// Create errors
pub enum CreateSubcommandError {
    ConfigFileNotFound { path: PathBuf },
    ConfigParsingFailed { path: PathBuf, format: ConfigFormat, source: ... },
    ConfigValidationFailed(CreateConfigError),
    CommandFailed(CreateCommandHandlerError),
    TemplateGenerationFailed(CreateConfigError),
}

// Destroy errors
pub enum DestroySubcommandError {
    InvalidEnvironmentName { name: String, source: ... },
    EnvironmentNotAccessible { name: String, data_dir: String },
    DestroyOperationFailed { name: String, source: ... },
    RepositoryAccessFailed { data_dir: String, reason: String },
}
```

Some use structured fields consistently, others mix approaches.

#### Proposed Solution

Establish consistent patterns:

1. **Always include relevant context** (paths, names, etc.)
2. **Always use `#[source]` for error chains**
3. **Always include brief tips in error messages**
4. **Group related error types** consistently

Standardize create errors:

```rust
#[derive(Debug, Error)]
pub enum CreateSubcommandError {
    // File-related errors
    #[error("Configuration file not found: {path}
Tip: Check that the file path is correct: ls -la {path}")]
    ConfigFileNotFound {
        path: PathBuf,
    },

    #[error("Failed to parse configuration file '{path}' as {format}: {source}
Tip: Validate JSON syntax with: jq . < {path}")]
    ConfigParsingFailed {
        path: PathBuf,
        format: ConfigFormat,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    // Validation errors
    #[error("Configuration validation failed: {source}
Tip: Check required fields and value formats")]
    ConfigValidationFailed {
        #[source]
        source: CreateConfigError,
    },

    // Execution errors
    #[error("Create command execution failed: {source}
Tip: Run with --log-output file-and-stderr for detailed logs")]
    CommandFailed {
        #[source]
        source: CreateCommandHandlerError,
    },

    // Template errors
    #[error("Template generation failed: {source}
Tip: Check output directory permissions and disk space")]
    TemplateGenerationFailed {
        #[source]
        source: CreateConfigError,
    },
}
```

#### Rationale

- **User Experience**: Consistent error messages across commands
- **Maintainability**: Same patterns make errors easier to understand
- **Traceability**: Proper source chains support debugging
- **Actionability**: Brief tips in messages, detailed help in `.help()`

#### Benefits

- ✅ Consistent error experience
- ✅ Better error chains for debugging
- ✅ Actionable guidance in all errors
- ✅ Easier to add new error types

#### Implementation Checklist

- [ ] Review all error variants in `create/errors.rs`
- [ ] Ensure all errors have structured context
- [ ] Add brief tips to all `#[error]` messages
- [ ] Verify `.help()` provides detailed guidance
- [ ] Review all error variants in `destroy/errors.rs`
- [ ] Apply same standards to destroy errors
- [ ] Update error creation sites to use structured fields
- [ ] Add tests for error display messages
- [ ] Add tests for error help text
- [ ] Run all tests
- [ ] Run linter

#### Testing Strategy

Add error message tests:

```rust
#[test]
fn it_should_display_actionable_error_messages() {
    let error = CreateSubcommandError::ConfigFileNotFound {
        path: PathBuf::from("missing.json"),
    };

    let message = error.to_string();
    assert!(message.contains("Configuration file not found"));
    assert!(message.contains("Tip:"));
    assert!(message.contains("missing.json"));
}

#[test]
fn it_should_provide_detailed_help() {
    let error = CreateSubcommandError::ConfigFileNotFound {
        path: PathBuf::from("missing.json"),
    };

    let help = error.help();
    assert!(help.contains("Troubleshooting"));
    assert!(help.len() > 100); // Substantial guidance
}
```

---

### Proposal 9: Create Command Handler Factory

**GitHub Issue**: [#72](https://github.com/torrust/torrust-tracker-deployer/issues/72)

**Status**: ⏳ Not Started  
**Type**: 🔬 Advanced Pattern  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵🔵 High  
**Priority**: P2  
**Depends On**: Proposal 3

#### Problem

Command handler creation is still somewhat manual:

```rust
let context = CommandContext::new(working_dir.to_path_buf());
let command_handler = CreateCommandHandler::new(
    context.repository().clone(),
    context.clock().clone(),
);
```

This pattern will be repeated for every command we add.

#### Proposed Solution

Create a factory in `src/presentation/commands/factory.rs`:

```rust
//! Command Handler Factory
//!
//! Provides consistent creation of command handlers with injected dependencies.

use std::sync::Arc;

use crate::application::command_handlers::{
    create::CreateCommandHandler,
    destroy::DestroyCommandHandler,
};
use super::context::CommandContext;

/// Factory for creating command handlers with consistent dependencies
pub struct CommandHandlerFactory {
    context: Arc<CommandContext>,
}

impl CommandHandlerFactory {
    /// Create a new factory with the given context
    pub fn new(context: Arc<CommandContext>) -> Self {
        Self { context }
    }

    /// Create a create command handler
    pub fn create_create_handler(&self) -> CreateCommandHandler {
        CreateCommandHandler::new(
            self.context.repository().clone(),
            self.context.clock().clone(),
        )
    }

    /// Create a destroy command handler
    pub fn create_destroy_handler(&self) -> DestroyCommandHandler {
        DestroyCommandHandler::new(
            self.context.repository().clone(),
            self.context.clock().clone(),
        )
    }
}
```

Usage becomes simpler:

```rust
let context = Arc::new(CommandContext::new(working_dir.to_path_buf()));
let factory = CommandHandlerFactory::new(context);

// Clean handler creation
let command_handler = factory.create_create_handler();
```

#### Rationale

- **Consistency**: All handlers created the same way
- **Testability**: Factory can inject mocks for testing
- **Maintainability**: Adding new handlers just extends factory

#### Benefits

- ✅ Consistent handler creation
- ✅ Easier testing with mock factory
- ✅ Future commands automatically get same pattern

#### Implementation Checklist

- [ ] Create `src/presentation/commands/factory.rs`
- [ ] Implement `CommandHandlerFactory`
- [ ] Add factory methods for each command handler
- [ ] Update handlers to use factory
- [ ] Add tests for factory
- [ ] Run all tests
- [ ] Run linter

#### Testing Strategy

```rust
#[test]
fn it_should_create_handlers_with_shared_context() {
    let temp_dir = TempDir::new().unwrap();
    let context = Arc::new(CommandContext::new(temp_dir.path().to_path_buf()));
    let factory = CommandHandlerFactory::new(context.clone());

    let create_handler = factory.create_create_handler();
    let destroy_handler = factory.create_destroy_handler();

    // Both handlers share same context dependencies
    assert!(Arc::ptr_eq(context.repository(), /* verify sharing */));
}
```

---

### Proposal 10: Add Progress Reporter Abstraction

**GitHub Issue**: [#73](https://github.com/torrust/torrust-tracker-deployer/issues/73)

**Status**: ⏳ Not Started  
**Type**: 🔬 Advanced Pattern  
**Impact**: 🟢 Low  
**Effort**: 🔵🔵 Medium  
**Priority**: P2  
**Depends On**: Proposal 3

#### Problem

User output patterns are repeated:

```rust
output.progress("Step 1...");
// do work
output.progress("Step 2...");
// do work
output.success("Done!");
```

Could be abstracted for complex multi-step operations.

#### Proposed Solution

Create `ProgressReporter` in `output.rs` using `Arc<UserOutput>` to avoid lifetime issues:

````rust
use std::sync::Arc;

/// Progress reporter for multi-step operations
///
/// Uses `Arc<UserOutput>` internally to avoid lifetime constraints and
/// enable flexible usage patterns without borrowing conflicts.
pub struct ProgressReporter {
    output: Arc<UserOutput>,
    total_steps: usize,
    current: usize,
}

impl ProgressReporter {
    /// Create a new progress reporter
    ///
    /// # Arguments
    ///
    /// * `output` - Shared user output instance
    /// * `total_steps` - Total number of steps in the operation
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::sync::Arc;
    /// use torrust_tracker_deployer_lib::presentation::commands::output;
    ///
    /// let output = Arc::new(output::create_default());
    /// let mut reporter = output::ProgressReporter::new(output.clone(), 3);
    /// ```
    pub fn new(output: Arc<UserOutput>, total_steps: usize) -> Self {
        Self {
            output,
            total_steps,
            current: 0,
        }
    }

    /// Report progress for the next step
    ///
    /// # Arguments
    ///
    /// * `step_name` - Description of the current step
    pub fn step(&mut self, step_name: &str) {
        self.current += 1;
        let progress = format!("[{}/{}] {}", self.current, self.total_steps, step_name);
        self.output.progress(&progress);
    }

    /// Report successful completion of all steps
    pub fn complete(&self) {
        self.output.success(&format!("Completed {} steps successfully", self.total_steps));
    }

    /// Report completion with custom message
    pub fn complete_with(&self, message: &str) {
        self.output.success(message);
    }
}
````

Usage becomes cleaner and more flexible:

```rust
use std::sync::Arc;

// Create shared output
let output = Arc::new(output::create_default());

// Create reporter with shared output
let mut reporter = ProgressReporter::new(output.clone(), 3);

reporter.step("Load configuration");
// do work

reporter.step("Validate settings");
// do work

reporter.step("Create environment");
// do work

reporter.complete();

// Can still use the original output reference elsewhere
output.result("Additional info...");
```

#### Rationale

- **UX**: Users see clear progress for long operations with step counters
- **Consistency**: Same progress formatting everywhere
- **Optional**: Only use for complex multi-step operations
- **Simplified API**: Using `Arc<UserOutput>` eliminates lifetime issues and borrowing conflicts
- **Flexibility**: Can share the `UserOutput` between reporter and other code without borrowing problems

#### Benefits

- ✅ Better UX for long-running operations with clear progress indicators
- ✅ Consistent progress reporting across all commands
- ✅ Optional - only use where it adds value
- ✅ No lifetime issues - `Arc<UserOutput>` simplifies ownership
- ✅ Can share output reference between reporter and other code
- ✅ Cleaner API without complex lifetime parameters

#### Implementation Checklist

- [ ] Design `ProgressReporter` API
- [ ] Implement in `output.rs`
- [ ] Add documentation and examples
- [ ] Use in create handler (if beneficial)
- [ ] Use in destroy handler (if beneficial)
- [ ] Add tests
- [ ] Run linter

#### Testing Strategy

```rust
use std::sync::Arc;

#[test]
fn it_should_report_progress_for_steps() {
    let output = Arc::new(output::create_default());
    let mut reporter = ProgressReporter::new(output.clone(), 2);

    reporter.step("Step 1");
    // Verify output contains "[1/2] Step 1"

    reporter.step("Step 2");
    // Verify output contains "[2/2] Step 2"

    reporter.complete();
    // Verify completion message
}

#[test]
fn it_should_allow_shared_output_usage() {
    let output = Arc::new(output::create_default());
    let mut reporter = ProgressReporter::new(output.clone(), 2);

    reporter.step("Step 1");

    // Can still use original output reference
    output.result("Intermediate result");

    reporter.step("Step 2");
    reporter.complete();
}

#[test]
fn it_should_support_custom_completion_message() {
    let output = Arc::new(output::create_default());
    let mut reporter = ProgressReporter::new(output.clone(), 3);

    reporter.step("Step 1");
    reporter.step("Step 2");
    reporter.step("Step 3");

    reporter.complete_with("Environment created successfully with all steps completed");
    // Verify custom message appears
}
```

---

## 📈 Timeline

- **Start Date**: October 28, 2025
- **Estimated Completion**: TBD (depends on priority and available time)
- **Actual Completion**: TBD

### Suggested Sprint Plan

**Sprint 1**: Quick wins

- Proposals 1-5 (1-2 weeks)
- Focus: Eliminate duplication, improve consistency, establish standard structure

**Sprint 2**: Structural improvements (Part 1)

- Proposals 6-7 (2-3 weeks)
- Focus: Refactor complex functions, shared utilities

**Sprint 3**: Structural improvements (Part 2)

- Proposal 8 (1-2 weeks)
- Focus: Error consistency across all commands

**Sprint 4** (Optional): Advanced patterns

- Proposals 9-10 (1-2 weeks)
- Focus: Nice-to-have abstractions

## 🔍 Review Process

### Approval Criteria

- [x] All proposals reviewed by project maintainers
- [x] Technical feasibility validated
- [x] Aligns with [Development Principles](../development-principles.md)
- [x] Implementation plan is clear and actionable

### Completion Criteria

- [ ] All active proposals implemented
- [ ] All tests passing
- [ ] All linters passing
- [ ] Documentation updated
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch

## 📚 Related Documentation

- [Development Principles](../development-principles.md)
- [Contributing Guidelines](../contributing/README.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
- [Module Organization](../contributing/module-organization.md)

## 💡 Notes

### Dependencies Between Proposals

- Proposal 3 (CommandContext with UserOutput) depends on Proposal 2 (constants)
- Proposal 4 (split handlers) depends on Proposal 1 (needs subcommands structure)
- Proposal 5 (eliminate println) depends on Proposal 3 (CommandContext with UserOutput)
- Proposal 6 (refactor steps) depends on Proposal 4 (needs environment_handler.rs)
- Proposal 9 (factory) depends on Proposal 3 (CommandContext)
- Proposal 10 (progress) depends on Proposal 3 (CommandContext with UserOutput)

### Migration Strategy

This refactoring can be done incrementally:

1. Quick win proposals (1-5) can be implemented independently in any order (except dependencies noted)
2. Each proposal should be a separate PR for easier review
3. Tests must pass after each proposal
4. The codebase remains functional throughout the refactoring

### Lessons Learned

Will be documented as implementation progresses.

---

**Created**: October 28, 2025  
**Last Updated**: October 28, 2025  
**Status**: 📋 Planning
