# Proposal 2: Create Dispatch Layer

**Issue**: #156  
**Parent Epic**: #154 - Presentation Layer Reorganization  
**Related**: [Refactor Plan](../refactors/plans/presentation-layer-reorganization.md) | [Design Proposal](../analysis/presentation-layer/design-proposal.md)

## Overview

Extract command routing logic into an explicit **Dispatch Layer** with `src/presentation/dispatch/router.rs` and `ExecutionContext`. This separates routing from command execution and creates a clean pattern for controllers to access dependencies.

**Impact**: 🟢🟢🟢 High - Separates routing from execution, enables clean controller pattern  
**Effort**: 🔵🔵 Medium - Extract routing logic, create ExecutionContext  
**Estimated Time**: 2-3 hours

## Goals

- [ ] Create `src/presentation/dispatch/` directory with router and context modules
- [ ] Extract routing logic from `src/main.rs` and `commands/mod.rs` into `route_command()`
- [ ] Create `ExecutionContext` wrapper around `Container` for command execution
- [ ] Update `src/main.rs` to use new router
- [ ] Add module documentation explaining dispatch layer's purpose
- [ ] Document as second step in refactor plan
- [ ] Ensure old `commands/` directory remains functional

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation  
**Module Path**: `src/presentation/dispatch/`  
**Pattern**: Dispatch Layer (Layer 2 of 4-layer presentation architecture)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Presentation layer handles command routing
- [ ] No business logic in dispatch layer (only routing)
- [ ] Use module organization conventions (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Dispatch layer only responsible for routing commands to controllers
- [ ] No command execution logic in dispatch layer
- [ ] `ExecutionContext` provides clean interface to Container services
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))

### Target Structure

After this proposal:

```text
src/presentation/
├── input/                    # Layer 1 - CLI parsing
├── dispatch/                 # ← NEW: Layer 2 - Routing & context
│   ├── mod.rs                #    Re-exports router and context
│   ├── router.rs             #    route_command() function
│   └── context.rs            #    ExecutionContext wrapper
├── commands/                 # ← UNCHANGED (for now)
├── user_output/              # ← UNCHANGED (for now)
├── progress.rs               # ← UNCHANGED (for now)
└── errors.rs                 # ← UNCHANGED
```

### Anti-Patterns to Avoid

- ❌ Adding business logic to dispatch layer
- ❌ Adding command execution logic to router
- ❌ Direct Controller instantiation (should use ExecutionContext)
- ❌ Breaking existing command functionality

## Specifications

### Dispatch Layer Purpose

The **Dispatch Layer** is responsible for:

1. **Command routing** - Map parsed CLI commands to controller handlers
2. **Context creation** - Provide `ExecutionContext` with lazy-loaded services
3. **Error propagation** - Forward controller errors to error display layer

**Not responsible for**:

