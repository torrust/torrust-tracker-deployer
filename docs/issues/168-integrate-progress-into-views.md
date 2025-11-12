# Integrate Progress into Views Layer

**Issue**: #168
**Parent Epic**: #154 - Presentation Layer Reorganization
**Related**:

- Proposal 4 (Create Views Layer) - ✅ Completed
- [Refactor Plan](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/refactors/plans/presentation-layer-reorganization.md)

## Overview

Move the orphaned `src/presentation/progress.rs` module into the views layer as `src/presentation/views/progress/` module to establish clear ownership and complete the four-layer presentation architecture.

## Goals

- [ ] Move `src/presentation/progress.rs` to `src/presentation/views/progress/` module structure
- [ ] Update all import statements to use `presentation::views::progress`
- [ ] Integrate progress module into views layer documentation
- [ ] Maintain all existing functionality and API
- [ ] Establish clear ownership of progress indicators within the Views layer

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation
**Module Path**: `src/presentation/views/progress/`
**Pattern**: Views Layer Integration

### Module Structure Requirements

- [ ] Create `src/presentation/views/progress/` directory
- [ ] Move `progress.rs` to `progress/mod.rs` in new location
- [ ] Update `src/presentation/views/mod.rs` to re-export progress module
- [ ] Follow views layer organization patterns

### Architectural Rationale

Progress indicators are UI/output concerns that logically belong in the Views layer:

- Progress reporting is presentation logic for user feedback
- Builds on top of `UserOutput` (already in Views layer)
- No business logic - purely presentational
- Follows MVC pattern where progress bars/indicators are view components

### Anti-Patterns to Avoid

- ❌ Keeping progress as orphaned module at presentation root
- ❌ Creating circular dependencies between views and progress
- ❌ Breaking existing API contracts during the move

## Specifications

### Current State Analysis

**Current Location**: `src/presentation/progress.rs` (718 lines)
**Dependencies**:

- `std::*` (standard library)
- `parking_lot::ReentrantMutex`
- `thiserror::Error`
- `crate::presentation::views::UserOutput`

**Current Consumers**:

- Controller handlers (create, destroy)
- Controller error types
- Test module (`presentation/tests/reentrancy_fix_test.rs`)

### Target State

**New Location**: `src/presentation/views/progress/mod.rs`
**Updated Import Path**: `crate::presentation::views::progress::*`
**Module Structure**:

```text
src/presentation/views/
├── progress/           # 🆕 New directory
│   └── mod.rs          # 🔄 Moved from ../progress.rs
├── mod.rs              # 🔄 Updated to re-export progress
└── ...                 # Existing views structure
```

### API Preservation

The progress module API must remain unchanged:

- `ProgressReporter` struct and all its methods
- `ProgressReporterError` enum and error types
- All public functions and traits
- Documentation examples should work without modification

## Implementation Plan

### Phase 1: Directory Structure (15 minutes)

- [ ] Create `src/presentation/views/progress/` directory
- [ ] Move `src/presentation/progress.rs` to `src/presentation/views/progress/mod.rs`
- [ ] Update `src/presentation/views/mod.rs` to include and re-export progress module
- [ ] Update `src/presentation/mod.rs` to remove progress module (no longer at root)

### Phase 2: Import Statement Updates (30 minutes)

- [ ] Update controller handlers:
  - `src/presentation/controllers/create/subcommands/template/handler.rs`
  - `src/presentation/controllers/create/subcommands/environment/handler.rs`
  - `src/presentation/controllers/destroy/handler.rs`
- [ ] Update controller error types:
  - `src/presentation/controllers/create/subcommands/environment/errors.rs`
  - `src/presentation/controllers/create/subcommands/template/errors.rs`
  - `src/presentation/controllers/destroy/errors.rs`
- [ ] Update test file:
  - `src/presentation/tests/reentrancy_fix_test.rs`

### Phase 3: Documentation Updates (15 minutes)

- [ ] Update documentation examples in `progress/mod.rs` to use new import path
- [ ] Update `src/presentation/views/mod.rs` documentation to mention progress
- [ ] Verify all doctests compile with new import paths

### Phase 4: Verification (15 minutes)

- [ ] Run full test suite to verify functionality preserved
- [ ] Run linting to ensure no unused imports or dead code
- [ ] Verify compilation and all doctests pass

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

### Functional Requirements

- [ ] Progress module moved from `src/presentation/progress.rs` to `src/presentation/views/progress/mod.rs`
- [ ] All import statements updated to use `presentation::views::progress` path
- [ ] Progress module properly integrated into views layer module structure
- [ ] All existing functionality and API preserved (no breaking changes)
- [ ] Documentation examples updated to use new import path

### Technical Requirements

- [ ] All compilation errors resolved from import path changes
- [ ] All tests pass including updated imports
- [ ] All doctests compile and pass with new import paths
- [ ] No dead code or unused imports introduced
- [ ] Module exports properly structured in `views/mod.rs`

### Quality Checks

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
- [ ] No circular dependencies introduced
- [ ] Code follows project module organization conventions

### Integration Requirements

- [ ] Progress module listed in `src/presentation/views/mod.rs` exports
- [ ] Old location `src/presentation/progress.rs` completely removed
- [ ] `src/presentation/mod.rs` no longer exports progress (now in views)
- [ ] Views layer documentation mentions progress as sub-module

## Related Documentation

- [Presentation Layer Reorganization Plan](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/refactors/plans/presentation-layer-reorganization.md)
- [Module Organization Conventions](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/module-organization.md)
- [DDD Layer Placement Guide](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/ddd-layer-placement.md)

## Risk Assessment

**Low Risk**: This is a simple module relocation with import path updates. The progress module is well-isolated, has clear dependencies, and a stable API.

**Potential Issues**:

- Import path updates might be missed in some files
- Documentation examples may reference old paths

**Mitigation**:

- Systematic grep search for all references
- Comprehensive testing after changes
- Verification of doctest compilation

## Notes

### Why This Is Simple

- Progress module is well-isolated with clear boundaries
- Only depends on standard library, thiserror, parking_lot, and views/UserOutput
- Has a stable, documented API that doesn't need changes
- Limited number of consumer files to update

### Integration Benefits

- Establishes clear ownership (progress indicators are UI concerns)
- Completes the views layer as comprehensive output/formatting module
- Eliminates orphaned module at presentation root
- Follows standard MVC patterns (progress indicators are view components)

**Estimated Time**: 1-2 hours
