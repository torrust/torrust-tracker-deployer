# Presentation Layer CLI/SDK Separation

## 📋 Overview

Establish a clean physical separation between the two delivery mechanisms (CLI and SDK) in the presentation layer. Currently, CLI-specific modules sit beside the SDK at the same directory level in `src/presentation/`, with no structural enforcement preventing cross-dependencies. This refactoring moves all CLI modules under an explicit `cli/` sub-tree.

This is the first step in a 4-part incremental plan to extract the SDK into its own workspace package:

1. **This plan** — Separate CLI from SDK in the presentation layer
2. [Extract SDK Workspace Package](extract-sdk-workspace-package.md) — Move SDK into `packages/sdk/`
3. [Extract Shared Types Package](extract-shared-types-package.md) — Create a shared types package for cross-cutting value objects
4. [SDK DDD Layer Boundary Fixes](sdk-ddd-layer-boundary-fixes.md) — Fix remaining DDD violations in the SDK

**Target Files:**

- `src/presentation/mod.rs` — restructure module declarations
- `src/presentation/controllers/` — move to `cli/`
- `src/presentation/dispatch/` — move to `cli/`
- `src/presentation/input/` — move to `cli/`
- `src/presentation/views/` — move to `cli/`
- `src/presentation/error.rs` — move to `cli/`
- `src/presentation/errors.rs` — move to `cli/`
- `src/presentation/tests/` — move to `cli/`
- All files that import from the moved modules (mainly `bootstrap/`, `main.rs`, and intra-CLI references)

**Scope:**

- Move all CLI-specific modules under `presentation/cli/`
- Update all import paths across the codebase
- Ensure SDK has zero imports from `presentation::cli`
- Preserve full backward compatibility for existing CLI behavior

**Out of Scope:**