- ❌ CLI parsing (that's input layer)
- ❌ Command execution (that's controllers layer)
- ❌ Output formatting (that's views layer)
- ❌ Business logic (that's domain/application layers)

### Router Module (`dispatch/router.rs`)

````rust
//! Command Router
//!
//! Routes parsed CLI commands to their corresponding controller handlers.

use crate::presentation::input::cli::Commands;
use crate::presentation::dispatch::context::ExecutionContext;

/// Route a CLI command to its corresponding controller handler
///
/// This is the main entry point for command execution after CLI parsing.
///
/// # Arguments
///
/// * `command` - Parsed CLI command from input layer
/// * `context` - Execution context with access to Container services
///
/// # Returns
///
/// * `Ok(())` if command executed successfully
/// * `Err(...)` if command execution failed
///
/// # Example
///
/// ```rust
/// use crate::presentation::input::cli::{Cli, Parser};
/// use crate::presentation::dispatch::{route_command, ExecutionContext};
/// use crate::bootstrap::container::Container;
///
/// let cli = Cli::parse();
/// let container = Container::new();
/// let context = ExecutionContext::new(container);
///
/// if let Some(command) = cli.command {
///     route_command(command, &context).await?;
/// }
/// ```
pub async fn route_command(
    command: Commands,
    context: &ExecutionContext,
) -> Result<(), Box<dyn std::error::Error>> {
    use Commands::*;

    match command {
        Create { action } => {
            let controller = crate::presentation::commands::create::handle_create_command;
            controller(action, context).await?;
        }
        Provision { environment } => {
            let controller = crate::presentation::commands::provision::handle_provision_command;
            controller(&environment, context).await?;
        }
        Configure { environment } => {
            let controller = crate::presentation::commands::configure::handle_configure_command;
            controller(&environment, context).await?;
        }
        // ... other commands
    }

    Ok(())
}
````

### ExecutionContext Module (`dispatch/context.rs`)

```rust
//! Execution Context
//!
//! Provides controllers with access to Container services through a clean interface.

use std::sync::Arc;
use crate::bootstrap::container::Container;

/// Execution context for command handlers
///
/// Wraps the Container and provides access to lazy-loaded services.
/// Controllers use this to access dependencies without direct Container coupling.
///
/// # Thread Safety
///
/// The context is `Send + Sync` and can be safely shared across threads.
/// Internal services use `Arc<Mutex<Option<Arc<T>>>>` for thread-safe lazy loading.
pub struct ExecutionContext {
    container: Arc<Container>,
}

impl ExecutionContext {
    /// Create a new execution context
    pub fn new(container: Arc<Container>) -> Self {
        Self { container }
    }

    /// Get the underlying container
    ///
    /// Controllers typically don't need this - use specific service accessors instead.
    pub fn container(&self) -> &Arc<Container> {
        &self.container
    }

    // Service accessors (examples - adjust based on actual Container API)

    /// Get the OpenTofu client
    pub fn opentofu_client(&self) -> Arc<dyn OpenTofuClient> {
        self.container.opentofu_client()
    }

    /// Get the Ansible client
    pub fn ansible_client(&self) -> Arc<dyn AnsibleClient> {
        self.container.ansible_client()
    }

    /// Get the environment repository
    pub fn environment_repository(&self) -> Arc<dyn EnvironmentRepository> {
        self.container.environment_repository()
    }

    // Add other service accessors as needed
}
```

### Module Documentation (`dispatch/mod.rs`)

````rust
//! Dispatch Layer - Command Routing
//!
//! This module implements the **Dispatch Layer** of the presentation architecture.
//! It is responsible for routing parsed commands to their corresponding controllers.
//!
//! ## Architecture
//!
//! This is Layer 2 of the four-layer presentation architecture:
//!
//! ```text
//! Input (CLI parsing) → Dispatch (routing) → Controllers (handling) → Views (output)
//! ```
//!
//! ## Responsibilities
//!
//! - Route commands from input layer to controllers
//! - Provide `ExecutionContext` with Container services
//! - Forward controller results to error display
//!
//! ## What Does NOT Belong Here
//!
//! - CLI parsing logic (see `input` layer)
//! - Command execution logic (see `controllers` layer)
//! - Output formatting (see `views` layer)
//! - Business logic (see `domain`/`application` layers)
//!
//! ## Usage
//!
//! ```rust
//! use crate::presentation::input::cli::{Cli, Parser};
//! use crate::presentation::dispatch::{route_command, ExecutionContext};
//! use crate::bootstrap::container::Container;
//!
//! let cli = Cli::parse();
//! let container = Container::new();
//! let context = ExecutionContext::new(Arc::new(container));
//!
//! if let Some(command) = cli.command {
//!     route_command(command, &context).await?;
//! }
//! ```
//!
//! ## Related Documentation
//!
//! - [Refactor Plan](../../../docs/refactors/plans/presentation-layer-reorganization.md)
//! - [Design Proposal](../../../docs/analysis/presentation-layer/design-proposal.md)

pub mod router;
pub mod context;

pub use router::route_command;
pub use context::ExecutionContext;
````

## Implementation Plan

### Phase 1: Create Directory Structure (30 minutes)

- [ ] Create `src/presentation/dispatch/` directory
- [ ] Create `src/presentation/dispatch/mod.rs` with module documentation
- [ ] Create empty `src/presentation/dispatch/router.rs`
- [ ] Create empty `src/presentation/dispatch/context.rs`
- [ ] Update `src/presentation/mod.rs` to include `pub mod dispatch;`
- [ ] Verify directory structure: `tree src/presentation/dispatch/`

### Phase 2: Implement ExecutionContext (45 minutes)

- [ ] Implement `ExecutionContext` struct in `context.rs`
- [ ] Add `new()` constructor accepting `Arc<Container>`
- [ ] Add `container()` accessor method
- [ ] Add service accessor methods based on Container API:
  - [ ] `opentofu_client()`
  - [ ] `ansible_client()`
  - [ ] `environment_repository()`
  - [ ] `clock()`
  - [ ] Other services as needed
- [ ] Add module documentation
- [ ] Verify compilation: `cargo check`

### Phase 3: Implement Router (60 minutes)

- [ ] Analyze current routing logic in `src/main.rs` and `commands/mod.rs`

- [ ] Implement `route_command()` function in `router.rs`:

  - [ ] Match on `Commands` enum
  - [ ] Route each command to existing handler functions
  - [ ] Pass `ExecutionContext` to handlers
  - [ ] Handle errors and propagate to caller

- [ ] Add comprehensive function documentation with examples

- [ ] Update command handlers to accept `&ExecutionContext` parameter:

  - [ ] Modify `handle_create_command()` signature
  - [ ] Modify `handle_provision_command()` signature
  - [ ] Modify `handle_configure_command()` signature
  - [ ] Modify `handle_destroy_command()` signature
  - [ ] Other command handlers as needed

- [ ] Verify compilation: `cargo check`

### Phase 4: Update Main Entry Point (30 minutes)

- [ ] Update `src/main.rs` to use new router:

  ```rust
  use torrust_tracker_deployer_lib::presentation::input::cli::Cli;
  use torrust_tracker_deployer_lib::presentation::dispatch::{route_command, ExecutionContext};
  use torrust_tracker_deployer_lib::bootstrap::container::Container;

  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // Parse CLI
      let cli = Cli::parse();

      // Initialize container
      let container = Arc::new(Container::new());

      // Create execution context
      let context = ExecutionContext::new(container);

      // Route command
      if let Some(command) = cli.command {
          route_command(command, &context).await?;
      }

      Ok(())
  }
  ```

- [ ] Remove old routing logic from `main.rs`

- [ ] Verify compilation: `cargo check`

### Phase 5: Documentation Updates (30 minutes)

- [ ] Add completion note to `docs/refactors/plans/presentation-layer-reorganization.md`:

  ```markdown
  ### Proposal 2: Create Dispatch Layer

  **Status**: ✅ Complete (Issue #X, PR #Y)
  **Completed**: [Date]
  **Lessons Learned**: [Brief notes on any discoveries]
  ```

- [ ] Update `README.md` if it references routing logic

- [ ] Verify all documentation links work

### Phase 6: Testing & Verification (30 minutes)

- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`

  - [ ] cargo machete (no unused dependencies)
  - [ ] Linters pass (markdown, yaml, toml, clippy, rustfmt, shellcheck)
  - [ ] Unit tests pass (`cargo test`)
  - [ ] Documentation builds (`cargo doc`)
  - [ ] E2E tests pass

- [ ] Verify all commands work:

  ```bash
  cargo run -- --help
  cargo run -- create --help
  cargo run -- provision --help
  cargo run -- configure --help
  cargo run -- destroy --help
  ```

- [ ] Test actual command execution:

  ```bash
  # Test create command
  cargo run -- create template

  # Verify routing works correctly
  ```

- [ ] Verify no compilation warnings: `cargo build 2>&1 | grep warning`

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Structure**:

- [ ] `src/presentation/dispatch/` directory exists
- [ ] `src/presentation/dispatch/mod.rs` contains module documentation
- [ ] `src/presentation/dispatch/router.rs` contains `route_command()` function
- [ ] `src/presentation/dispatch/context.rs` contains `ExecutionContext` struct

**Routing**:

- [ ] `route_command()` handles all CLI commands
- [ ] All commands route to correct handlers
- [ ] Routing logic extracted from `main.rs`
- [ ] `src/main.rs` uses new router

**ExecutionContext**:

- [ ] `ExecutionContext` wraps `Container`
- [ ] Provides service accessor methods
- [ ] Command handlers accept `&ExecutionContext` parameter
- [ ] Thread-safe (`Send + Sync`)

**Functionality**:

- [ ] All CLI commands work as before
- [ ] Command execution unchanged
- [ ] Error handling works correctly
- [ ] Old `commands/` directory still functional

**Documentation**:

- [ ] `dispatch/mod.rs` explains dispatch layer purpose
- [ ] `router.rs` has comprehensive function documentation
- [ ] `context.rs` documents ExecutionContext usage
- [ ] References refactor plan for context
- [ ] Refactor plan updated with completion status

**Mergeable State**:

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Documentation accurate
- [ ] Ready to merge to main
- [ ] No intermediate or broken state

## Related Documentation

### Refactor Plan

- [Presentation Layer Reorganization Plan](../refactors/plans/presentation-layer-reorganization.md) - Full refactor context
- [Proposal 2 in Refactor Plan](../refactors/plans/presentation-layer-reorganization.md#proposal-2-create-dispatch-layer) - High-level overview

### Design & Analysis

- [Design Proposal](../analysis/presentation-layer/design-proposal.md) - Four-layer architecture design
- [Current Structure Analysis](../analysis/presentation-layer/current-structure.md) - Problems being solved

### Guidelines

- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md) - Where code belongs
- [Module Organization](../contributing/module-organization.md) - How to organize modules
- [Error Handling Guide](../contributing/error-handling.md) - Error handling patterns
- [Codebase Architecture](../codebase-architecture.md) - Overall architecture

### Container Documentation

- [Container Pattern in Design Proposal](../analysis/presentation-layer/design-proposal.md#dependency-injection-with-container) - Container integration details
- `src/bootstrap/container.rs` - Actual Container implementation

## Notes

### Why This Second?

- **Foundation for Controllers** - Controllers need ExecutionContext to access dependencies
- **Clear Separation** - Separates routing from execution
- **Enables Testing** - ExecutionContext makes mocking easier
- **Dependency on Input Layer** - Requires Proposal 1 (input layer) to be complete

### ExecutionContext vs Direct Container

**Why wrap Container in ExecutionContext?**

- ✅ Provides stable interface for controllers
- ✅ Hides Container implementation details
- ✅ Easier to add cross-cutting concerns (logging, metrics)
- ✅ Cleaner API for controller needs

### Container Integration Details

The `ExecutionContext` uses the existing `Container` from `src/bootstrap/container.rs` which provides:

- **Lazy loading**: Services initialized on first access with `Arc<Mutex<Option<Arc<T>>>>`
- **Thread safety**: Can be shared across threads
- **Single instance**: Each service created once per application lifetime

See [Container Pattern section](../analysis/presentation-layer/design-proposal.md#dependency-injection-with-container) in design proposal for detailed rationale.

### Next Steps After Completion

1. **Merge to main** - This proposal is independently mergeable
2. **Review current state** - How did routing extraction go? Any issues?
3. **Update refactor plan** - Add completion status and lessons learned
4. **Re-evaluate design** - Is four-layer architecture still optimal?
5. **Detail Proposal 3** - Add implementation plan for Controllers Layer
6. **Create issue for Proposal 3** - Ready for next work

### Success Indicators

After this proposal, developers should be able to:

- ✅ Understand routing is separate from execution
- ✅ See clear command flow: Input → Dispatch → Controllers → Views
- ✅ Use ExecutionContext to access Container services
- ✅ Add new commands by updating router, not main.rs
- ✅ Know this is part of larger refactoring (via documentation)

---

**Created**: November 6, 2025  
**Status**: Ready for Implementation (after Proposal 1 completes)  
**Next Action**: Wait for Proposal 1 completion, then begin Phase 1
