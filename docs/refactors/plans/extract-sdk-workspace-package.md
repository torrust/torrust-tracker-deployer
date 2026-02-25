# Extract SDK Workspace Package

## 📋 Overview

Extract the SDK delivery module from `src/presentation/sdk/` into its own workspace package at `packages/sdk/`. This makes the SDK independently consumable as a Rust crate, with an explicit dependency on the root `torrust_tracker_deployer_lib` crate for application-layer types.

This is the second step in a 4-part incremental plan:

1. [Presentation CLI/SDK Separation](presentation-cli-sdk-separation.md) — Separate CLI from SDK (**prerequisite**)
2. **This plan** — Extract SDK into `packages/sdk/`
3. [Extract Shared Types Package](extract-shared-types-package.md) — Create a shared types package
4. [SDK DDD Layer Boundary Fixes](sdk-ddd-layer-boundary-fixes.md) — Fix remaining DDD violations

**Target Files:**

- `src/presentation/sdk/` — current SDK location (to be moved)
- `packages/sdk/` — new package location
- `Cargo.toml` — workspace members list
- `examples/sdk/` — SDK examples (update imports)

**Scope:**

- Create `packages/sdk/` with its own `Cargo.toml` and `src/lib.rs`
- Move SDK source files into the new package
- The SDK package depends on `torrust_tracker_deployer_lib` for application-layer types
- Update SDK examples to use the new package
- Keep the root crate re-exporting SDK types for backward compatibility during transition

**Out of Scope:**

- Removing the SDK's dependency on the root crate (that requires Plans 3 and 4)
- Changing internal SDK behavior
- Publishing the package to crates.io

## 📊 Progress Tracking

**Total Active Proposals**: 1
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 1

### Phase Summary

- **Phase 0 - Package Extraction (High Impact, Medium Effort)**: ⏳ 0/1 completed (0%)

### Discarded Proposals

None.

### Postponed Proposals

None.

## 🎯 Key Problems Identified

### 1. SDK Is Not Independently Consumable

The SDK currently lives inside the monolithic root crate. Any consumer must depend on `torrust_tracker_deployer_lib`, pulling in CLI code, infrastructure modules, and all transitive dependencies — even if they only need the programmatic API.

### 2. Dependency Graph Is Implicit

The SDK's actual dependencies on application-layer types are hidden inside the root crate's module tree. Extracting it into a separate package makes the dependency graph explicit and enforceable by Cargo.

## 🚀 Refactoring Phases

---

## Phase 0: Package Extraction (High Impact, Medium Effort)

### Proposal #0: Create `packages/sdk/` Workspace Package

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵🔵 Medium
**Priority**: P0
**Depends On**: [Presentation CLI/SDK Separation](presentation-cli-sdk-separation.md) (Plan 1)
**Completed**: -
**Commit**: -

#### Problem

The SDK is a module within the root crate. There is no way for an external consumer to depend only on the SDK without bringing the entire deployer crate as a dependency.

#### Proposed Solution

Create a new workspace package `packages/sdk/` that contains the SDK source files. Initially, this package depends on the root `torrust_tracker_deployer_lib` crate for all application-layer types.

**Package structure:**

```text
packages/sdk/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs         ← Public API surface (re-exports)
    ├── builder.rs     ← DeployerBuilder
    ├── deployer.rs    ← Deployer facade
    └── error.rs       ← SDK-specific error types
```

**`packages/sdk/Cargo.toml`:**

```toml
[package]
name = "torrust-tracker-deployer-sdk"
version = "0.1.0"
edition = "2021"
description = "Programmatic SDK for the Torrust Tracker Deployer"
license = "MIT"

[dependencies]
torrust-tracker-deployer = { path = "../.." }
# External crates used by SDK directly:
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
```

**Root `Cargo.toml` workspace update:**

```toml
[workspace]
members = [
  "packages/linting",
  "packages/dependency-installer",
  "packages/sdk",
]
```

**`packages/sdk/src/lib.rs`** — mirrors the current `src/presentation/sdk/mod.rs` re-exports, but imports from `torrust_tracker_deployer_lib` instead of `crate`:

```rust
//! Torrust Tracker Deployer SDK
//!
//! Programmatic API for deploying Torrust Tracker instances.

mod builder;
mod deployer;
mod error;

// Core facade
pub use builder::{DeployerBuildError, DeployerBuilder};
pub use deployer::Deployer;

// Re-export types from the root crate that SDK consumers need
pub use torrust_tracker_deployer_lib::domain::{EnvironmentName, EnvironmentNameError};
pub use torrust_tracker_deployer_lib::application::command_handlers::create::config::{
    EnvironmentCreationConfig, EnvironmentCreationConfigBuildError,
    EnvironmentCreationConfigBuilder,
};
// ... (all current re-exports, updated to use the root crate path)
```

#### Key Decision: Dependency Direction

Initially the SDK package depends on the root crate:

```text
torrust-tracker-deployer-sdk → torrust-tracker-deployer (root crate)
```

This is a temporary arrangement. Plans 3 and 4 will progressively decouple the SDK from the root crate:

