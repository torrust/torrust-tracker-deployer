# Standardize Command View Folder Structure

## 📋 Overview

Bring three command view modules (`run`, `list`, `show`) into alignment with the
canonical folder structure that all other command view modules already follow. The
inconsistencies accumulate across three distinct problems in those commands.

**Expected canonical structure (followed by 10 of 13 commands):**

```text
<command>/
  mod.rs
  view_data/<dto>.rs      ← presentation-layer DTO struct
  views/
    json_view.rs          ← JsonView struct + render method
    text_view.rs          ← TextView struct + render method
```

**Deviating commands:**

| Command | Problem                                                                          |
| ------- | -------------------------------------------------------------------------------- |
| `run`   | No `views/` subfolder — `json_view.rs` and `text_view.rs` sit directly in `run/` |
| `run`   | No `view_data/` — DTO struct is defined inline inside `json_view.rs`             |
| `list`  | No `view_data/` — uses application-layer DTO (`EnvironmentList`) directly        |
| `show`  | No `view_data/` — uses application-layer DTO (`EnvironmentInfo`) directly        |

**Target Files (after refactor):**

- `src/presentation/cli/views/commands/run/views/json_view.rs` (moved from `run/json_view.rs`)
- `src/presentation/cli/views/commands/run/views/text_view.rs` (moved from `run/text_view.rs`)
- `src/presentation/cli/views/commands/run/view_data/run_details.rs` (new — extracted DTO)
- `src/presentation/cli/views/commands/list/view_data/list_details.rs` (new)
- `src/presentation/cli/views/commands/show/view_data/show_details.rs` (new)

**Scope:**

- Move and reorganize files only — no logic changes
- Extract the inline DTO from `run/json_view.rs` to `run/view_data/run_details.rs`
- Introduce presentation-layer view DTOs for `list` and `show` to replace direct use
  of application-layer types in the view layer
- Update `mod.rs` files and all import paths
- Out of scope: changing the `Render<T>` trait, `ViewRenderError`, or any serialization logic

## ⚠️ Ordering Constraint

**This plan must be completed before implementing Proposal #2
of [`standardize-json-view-render-api`](standardize-json-view-render-api.md).**

That plan's Proposal #2 (standardize return types) modifies every `json_view.rs` file.
If the file moves in this plan happen after Proposal #2 is merged, the diff will be
harder to review (logic changes + file moves mixed). The correct order is:

1. **This plan** (folder structure only — pure moves and one DTO extraction)
2. `standardize-json-view-render-api` Proposal #1 (add `ViewRenderError` + `Render<T>` trait)
3. `standardize-json-view-render-api` Proposal #2 (convert all return types)

## 📊 Progress Tracking

**Total Active Proposals**: 3
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 3
**In Progress**: 0
**Not Started**: 0

### Phase Summary

- **Phase 0 - Fix `run` module layout (High Impact, Low Effort)**: ✅ 2/2 completed (100%)
- **Phase 1 - Fix `list` and `show` module layout (Medium Impact, Low Effort)**: ✅ 1/1 completed (100%)

### Discarded Proposals

None

### Postponed Proposals

None

## 🎯 Key Problems Identified

### 1. `run` Has No `views/` Subfolder

The `run` command module places `json_view.rs` and `text_view.rs` directly inside `run/`,
with no `views/` subfolder:

```text
run/
  mod.rs
  json_view.rs       ← should be views/json_view.rs
  text_view.rs       ← should be views/text_view.rs
```

Every other command uses a `views/` subfolder to hold its view implementations. The flat
layout makes `run` visually inconsistent and breaks the pattern that allows contributors
to navigate any command directory predictably.

### 2. `run` Has an Inline DTO

Inside `run/json_view.rs`, a presentation-layer DTO struct is defined inline:

```rust
/// DTO for JSON output of run command
#[derive(Debug, Serialize)]
pub struct RunData { ... }
```

In all other commands this type lives in a dedicated `view_data/<dto>.rs` file. Embedding
the DTO inside the view file conflates two responsibilities: data shaping and rendering.
It also makes the DTO harder to discover and test independently.

### 3. `list` and `show` Use Application-Layer DTOs Directly in the View Layer

`list/views/json_view.rs` imports `EnvironmentList` from
`application::command_handlers::list::info`. `show/views/json_view.rs` imports
`EnvironmentInfo` from `application::command_handlers::show::info`.

The standard pattern is for the view layer to own its input DTOs in `view_data/`, so that:

- The presentation layer is not directly coupled to application-layer module paths
- The `view_data/` location serves as the explicit contract between command handler and view
- Contributors can navigate to `view_data/` to understand what a view expects, without
  reading the application layer

For `list` and `show`, this means creating thin wrapper or re-export modules in
`view_data/` that bring the relevant types into the presentation layer's own namespace.