- Changing any business logic
- Modifying the SDK module structure
- Extracting packages (that's Plan 2)

## 📊 Progress Tracking

**Total Active Proposals**: 1
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 1

### Phase Summary

- **Phase 0 - Directory Reorganization (High Impact, Medium Effort)**: ⏳ 0/1 completed (0%)

### Discarded Proposals

None.

### Postponed Proposals

None.

## 🎯 Key Problems Identified

### 1. Flat Namespace Mixes CLI and SDK Modules

The current `src/presentation/` layout places CLI-specific modules at the same level as the SDK:

```text
presentation/
├── controllers/   ← CLI-only
├── dispatch/      ← CLI-only
├── input/         ← CLI-only (Clap parsing)
├── views/         ← CLI-only (UserOutput, Theme, formatters)
├── error.rs       ← CLI-only (handle_error for UserOutput)
├── errors.rs      ← CLI-only (CommandError enum)
├── sdk/           ← SDK-only
├── tests/         ← CLI-only
└── mod.rs         ← Re-exports from both
```

The SDK and CLI are independent today (the SDK has zero imports from CLI modules), but this is **implicit** — there is no structural enforcement. Any developer (or AI agent) adding code to `presentation/` could accidentally create cross-dependencies.

## 🚀 Refactoring Phases

---

## Phase 0: Directory Reorganization (High Impact, Medium Effort)

### Proposal #0: Move CLI Modules into an Explicit `cli/` Sub-Tree

**Status**: ⏳ Not Started
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵🔵 Medium
**Priority**: P0
**Depends On**: None
**Completed**: -
**Commit**: -

#### Problem

No structural enforcement separates CLI and SDK in the presentation layer. The two delivery mechanisms share a flat namespace, making it easy to accidentally create cross-dependencies.

#### Proposed Solution

Move all CLI-specific modules under a new `presentation/cli/` directory:

```text
presentation/
├── cli/              ← All CLI delivery code lives here
│   ├── controllers/
│   ├── dispatch/
│   ├── input/
│   ├── views/
│   ├── error.rs
│   ├── errors.rs
│   ├── tests/
│   └── mod.rs        ← Re-exports for CLI consumers
├── sdk/              ← All SDK delivery code lives here (unchanged)
│   ├── builder.rs
│   ├── deployer.rs
│   ├── error.rs
│   └── mod.rs
└── mod.rs            ← Only declares `pub mod cli;` and `pub mod sdk;`
```

The top-level `presentation/mod.rs` becomes minimal — it declares the two sub-modules and optionally re-exports the most-used types for backward compatibility during the transition.

#### Rationale

- **Explicit boundary**: A developer looking at `presentation/` immediately sees two independent delivery mechanisms
- **No shared code by default**: If shared logic emerges later (e.g., common error mapping), it must be placed in an explicit `presentation/shared/` module — making the coupling visible and deliberate
- **Prerequisite for package extraction**: Plan 2 (Extract SDK Package) needs a clean boundary to know exactly which files belong to the SDK

#### Benefits

- ✅ Physical separation prevents accidental coupling between CLI and SDK
- ✅ Clear ownership — each sub-tree has a single delivery's concerns
- ✅ Makes the two-delivery-mechanism architecture immediately visible in the directory tree
- ✅ Future-proof — adding a third delivery mechanism (e.g., gRPC) follows the same pattern

#### Implementation Checklist

- [ ] Create `src/presentation/cli/` directory
- [ ] Move `controllers/`, `dispatch/`, `input/`, `views/`, `error.rs`, `errors.rs`, `tests/` into `presentation/cli/`
- [ ] Create `src/presentation/cli/mod.rs` with the same module declarations and re-exports currently in `presentation/mod.rs` (minus `sdk`)
- [ ] Update `src/presentation/mod.rs` to declare only `pub mod cli;` and `pub mod sdk;` (plus backward-compat re-exports if needed)
- [ ] Update all `use crate::presentation::controllers::...` imports across the codebase to `use crate::presentation::cli::controllers::...`
- [ ] Update all `use crate::presentation::views::...` imports to `use crate::presentation::cli::views::...`
- [ ] Update all `use crate::presentation::dispatch::...`, `use crate::presentation::input::...`, `use crate::presentation::error*` imports similarly
- [ ] Update `src/main.rs` and `src/bootstrap/` imports
- [ ] Verify the SDK module has zero imports from `crate::presentation::cli`
- [ ] Update the `presentation/mod.rs` module documentation
- [ ] Verify all tests pass (`cargo test`)
- [ ] Run linter and fix any issues

#### Testing Strategy

- Pure move/rename — no logic changes. All existing tests must pass after updating import paths.
- `cargo test` full suite must pass
- `cargo run --bin linter all` must pass
- SDK examples must compile unchanged (the `presentation::sdk` path is unaffected)

---

## 📈 Timeline

- **Start Date**: TBD
- **Actual Completion**: TBD

## 🔍 Review Process

### Approval Criteria

- [ ] Technical feasibility validated
- [ ] Aligns with [Development Principles](../../development-principles.md)

### Completion Criteria

- [ ] `src/presentation/` contains only `cli/`, `sdk/`, and `mod.rs` at the top level
- [ ] No `use crate::presentation::cli::` statement in any `src/presentation/sdk/*.rs` file
- [ ] All tests passing (`cargo test`)
- [ ] All linters passing (`cargo run --bin linter all`)
- [ ] SDK examples compile unchanged

## 📚 Related Documentation

- [SDK Interface Design ADR](../../decisions/sdk-presentation-layer-interface-design.md)
- [Codebase Architecture](../../codebase-architecture.md)
- [Extract SDK Workspace Package](extract-sdk-workspace-package.md) (next step)

## 💡 Notes

- This is a prerequisite for all subsequent SDK extraction and DDD fix plans
- The refactoring is purely structural — no behavioral changes
- All import path updates are mechanical and can be verified by compilation

---

**Created**: 2026-02-24
**Last Updated**: 2026-02-24
**Status**: 📋 Planning