- Plan 3 extracts shared types into `packages/deployer-types/`, so both the SDK and root crate depend on the types package
- Plan 4 introduces application-layer abstractions so the SDK no longer needs domain/infrastructure types

The end-state dependency graph will be:

```text
torrust-tracker-deployer (root) → torrust-deployer-types
torrust-tracker-deployer-sdk    → torrust-deployer-types
torrust-tracker-deployer-sdk    → torrust-tracker-deployer (root, for application handlers only)
```

#### Rationale

- Extracting the SDK first (even with a dependency on the root crate) establishes the package boundary and build pipeline
- Every subsequent refactoring (Plans 3 and 4) has a concrete target package to improve
- External consumers can start depending on `torrust-tracker-deployer-sdk` immediately
- Follows the existing workspace pattern (`packages/linting`, `packages/dependency-installer`)

#### Benefits

- ✅ SDK is independently consumable as a crate
- ✅ Explicit dependency graph enforced by Cargo
- ✅ External consumers don't need to know about CLI modules
- ✅ Foundation for further decoupling (Plans 3 and 4)
- ✅ SDK examples move to the SDK package, keeping them co-located

#### Implementation Checklist

- [ ] Create `packages/sdk/` directory structure
- [ ] Create `packages/sdk/Cargo.toml` with dependency on root crate
- [ ] Create `packages/sdk/README.md`
- [ ] Move `src/presentation/sdk/builder.rs` → `packages/sdk/src/builder.rs`
- [ ] Move `src/presentation/sdk/deployer.rs` → `packages/sdk/src/deployer.rs`
- [ ] Move `src/presentation/sdk/error.rs` → `packages/sdk/src/error.rs`
- [ ] Create `packages/sdk/src/lib.rs` with re-exports (replacing `src/presentation/sdk/mod.rs`)
- [ ] Update all `crate::` imports in moved files to `torrust_tracker_deployer_lib::`
- [ ] Add `"packages/sdk"` to workspace members in root `Cargo.toml`
- [ ] Add `torrust-tracker-deployer-sdk = { path = "packages/sdk" }` to root `[dependencies]`
- [ ] Keep `src/presentation/sdk/` as a thin re-export module for backward compatibility:

  ```rust
  // src/presentation/sdk/mod.rs — backward compat re-exports
  pub use torrust_tracker_deployer_sdk::*;
  ```

- [ ] Move `examples/sdk/` → `packages/sdk/examples/` and update imports
- [ ] Update example declarations in root `Cargo.toml` (remove `[[example]]` entries for SDK examples)
- [ ] Add `[[example]]` entries in `packages/sdk/Cargo.toml`
- [ ] Verify `cargo build` succeeds for the entire workspace
- [ ] Verify `cargo test` succeeds for the entire workspace
- [ ] Verify SDK examples compile: `cargo build --examples -p torrust-tracker-deployer-sdk`
- [ ] Run linter and fix any issues
- [ ] Update codebase architecture docs to reflect the new package

#### Testing Strategy

- All existing tests must pass. The SDK is a code relocation — no behavioral changes.
- SDK examples must compile and work identically from the new package location.
- `cargo test --workspace` must pass (includes the new package).
- Verify that consumers importing from `torrust_tracker_deployer_lib::presentation::sdk` still work (backward compat re-exports).

---

## 📈 Timeline

- **Start Date**: TBD (after Plan 1 is complete)
- **Actual Completion**: TBD

## 🔍 Review Process

### Approval Criteria

- [ ] Technical feasibility validated
- [ ] Package name confirmed (`torrust-tracker-deployer-sdk`)
- [ ] Dependency direction documented and accepted

### Completion Criteria

- [ ] `packages/sdk/` exists as a workspace package
- [ ] SDK compiles independently: `cargo build -p torrust-tracker-deployer-sdk`
- [ ] All workspace tests pass: `cargo test --workspace`
- [ ] SDK examples compile from the SDK package
- [ ] Backward compatibility re-exports work
- [ ] All linters pass

## 📚 Related Documentation

- [SDK Interface Design ADR](../../decisions/sdk-presentation-layer-interface-design.md)
- [Codebase Architecture](../../codebase-architecture.md)
- [Presentation CLI/SDK Separation](presentation-cli-sdk-separation.md) (prerequisite)
- [Extract Shared Types Package](extract-shared-types-package.md) (next step)
- [SDK DDD Layer Boundary Fixes](sdk-ddd-layer-boundary-fixes.md) (final step)

## 💡 Notes

- The backward compatibility re-export layer (`src/presentation/sdk/mod.rs` → `pub use torrust_tracker_deployer_sdk::*`) can be removed in a later cleanup once all internal consumers are updated
- The SDK package name follows the `torrust-` prefix convention used by other Torrust ecosystem crates
- Initially the SDK package pulls the entire root crate as a dependency — this is intentional and will be refined in Plans 3 and 4
- Consider whether the SDK package should have its own `CHANGELOG.md` and versioning strategy

---

**Created**: 2026-02-24
**Last Updated**: 2026-02-24
**Status**: 📋 Planning
