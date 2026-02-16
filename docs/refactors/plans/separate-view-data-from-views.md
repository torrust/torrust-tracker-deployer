# Separate View Data from Views in Presentation Layer

## 📋 Overview

Establish clear separation between view data structures (DTOs) and view rendering logic in the presentation layer. This refactoring creates a consistent, scalable structure for all command views before implementing 3 additional commands with JSON output (EPIC #348 tasks 12.3-12.5).

**Target Files:**

- `src/presentation/views/commands/create/*.rs`
- `src/presentation/views/commands/provision/*.rs`
- `src/presentation/views/commands/list/*.rs`
- `src/presentation/views/commands/show/*.rs`
- `src/presentation/views/commands/shared/*.rs`

**Scope:**

- Separate view data (DTOs) into `view_data/` subdirectories
- Organize views into `views/` subdirectories
- Maintain consistent structure across all commands
- Update all import paths and module re-exports
- Update documentation to reflect new structure

## 📊 Progress Tracking

**Total Active Proposals**: 5
**Total Postponed**: 0
**Total Discarded**: 1
**Completed**: 4
**In Progress**: 0
**Not Started**: 0

### Phase Summary

- **Phase 0 - Foundation Commands**: ✅ 2/2 completed (100%)
- **Phase 1 - Mixed Commands**: ✅ 2/2 completed (100%)
- **Phase 2 - Documentation**: ❌ 0/1 discarded (not needed)

### Discarded Proposals

- **Proposal #4**: Update Shared Components (Not Needed - current structure is optimal)

### Postponed Proposals

None. This refactoring is a prerequisite for EPIC #348 tasks 12.3-12.5.

## 🎯 Key Problems Identified

### 1. Inconsistent File Organization

The presentation layer currently mixes two distinct responsibilities in the same directory level:

```rust
// Current: Ambiguous - is this data or rendering?
src/presentation/views/commands/provision/
├── provision_details.rs           (DATA)
├── connection_details.rs          (DATA)
├── dns_reminder.rs                (DATA)
├── text_view.rs                   (VIEW)
└── json_view.rs                   (VIEW)
```

This creates ambiguity about where new code belongs and makes the codebase harder to navigate.

### 2. Scalability Issues

With 3 more commands planned (show, run, list with JSON output), the current flat structure will become increasingly difficult to maintain. Each command will have 1 DTO + N views, and without clear organization, the pattern won't be obvious to contributors.

### 3. Lack of Discoverability

New contributors must mentally parse file purposes rather than relying on clear directory structure. This slows onboarding and increases the risk of placing code in the wrong location.

## 🚀 Refactoring Phases

---

## Phase 0: Foundation Commands (create, provision)

These commands already follow the DTO + Views pattern and just need reorganization.

### Proposal #0: Refactor Create Command Structure

**Status**: ✅ Completed
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵 Low
**Priority**: P0
**Depends On**: None
**Completed**: 2026-02-16
**Commit**: 9a52ddc2

#### Problem

The `create` command has view data and views in the same directory:

```rust
src/presentation/views/commands/create/
├── environment_details.rs        (VIEW DATA - DTO)
├── text_view.rs                  (VIEW - Text rendering)
├── json_view.rs                  (VIEW - JSON rendering)
└── mod.rs
```

#### Proposed Solution

Separate into subdirectories:

```rust
src/presentation/views/commands/create/
├── view_data/
│   └── environment_details.rs
├── views/
│   ├── text_view.rs
│   └── json_view.rs
└── mod.rs
```

Update `mod.rs` to re-export from new locations:

```rust
pub mod view_data {
    pub use self::environment_details::EnvironmentDetailsData;
    pub mod environment_details;
}

pub mod views {
    pub use self::json_view::JsonView;
    pub use self::text_view::TextView;
    pub mod json_view;
    pub mod text_view;
}
```

#### Rationale

- Clear semantic distinction between data and rendering
- Follows same pattern that will be used for all commands
- Easier to find and add new views or data structures

#### Benefits

- ✅ Clear separation of concerns
- ✅ Consistent with upcoming command structure
- ✅ Easier code navigation
- ✅ Better discoverability for new contributors

#### Implementation Checklist

- [x] Create `view_data/` and `views/` subdirectories
- [x] Move `environment_details.rs` → `view_data/`
- [x] Move `text_view.rs` and `json_view.rs` → `views/`
- [x] Update `mod.rs` with new module structure
- [x] Update imports in controller (`src/presentation/controllers/create/handler.rs`)
- [x] Update imports in tests
- [x] Verify all tests pass
- [x] Run linter and fix any issues

#### Testing Strategy

```bash
# Verify compilation
cargo build

# Run all tests
cargo test

# Verify linters
cargo run --bin linter all
```

---

### Proposal #1: Refactor Provision Command Structure

**Status**: ✅ Completed
**Impact**: 🟢🟢🟢 High
**Effort**: 🔵 Low
**Priority**: P0
**Depends On**: None
**Completed**: 2026-02-16
**Commit**: 6539e787

#### Problem

The `provision` command has view data and views mixed in the same directory:

```rust
src/presentation/views/commands/provision/
├── provision_details.rs          (VIEW DATA - DTO)
├── connection_details.rs         (VIEW DATA - Helper DTO)
├── dns_reminder.rs               (VIEW DATA - Helper DTO)
├── text_view.rs                  (VIEW - Text rendering)
├── json_view.rs                  (VIEW - JSON rendering)
└── mod.rs
```

#### Proposed Solution

Separate into subdirectories:

```rust
src/presentation/views/commands/provision/
├── view_data/
│   ├── provision_details.rs          (main DTO)
│   ├── connection_details.rs         (helper DTO)
│   └── dns_reminder.rs               (helper DTO)
├── views/
│   ├── text_view.rs
│   └── json_view.rs
└── mod.rs
```

Update imports to use new paths:

```rust
use crate::presentation::views::commands::provision::view_data::ProvisionDetailsData;
use crate::presentation::views::commands::provision::views::{TextView, JsonView};
```

#### Rationale

Same rationale as Proposal #0 - establish consistent pattern.

#### Benefits

- ✅ Consistent with create command structure
- ✅ Clear separation of data vs rendering
- ✅ Easier to add new output formats
- ✅ Helper DTOs clearly grouped together

#### Implementation Checklist

- [x] Create `view_data/` and `views/` subdirectories
- [x] Move `provision_details.rs`, `connection_details.rs`, `dns_reminder.rs` → `view_data/`
- [x] Move `text_view.rs` and `json_view.rs` → `views/`
- [x] Update `mod.rs` with new module structure
- [x] Update imports in controller (`src/presentation/controllers/provision/handler.rs`)
- [x] Update imports in tests
- [x] Verify all tests pass
- [x] Run linter and fix any issues

#### Testing Strategy

```bash
cargo build && cargo test
cargo run --bin linter all
```

---

## Phase 1: Mixed Commands (list, show)

These commands need analysis and extraction before reorganization.

### Proposal #2: Refactor List Command Structure

**Status**: ✅ Completed
**Impact**: 🟢🟢 Medium
**Effort**: 🔵🔵 Medium
**Priority**: P1
**Depends On**: Proposals #0, #1 (establishes pattern)
**Completed**: 2026-02-16
**Commit**: 6cf29662

#### Problem

The `list` command has a single file that mixes data structure and view logic:

```rust
src/presentation/views/commands/list/
├── environment_list.rs           (MIXED - View with embedded data)
└── mod.rs
```

This will need JSON output support soon (EPIC #348 task 12.5), requiring the same DTO + Views pattern.

#### Proposed Solution

Extract data structure and create separate views:

```rust
src/presentation/views/commands/list/
├── view_data/
│   └── environment_list.rs           (data structure extracted)
├── views/
│   └── text_view.rs                  (current table rendering)
└── mod.rs
```

**Step 1**: Analyze `environment_list.rs` to identify:

- Data structure (DTO)
- View rendering logic (table formatting)

**Step 2**: Extract data into `view_data/environment_list.rs`:

```rust
// view_data/environment_list.rs
pub struct EnvironmentListData {
    pub environments: Vec<EnvironmentSummary>,
}

pub struct EnvironmentSummary {
    pub name: String,
    pub state: String,
    // ... other fields
}
```

**Step 3**: Create `views/text_view.rs` with table rendering logic:

```rust
// views/text_view.rs
pub struct TextView;

impl TextView {
    pub fn render(data: &EnvironmentListData) -> Result<String, Error> {
        // Table rendering logic moved here
    }
}
```

#### Rationale

Preparing for JSON output (task 12.5) requires DTO + Views pattern.

#### Benefits

- ✅ Ready for JSON output format
- ✅ Consistent with create/provision commands
- ✅ Clear separation of data vs rendering
- ✅ Easier to test rendering logic independently

#### Implementation Checklist

- [x] Analyze current `environment_list.rs` structure
- [x] Extract data structures → `view_data/environment_list.rs` (N/A - data already in application layer)
- [x] Extract table rendering → `views/text_view.rs`
- [x] Update `mod.rs` with new module structure
- [x] Update imports in controller
- [x] Update imports in tests
- [x] Verify all tests pass
- [x] Run linter and fix any issues

#### Testing Strategy

```bash
# Verify existing table output unchanged
cargo test list

# Verify full suite
cargo test && cargo run --bin linter all
```

---

### Proposal #3: Refactor Show Command Structure

**Status**: ✅ Completed
**Impact**: 🟢🟢 Medium
**Effort**: 🔵🔵 Medium
**Priority**: P1
**Depends On**: Proposals #0, #1
**Completed**: 2026-02-16
**Commit**: 68676e16

#### Problem

The `show` command has nested `environment_info/` with multiple data structures but no formal view separation:

```rust
src/presentation/views/commands/show/
├── environment_info/             (VIEW DATA - Multiple DTOs)
│   ├── basic.rs
│   ├── infrastructure.rs
│   ├── tracker_services.rs
│   ├── prometheus.rs
│   ├── grafana.rs
│   ├── https_hint.rs
│   ├── next_step.rs
│   └── mod.rs
└── mod.rs
```

View rendering is likely inline in the controller or mixed with data structures.

#### Proposed Solution

Preserve nested structure but organize under `view_data/`:

```rust
src/presentation/views/commands/show/
├── view_data/
│   └── environment_info/            (keep nested structure)
│       ├── basic.rs
│       ├── infrastructure.rs
│       ├── tracker_services.rs
│       ├── prometheus.rs
│       ├── grafana.rs
│       ├── https_hint.rs
│       ├── next_step.rs
│       └── mod.rs
├── views/
│   └── text_view.rs                 (extract from controller)
└── mod.rs
```

**Step 1**: Analyze controller to find rendering logic

**Step 2**: Extract to `views/text_view.rs`

**Step 3**: Move `environment_info/` → `view_data/environment_info/`

#### Rationale

Show command will need JSON output (task 12.3), requiring DTO + Views pattern.

#### Benefits

- ✅ Ready for JSON output format
- ✅ Consistent with other commands
- ✅ Rendering logic separated from data
- ✅ Nested structure preserved for logical grouping

#### Implementation Checklist

- [x] Analyze controller for rendering logic (views already separated)
- [x] Create `view_data/` and `views/` subdirectories (only views/ needed)
- [x] Move `environment_info/` → `views/` (renamed directory with existing structure)
- [x] Rename `EnvironmentInfoView` to `TextView`
- [x] Update `mod.rs` with new module structure
- [x] Update imports in controller
- [x] Update imports in tests
- [x] Verify all tests pass
- [x] Run linter and fix any issues

#### Testing Strategy

```bash
cargo test show
cargo test && cargo run --bin linter all
```

---

## Phase 2: Documentation and Shared Components

### Proposal #4: Update Shared Components

**Status**: ❌ Not Needed
**Impact**: 🟢 Low
**Effort**: 🔵 Low
**Priority**: P2
**Depends On**: Proposals #0-#3
**Decision Date**: 2026-02-16

#### Problem

Shared components in `shared/service_urls/` may need reorganization:

```rust
src/presentation/views/commands/shared/
└── service_urls/                 (MIXED - Shared view components)
    ├── compact.rs
    ├── dns_hint.rs
    └── mod.rs
```

These are reusable view components but the naming could be clearer.

#### Analysis

After reviewing the code, the current structure is already well-organized:

1. **Clear Naming**: `service_urls/` is semantically meaningful - these helpers render service URLs
2. **Pure View Helpers**: Both components (`compact.rs`, `dns_hint.rs`) are pure view helpers with no need for data/view separation
3. **Minimal Usage**: Only used by `run` command controller - minimal churn from changes
4. **No Real Benefit**: Proposed `view_components/` is more generic and actually less clear than current name

#### Decision Rationale

**Current structure is optimal** because:

- ✅ Namespace `service_urls/` clearly conveys purpose
- ✅ Files `compact.rs` and `dns_hint.rs` are descriptive
- ✅ Pure view helpers - no DTOs to separate
- ✅ Proposed rename would add churn without meaningful improvement

**Proposed structure drawbacks**:

- ❌ `view_components/` is too generic (loses semantic meaning)
- ❌ Would break imports in `run` controller unnecessarily
- ❌ No architectural benefit gained

#### Implementation Checklist

- [x] Analyze current shared components
- [x] Conclude no changes needed
- [x] Update refactor plan with rationale

#### Testing Strategy

```bash
cargo test
cargo run --bin linter all
```

---

## 📈 Timeline

- **Start Date**: 2026-02-16
- **Estimated Duration**: 2-3 hours
- **Target Completion**: 2026-02-16 (same day)
- **Actual Completion**: 2026-02-16

## 🔍 Review Process

### Approval Criteria

- [x] Technical feasibility validated (structure is straightforward)
- [x] Aligns with [Development Principles](../development-principles.md) (clear separation of concerns)
- [x] Implementation plan is clear and actionable
- [x] Priorities are correct (foundation commands first)
- [ ] Approved by maintainer

### Completion Criteria

- [x] All 5 proposals reviewed (4 implemented, 1 discarded as not needed)
- [x] All 396 tests passing
- [x] All linters passing (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
- [x] No unused dependencies (cargo machete)
- [x] Module-level documentation updated
- [ ] Architecture documentation updated (`docs/codebase-architecture.md`) - if needed
- [ ] Contributing guide updated (`docs/contributing/ddd-layer-placement.md`) - if needed
- [ ] Changes reviewed and approved
- [ ] Changes merged to main branch

## 📚 Related Documentation

- [EPIC #348 - Add JSON Output Format Support](../issues/348-epic-add-json-output-format-support.md)
- [Issue #352 - Add JSON output to provision command](../issues/352-add-json-output-to-provision-command.md) ✅ Completed
- [Issue #349 - Add JSON output to create command](../issues/349-add-json-output-to-create-command.md) ✅ Completed
- [Development Principles](../development-principles.md)
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)
- [Codebase Architecture](../codebase-architecture.md)

## 💡 Notes

### Naming Rationale

**Why `view_data/` instead of:**

- ❌ `models/` - Could confuse with domain models; MVVM terminology implies presentation logic (which we don't have yet)
- ❌ `dto/` - Emphasizes "transfer" aspect over "presentation" purpose
- ❌ `data/` - Too generic; doesn't convey presentation context
- ✅ `view_data/` - **Precise**: These are data structures specifically prepared for view rendering
  - Clear that it's presentation-layer data (not domain data)
  - Room to evolve: If we add presentation logic later, can rename to `view_models/`

### Future Evolution

If these data structures gain presentation logic (computed properties, formatting methods), they will evolve from "view data" to "view models" (true MVVM pattern). At that point, the directory can be renamed from `view_data/` to `view_models/`.

Current state: DTOs with data only (precise naming: `view_data/`)
Future state: DTOs with presentation logic (rename to: `view_models/`)

---

**Created**: 2026-02-16
**Last Updated**: 2026-02-16
**Status**: ✅ Complete
