# Proposal 1: Create Input Layer

**Issue**: #155  
**Parent Epic**: #154 - Presentation Layer Reorganization  
**Related**: [Refactor Plan](../refactors/plans/presentation-layer-reorganization.md) | [Design Proposal](../analysis/presentation-layer/design-proposal.md)

## Overview

Establish an explicit **Input Layer** in the presentation structure by creating `src/presentation/input/` and moving the CLI module there. This is the first step in transforming the presentation layer into a four-layer architecture (Input → Dispatch → Controllers → Views).

**Impact**: 🟢🟢 Medium - Clear separation of user input parsing from command execution  
**Effort**: 🔵🔵 Medium - Straightforward move with import updates  
**Estimated Time**: 2-3 hours

## Goals

- [ ] Create `src/presentation/input/` directory with proper module structure
- [ ] Move `src/presentation/cli/` to `src/presentation/input/cli/`
- [ ] Update all imports from `presentation::cli` to `presentation::input::cli`
- [ ] Add module documentation explaining the input layer's purpose
- [ ] Document this as first step in refactor plan
- [ ] Ensure old structure (`commands/`, `user_output/`) remains functional

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation  
**Module Path**: `src/presentation/input/`  
**Pattern**: Input Layer (Layer 1 of 4-layer presentation architecture)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Presentation layer handles user interaction (CLI parsing)
- [ ] No business logic in input layer (only parsing and validation)
- [ ] Use module organization conventions (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Input layer only responsible for CLI parsing (using Clap)
- [ ] No command execution logic in input layer
- [ ] No application logic in input layer
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))

### Target Structure

After this proposal:

```text
src/presentation/
├── input/                    # ← NEW: Layer 1 - Input parsing
│   ├── mod.rs                #    Re-exports cli module
│   └── cli/                  #    ← MOVED from presentation/cli/
│       ├── mod.rs            #    Main CLI structure
│       └── global.rs         #    Global arguments
├── commands/                 # ← UNCHANGED (for now)
├── user_output/              # ← UNCHANGED (for now)
├── progress.rs               # ← UNCHANGED (for now)
└── errors.rs                 # ← UNCHANGED
```

### Anti-Patterns to Avoid

- ❌ Adding any business logic to input layer
- ❌ Adding command execution logic to input layer
- ❌ Mixing input parsing with routing or handling
- ❌ Breaking existing `commands/` or `user_output/` functionality

## Specifications

### Input Layer Purpose

The **Input Layer** is responsible for:

1. **Parsing user input** - Using Clap to parse CLI arguments
2. **Input validation** - Basic format and type validation
3. **Command identification** - Determining which command was requested
4. **Argument extraction** - Extracting command-specific arguments

**Not responsible for**:

