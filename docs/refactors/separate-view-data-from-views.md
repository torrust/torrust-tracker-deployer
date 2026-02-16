# Refactor: Separate View Data from Views in Presentation Layer

**Created**: 2026-02-16
**Status**: Planned
**Priority**: High (before continuing EPIC #348)
**Estimated Effort**: 2-3 hours

## Context

The presentation layer currently mixes two distinct responsibilities in the same directory level:

1. **View Data** - Data structures (DTOs) that bridge domain → presentation
2. **Views** - Rendering logic that transforms data → user output

This refactor establishes a clear, scalable structure before implementing 3 additional commands with JSON output (EPIC #348: tasks 12.3, 12.4, 12.5).

## Current State Analysis

### Directory Structure

```
src/presentation/views/commands/
├── create/
│   ├── environment_details.rs        (VIEW DATA - DTO)
│   ├── text_view.rs                  (VIEW - Text rendering)
│   ├── json_view.rs                  (VIEW - JSON rendering)
│   └── mod.rs
├── provision/
│   ├── provision_details.rs          (VIEW DATA - DTO)
│   ├── connection_details.rs         (VIEW DATA - Helper DTO)
│   ├── dns_reminder.rs               (VIEW DATA - Helper DTO)
│   ├── text_view.rs                  (VIEW - Text rendering)
│   ├── json_view.rs                  (VIEW - JSON rendering)
│   └── mod.rs
├── list/
│   ├── environment_list.rs           (MIXED - View with embedded data)
│   └── mod.rs
├── show/
│   ├── environment_info/             (VIEW DATA - Multiple DTOs)
│   │   ├── basic.rs
│   │   ├── infrastructure.rs
│   │   ├── tracker_services.rs
│   │   ├── prometheus.rs
│   │   ├── grafana.rs
│   │   ├── https_hint.rs
│   │   ├── next_step.rs
│   │   └── mod.rs
│   └── mod.rs
└── shared/
    └── service_urls/                 (MIXED - Shared view components)
        ├── compact.rs
        ├── dns_hint.rs
        └── mod.rs
```

### Classification

**View Data (DTOs):**

- `create/environment_details.rs` - DTO with data only
- `provision/provision_details.rs` - DTO with data only
- `provision/connection_details.rs` - Helper DTO for text view sub-components
- `provision/dns_reminder.rs` - Helper DTO for text view sub-components
- `show/environment_info/*` - Multiple DTOs for show command sections

**Views:**

- `create/text_view.rs` - Renders EnvironmentDetailsData as text
- `create/json_view.rs` - Renders EnvironmentDetailsData as JSON
- `provision/text_view.rs` - Renders ProvisionDetailsData as text
- `provision/json_view.rs` - Renders ProvisionDetailsData as JSON

**Mixed (Need Analysis):**

- `list/environment_list.rs` - View with inline data structure
- `shared/service_urls/*` - Shared view components (may contain data)

## Goals

1. **Clear Separation**: View data and views in separate subdirectories
2. **Consistent Structure**: All commands follow the same organizational pattern
3. **Scalability**: Easy to add new commands (show, run, list with JSON output)
4. **Discoverability**: New contributors immediately understand where code belongs
5. **Precision**: Folder names reflect current state (not aspirational MVVM)

## Proposed Structure

```
src/presentation/views/commands/
├── create/
│   ├── view_data/
│   │   └── environment_details.rs
│   ├── views/
│   │   ├── text_view.rs
│   │   └── json_view.rs
│   └── mod.rs
├── provision/
│   ├── view_data/
│   │   ├── provision_details.rs          (main DTO)
│   │   ├── connection_details.rs         (helper DTO)
│   │   └── dns_reminder.rs               (helper DTO)
│   ├── views/
│   │   ├── text_view.rs
│   │   └── json_view.rs
│   └── mod.rs
├── list/
│   ├── view_data/
│   │   └── environment_list.rs           (extract data from current file)
│   ├── views/
│   │   └── text_view.rs                  (extract view from current file)
│   └── mod.rs
├── show/
│   ├── view_data/
│   │   └── environment_info/            (keep nested structure)
│   │       ├── basic.rs
│   │       ├── infrastructure.rs
│   │       ├── tracker_services.rs
│   │       ├── prometheus.rs
│   │       ├── grafana.rs
│   │       ├── https_hint.rs
│   │       ├── next_step.rs
│   │       └── mod.rs
│   ├── views/
│   │   └── text_view.rs                 (to be created - currently inline)
│   └── mod.rs
└── shared/
    └── view_components/                  (renamed from service_urls)
        ├── service_urls.rs
        ├── dns_hints.rs
        └── mod.rs
```

## Naming Rationale

**Why `view_data/` instead of:**

- ❌ `models/` - Could confuse with domain models; MVVM terminology implies presentation logic (which we don't have yet)
- ❌ `dto/` - Emphasizes "transfer" aspect over "presentation" purpose
- ❌ `data/` - Too generic; doesn't convey presentation context
- ✅ `view_data/` - **Precise**: These are data structures specifically prepared for view rendering
  - Clear that it's presentation-layer data (not domain data)
  - Room to evolve: If we add presentation logic later, can rename to `view_models/`

**Why `views/` (plural):**

- Standard convention (multiple view implementations)
- Parallel with `view_data/` (both plural)
- Clear distinction from parent `views/` module at `src/presentation/views/`

## Implementation Plan

### Phase 1: Create Commands (create, provision)

These already follow the DTO + Views pattern, just need reorganization.

**Tasks:**

1. **Create `view_data/` and `views/` subdirectories**
2. **Move files into subdirectories**
3. **Update module re-exports in `mod.rs`**
4. **Update imports across codebase**
5. **Run tests to verify no breakage**

**Commands:** `create`, `provision`

### Phase 2: List Command

Currently has one file `environment_list.rs` that mixes data + view logic.

**Tasks:**

1. **Analyze `environment_list.rs`** - Identify data vs view logic
2. **Extract data structure** → `view_data/environment_list.rs`
3. **Extract view logic** → `views/text_view.rs`
4. **Create `view_data/` and `views/` subdirectories**
5. **Update module re-exports**
6. **Update imports**
7. **Run tests**

### Phase 3: Show Command

Has nested `environment_info/` with multiple data structures, but no formal view separation.

**Tasks:**

1. **Keep nested structure** - Move `environment_info/` → `view_data/environment_info/`
2. **Extract view rendering** - Create `views/text_view.rs` (likely inline in controller currently)
3. **Create subdirectories**
4. **Update imports**
5. **Run tests**

### Phase 4: Shared Components

Contains reusable view components (service URLs, DNS hints).

**Tasks:**

1. **Rename directory** - `service_urls/` → `view_components/`
2. **Analyze each file** - Separate data vs rendering if needed
3. **Update imports across codebase**
4. **Run tests**

### Phase 5: Documentation & Verification

**Tasks:**

1. **Update architecture documentation** - Reflect new structure in `docs/codebase-architecture.md`
2. **Update contributing guide** - `docs/contributing/ddd-layer-placement.md`
3. **Update module-level documentation** - Update `mod.rs` files with new structure
4. **Run full test suite** - Verify all 2200+ tests pass
5. **Run all linters** - `cargo run --bin linter all`
6. **Check for unused dependencies** - `cargo machete`

## Benefits

### Before (Current State)

```rust
// Ambiguous: Is this a DTO or a View?
use crate::presentation::views::commands::provision::provision_details::ProvisionDetailsData;
use crate::presentation::views::commands::provision::text_view::TextView;
```

### After (Refactored)

```rust
// Clear semantic distinction
use crate::presentation::views::commands::provision::view_data::ProvisionDetailsData;
use crate::presentation::views::commands::provision::views::{TextView, JsonView};
```

**Clarity gains:**

- ✅ Immediately obvious which imports are data vs rendering
- ✅ Consistent pattern across all commands
- ✅ Easier to review PRs (clear where code belongs)
- ✅ Faster onboarding for new contributors

## Risks & Mitigation

### Risk 1: Breaking Changes

**Impact**: High - Many imports will change
**Mitigation**:

- Comprehensive test suite ensures no behavior changes
- Compiler will catch all import errors
- One command at a time (incremental approach)

### Risk 2: Increased Verbosity

**Impact**: Low - Import paths are longer
**Mitigation**:

- Module re-exports can provide convenience paths
- Most imports are in controllers (relatively few call sites)
- Clarity benefits outweigh verbosity cost

### Risk 3: Inconsistency During Transition

**Impact**: Medium - Codebase will be inconsistent during refactor
**Mitigation**:

- Complete refactor in one PR (don't leave half-done)
- Estimated 2-3 hours - manageable in single session
- Use atomic git commits per phase

## Testing Strategy

**Per Phase:**

1. Move files
2. Update imports
3. Run `cargo build` - verify compilation
4. Run `cargo test` - verify behavior unchanged
5. Run `cargo run --bin linter all` - verify quality standards
6. Git commit with descriptive message

**Final Verification:**

```bash
# Full test suite
cargo test

# All linters
cargo run --bin linter all

# No unused dependencies
cargo machete

# Pre-commit checks
./scripts/pre-commit.sh
```

## Success Criteria

- [ ] All commands follow `view_data/` + `views/` structure
- [ ] All 2200+ tests pass
- [ ] All linters pass (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
- [ ] No unused dependencies (cargo-machete)
- [ ] Documentation updated
- [ ] Module-level docs updated
- [ ] Import paths consistent across codebase
- [ ] Git history shows clean, atomic commits per phase

## Timeline

**Estimated Duration**: 2-3 hours (focused session)

**Breakdown:**

- Phase 1 (create, provision): 45 minutes
- Phase 2 (list): 30 minutes
- Phase 3 (show): 30 minutes
- Phase 4 (shared): 15 minutes
- Phase 5 (docs, verification): 30 minutes
- Buffer: 30 minutes

**Dependencies**: None (can start immediately)
**Blocking**: Implementation of EPIC #348 tasks 12.3-12.5

## Next Steps

1. **Review this plan** - Get approval before implementation
2. **Create feature branch** - `refactor/separate-view-data-from-views`
3. **Implement Phase 1** - Start with `create` and `provision` commands
4. **Continue through phases** - One phase at a time with test verification
5. **Create PR** - Single PR with all changes for atomic merge
6. **Merge to main** - After review and CI passes
7. **Continue EPIC #348** - Implement tasks 12.3-12.5 with new structure

## Alternatives Considered

### Alternative 1: Keep Current Structure

**Pros**: No refactoring cost, works fine for current scale
**Cons**: Will cause friction with 3+ more commands, technical debt accumulates

**Decision**: Rejected - EPIC #348 is confirmed, better to establish pattern now

### Alternative 2: Full MVVM (use `models/` with presentation logic)

**Pros**: Industry-standard terminology, extensible if we add logic later
**Cons**: Aspirational naming doesn't match current state (DTOs have no behavior)

**Decision**: Rejected - Prefer precision over aspiration; can rename later if DTOs evolve

### Alternative 3: Flat Structure with Naming Convention

**Pros**: Simpler file organization, fewer directories
**Cons**: Doesn't scale well, still requires mental parsing of file purpose

**Decision**: Rejected - Clear directory structure better than naming convention alone

## Related Documentation

- [EPIC #348 - Add JSON Output Format Support](../issues/348-epic-add-json-output-format-support.md)
- [Issue #352 - Add JSON output to provision command](../issues/352-add-json-output-to-provision-command.md) ✅ Completed
- [Issue #349 - Add JSON output to create command](../issues/349-add-json-output-to-create-command.md) ✅ Completed
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)
- [Codebase Architecture](../codebase-architecture.md)

## Approval

- [ ] Plan reviewed and approved
- [ ] Ready for implementation

---

**Note**: This refactor is a prerequisite for continuing EPIC #348. Establishing this structure now will make implementing tasks 12.3 (show), 12.4 (run), and 12.5 (list) significantly easier and more consistent.