## 🚀 Refactoring Phases

---

## Phase 0: Fix `run` Module Layout

### Proposal #1: Move `run` View Files Into `views/` Subfolder

**Status**: ✅ Completed
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵 Low
**Priority**: P0
**Depends On**: None
**Completed**: 2026-02-28
**Commit**: `8e3f8a41`

#### Problem

`run/json_view.rs` and `run/text_view.rs` are not in a `views/` subfolder. The `run/mod.rs`
declares them as direct children:

```rust
mod json_view;
mod text_view;
```

Contributors looking for the run command's view implementations follow the pattern from
other commands (`views/json_view.rs`) and do not find a `views/` directory.

#### Proposed Solution

Move files:

```text
run/json_view.rs  →  run/views/json_view.rs
run/text_view.rs  →  run/views/text_view.rs
```

Create `run/views/mod.rs`:

```rust
mod json_view;
mod text_view;

pub use json_view::JsonView;
pub use text_view::TextView;
```

Update `run/mod.rs`:

```rust
pub mod view_data;
mod views;

pub use views::JsonView;
pub use views::TextView;
```

All public exports from `run/mod.rs` remain the same — callers are unaffected.

#### Benefits

- ✅ `run` matches the layout of all other 12 command view modules
- ✅ Contributors can navigate to `views/json_view.rs` predictably

#### Implementation Checklist

- [x] Create directory `src/presentation/cli/views/commands/run/views/`
- [x] Move `run/json_view.rs` to `run/views/json_view.rs`
- [x] Move `run/text_view.rs` to `run/views/text_view.rs`
- [x] Create `run/views/mod.rs` exporting `JsonView` and `TextView`
- [x] Update `run/mod.rs` to use `mod views` and re-export through it
- [x] Verify all tests pass
- [x] Run `cargo run --bin linter all` and fix issues

#### Testing Strategy

Pure file move — no logic changes. Compilation is the test. All existing unit tests and
E2E tests must pass unchanged.

---

### Proposal #2: Extract `run` Inline DTO to `view_data/`

**Status**: ✅ Completed
**Impact**: 🟢🟢 Medium
**Effort**: 🔵 Low
**Priority**: P0
**Depends On**: Proposal #1
**Completed**: 2026-02-28
**Commit**: `ff0c6d7b`

#### Problem

The DTO struct for the `run` command's JSON output is defined inside `json_view.rs`:

```rust
// run/views/json_view.rs (after Proposal #1)
#[derive(Debug, Serialize)]
pub struct RunData {
    pub environment_name: String,
    pub state: String,
    pub services: Vec<ServiceInfo>,
    pub grafana: Option<GrafanaInfo>,
}
```

This mixes data shaping (what the DTO looks like) with rendering (how it is serialized).
All other commands place their DTOs in a dedicated `view_data/<dto>.rs` file.

#### Proposed Solution

Create `run/view_data/run_details.rs`:

```rust
//! View data DTO for the run command output.

use serde::Serialize;

use crate::application::command_handlers::show::info::{GrafanaInfo, ServiceInfo};

/// DTO for JSON output of run command.
#[derive(Debug, Serialize)]
pub struct RunDetailsData {
    pub environment_name: String,
    pub state: String,
    pub services: Vec<ServiceInfo>,
    pub grafana: Option<GrafanaInfo>,
}
```

Create `run/view_data/mod.rs`:

```rust
pub mod run_details;

pub use run_details::RunDetailsData;
```

Update `run/views/json_view.rs` to import `RunDetailsData` from `run::view_data` and
remove the inline struct definition.

Rename the struct to `RunDetailsData` for consistency with the naming convention used
in other commands (e.g., `ConfigureDetailsData`, `DestroyDetailsData`).

#### Benefits

- ✅ Consistent DTO placement across all 13 command view modules
- ✅ DTO is independently discoverable and testable
- ✅ Clear separation between data shape and rendering logic

#### Implementation Checklist

- [x] Create `src/presentation/cli/views/commands/run/view_data/run_details.rs` with `RunDetailsData`
- [x] Create `src/presentation/cli/views/commands/run/view_data/mod.rs`
- [x] Update `run/mod.rs` to declare `pub mod view_data`
- [x] Update `run/views/json_view.rs` to use `RunDetailsData` from `super::super::view_data`
      (or the crate-level path) and remove the inline struct
- [x] Update all call sites constructing the old inline struct to construct `RunDetailsData`
- [x] Verify all tests pass
- [x] Run `cargo run --bin linter all` and fix issues

#### Testing Strategy

Changes struct name and file location only. Update any test that constructs or names the
old struct. Observable JSON output is unaffected (field names do not change).

---

## Phase 1: Fix `list` and `show` Module Layout

### Proposal #3: Add `view_data/` to `list` and `show`