- ❌ Command routing (that's dispatch layer)
- ❌ Command execution (that's controllers layer)
- ❌ Output formatting (that's views layer)
- ❌ Business logic (that's domain/application layers)

### Module Documentation

Add to `src/presentation/input/mod.rs`:

````rust
//! Input Layer - User Input Parsing
//!
//! This module implements the **Input Layer** of the presentation architecture.
//! It is responsible for parsing user input from the command line interface.
//!
//! ## Architecture
//!
//! This is Layer 1 of the four-layer presentation architecture:
//!
//! ```text
//! Input (CLI parsing) → Dispatch (routing) → Controllers (handling) → Views (output)
//! ```
//!
//! ## Responsibilities
//!
//! - Parse CLI arguments using Clap
//! - Validate input format and types
//! - Identify requested commands
//! - Extract command-specific arguments
//!
//! ## What Does NOT Belong Here
//!
//! - Command routing logic (see `dispatch` layer)
//! - Command execution logic (see `controllers` layer)
//! - Output formatting (see `views` layer)
//! - Business logic (see `domain`/`application` layers)
//!
//! ## Related Documentation
//!
//! - [Refactor Plan](../../../docs/refactors/plans/presentation-layer-reorganization.md)
//! - [Design Proposal](../../../docs/analysis/presentation-layer/design-proposal.md)
//! - [Codebase Architecture](../../../docs/codebase-architecture.md)

pub mod cli;
````

### Existing CLI Module

The `cli/` module contains:

- `mod.rs` - Main `Cli` struct with Clap derives and `Commands` enum
- `global.rs` - `GlobalArgs` struct for global CLI options

**No changes needed** to the CLI module itself - only move location and update imports.

## Implementation Plan

### Phase 1: Create Directory Structure (30 minutes)

- [ ] Create `src/presentation/input/` directory
- [ ] Create `src/presentation/input/mod.rs` with module documentation
- [ ] Verify directory structure with `tree src/presentation/input/`

### Phase 2: Move CLI Module (30 minutes)

- [ ] Move `src/presentation/cli/` to `src/presentation/input/cli/`
- [ ] Verify files moved correctly:
  - [ ] `src/presentation/input/cli/mod.rs` exists
  - [ ] `src/presentation/input/cli/global.rs` exists
- [ ] Remove old `src/presentation/cli/` directory

### Phase 3: Update Imports (60 minutes)

Find all files importing from `presentation::cli` and update to `presentation::input::cli`:

- [ ] Update `src/presentation/mod.rs`:

  ```rust
  // OLD: pub mod cli;
  // NEW:
  pub mod input;
  ```

- [ ] Update `src/main.rs`:

  ```rust
  // OLD: use torrust_tracker_deployer_lib::presentation::cli::Cli;
  // NEW: use torrust_tracker_deployer_lib::presentation::input::cli::Cli;
  ```

- [ ] Update `src/presentation/commands/mod.rs` (if importing cli)

- [ ] Search for all usages:

  ```bash
  # Find all files importing presentation::cli
  rg "use.*presentation::cli" --type rust
  rg "presentation::cli::" --type rust
  ```

- [ ] Update each file found with new path `presentation::input::cli`

### Phase 4: Documentation Updates (30 minutes)

- [ ] Add note to `docs/refactors/plans/presentation-layer-reorganization.md`:

  ```markdown
  ### Proposal 1: Create Input Layer

  **Status**: ✅ Complete (Issue #X, PR #Y)
  **Completed**: [Date]
  **Lessons Learned**: [Brief notes on any discoveries during implementation]
  ```

- [ ] Update `README.md` if it references presentation structure

- [ ] Verify all documentation links still work

### Phase 5: Testing & Verification (30 minutes)

- [ ] Run pre-commit checks: `./scripts/pre-commit.sh`

  - [ ] cargo machete (no unused dependencies)
  - [ ] Linters pass (markdown, yaml, toml, clippy, rustfmt, shellcheck)
  - [ ] Unit tests pass (`cargo test`)
  - [ ] Documentation builds (`cargo doc`)
  - [ ] E2E tests pass

- [ ] Verify commands work:

  ```bash
  # Test various commands to ensure imports work
  cargo run -- --help
  cargo run -- create --help
  cargo run -- provision --help
  cargo run -- destroy --help
  ```

- [ ] Verify no compilation warnings: `cargo build 2>&1 | grep warning`

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Structure**:

- [ ] `src/presentation/input/` directory exists
- [ ] `src/presentation/input/mod.rs` contains module documentation
- [ ] `src/presentation/input/cli/` contains moved CLI module
- [ ] Old `src/presentation/cli/` directory removed

**Imports**:

- [ ] All imports updated from `presentation::cli` to `presentation::input::cli`
- [ ] No references to old path remain
- [ ] `cargo build` completes without warnings

**Functionality**:

- [ ] All CLI commands work as before
- [ ] `--help` output unchanged
- [ ] Command execution unchanged
- [ ] Old structure (`commands/`, `user_output/`) still functional

**Documentation**:

- [ ] `input/mod.rs` explains input layer purpose
- [ ] References refactor plan for context
- [ ] Refactor plan updated with completion status
- [ ] No broken documentation links

**Mergeable State**:

- [ ] Code compiles without warnings
- [ ] All tests pass
- [ ] Documentation accurate
- [ ] Ready to merge to main
- [ ] No intermediate or broken state

## Related Documentation

### Refactor Plan

- [Presentation Layer Reorganization Plan](../refactors/plans/presentation-layer-reorganization.md) - Full refactor context
- [Proposal 1 in Refactor Plan](../refactors/plans/presentation-layer-reorganization.md#proposal-1-create-input-layer) - High-level overview

### Design & Analysis

- [Design Proposal](../analysis/presentation-layer/design-proposal.md) - Four-layer architecture design
- [Current Structure Analysis](../analysis/presentation-layer/current-structure.md) - Problems being solved

### Guidelines

- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md) - Where code belongs
- [Module Organization](../contributing/module-organization.md) - How to organize modules
- [Codebase Architecture](../codebase-architecture.md) - Overall architecture

## Notes

### Why This First?

- **Clear separation** - Establishes pattern for four-layer architecture
- **Low risk** - CLI is well-isolated and easy to move
- **Foundation** - Other proposals build on this structure
- **Documentation** - Introduces refactor plan to readers

### Next Steps After Completion

1. **Merge to main** - This proposal is independently mergeable
2. **Review current state** - How did the move go? Any surprises?
3. **Update refactor plan** - Add completion status and lessons learned
4. **Detail Proposal 2** - Add implementation plan for Dispatch Layer
5. **Create issue for Proposal 2** - Ready for next work

### Success Indicators

After this proposal, developers should be able to:

- ✅ Understand the input layer is for CLI parsing only
- ✅ See clear separation between parsing and other concerns
- ✅ Find CLI code in predictable location (`input/cli/`)
- ✅ Know where to add new CLI arguments (in input layer)
- ✅ Know this is part of a larger refactoring (via documentation)

---

**Created**: November 6, 2025  
**Status**: Ready for Implementation  
**Next Action**: Begin Phase 1 (Create Directory Structure)