**Status**: ✅ Completed
**Impact**: 🟢🟢 Medium
**Effort**: 🔵 Low
**Priority**: P1
**Depends On**: None (can be parallel with Phase 0)
**Completed**: 2026-02-28
**Commit**: `3d63edc6`

#### Problem

`list` and `show` use application-layer types directly in their view files:

```rust
// list/views/json_view.rs
use crate::application::command_handlers::list::info::EnvironmentList;

// show/views/json_view.rs
use crate::application::command_handlers::show::info::EnvironmentInfo;
```

Both are missing a `view_data/` subfolder. The standard pattern puts the DTO (or its
re-export) in `view_data/` so that the view files only need to look one level up for
their input type, and the application-layer path is not scattered across view files.

#### Proposed Solution

Create `list/view_data/list_details.rs`:

```rust
//! View data for the list command.
//!
//! Re-exports the application-layer DTO as the canonical view input type.
//! The presentation layer references this module rather than importing directly
//! from the application layer.

pub use crate::application::command_handlers::list::info::EnvironmentList;
```

Create `show/view_data/show_details.rs`:

```rust
//! View data for the show command.

pub use crate::application::command_handlers::show::info::EnvironmentInfo;
```

Add `view_data/mod.rs` for each:

```rust
// list/view_data/mod.rs
pub mod list_details;
pub use list_details::EnvironmentList;

// show/view_data/mod.rs
pub mod show_details;
pub use show_details::EnvironmentInfo;
```

Update `list/mod.rs` and `show/mod.rs` to declare `pub mod view_data`.

Update imports in `list/views/json_view.rs`, `list/views/text_view.rs`,
`show/views/json_view.rs`, and `show/views/text_view.rs` to use the local `view_data`
re-exports instead of the application-layer paths.

#### Rationale

Using re-exports rather than new wrapper structs avoids duplicating type definitions while
still establishing `view_data/` as the explicit boundary. If the application-layer DTO
ever changes its module path, only the `view_data/` file needs updating — not every view
file in the command.

#### Benefits

- ✅ All 13 command modules have `view_data/`
- ✅ View files do not import directly from the application layer
- ✅ Single place to update if application DTO paths change
- ✅ Consistent navigation: every command has the same discoverable layout

#### Implementation Checklist

- [x] Create `src/presentation/cli/views/commands/list/view_data/list_details.rs`
- [x] Create `src/presentation/cli/views/commands/list/view_data/mod.rs`
- [x] Update `list/mod.rs` to declare `pub mod view_data`
- [x] Update imports in `list/views/json_view.rs` and `list/views/text_view.rs`
- [x] Create `src/presentation/cli/views/commands/show/view_data/show_details.rs`
- [x] Create `src/presentation/cli/views/commands/show/view_data/mod.rs`
- [x] Update `show/mod.rs` to declare `pub mod view_data`
- [x] Update imports in `show/views/json_view.rs`, `show/views/text_view.rs`,
      and all other files under `show/views/` that import application-layer types
- [x] Verify all tests pass
- [x] Run `cargo run --bin linter all` and fix issues

#### Testing Strategy

Pure import path changes. Compilation is the primary test. No behavior changes.

---

## 📈 Timeline

- **Start Date**: 2026-02-27
- **Actual Completion**: TBD

## 🔍 Review Process

### Approval Criteria

- [x] Technical feasibility validated
- [x] Aligns with [Development Principles](../development-principles.md)
- [x] Implementation plan is clear and actionable
- [x] Priorities are correct (high-impact/low-effort first)

### Completion Criteria

- [x] All active proposals implemented
- [x] All tests passing
- [x] All linters passing
- [x] Documentation updated
- [x] Changes merged to main branch

## 📚 Related Documentation

- [Development Principles](../development-principles.md)
- [Contributing Guidelines](../contributing/README.md)
- [Module Organization](../contributing/module-organization.md)
- [Standardize JsonView Render API](standardize-json-view-render-api.md) — must be done **after** this plan

## 💡 Notes

The `run` command was likely added in a hurry and missed the `views/` subfolder because
no other existing command at that time had the wrong structure to copy from. The `list`
and `show` commands may have intentionally imported application DTOs directly, since those
types are already well-shaped for display — but the consistency cost of skipping
`view_data/` outweighs the slight brevity of a direct import.

The `show` command's `views/` subfolder already contains multiple sub-views
(`basic.rs`, `grafana.rs`, `https_hint.rs`, `infrastructure.rs`, `next_step.rs`,
`prometheus.rs`, `tracker_services.rs`, `mod.rs`) in addition to `json_view.rs` and
`text_view.rs`. This is intentional — `show` renders richer data — and is not a deviation.

---

**Created**: 2026-02-27
**Last Updated**: 2026-02-28
**Status**: ✅ Completed
