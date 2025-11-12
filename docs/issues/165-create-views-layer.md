# Create Views Layer

**Issue**: #165
**Parent Epic**: #154 - Presentation Layer Reorganization
**Related**: [Refactor Plan - Proposal #4](../refactors/plans/presentation-layer-reorganization.md)

## Overview

Rename `src/presentation/user_output/` to `src/presentation/views/` to follow standard MVC terminology and complete the four-layer presentation architecture. This proposal establishes the final layer in our Input → Dispatch → Controllers → Views pattern.

## Goals

- [ ] Rename `user_output/` directory to `views/` following MVC conventions
- [ ] Update all import statements across the codebase to use `presentation::views`
- [ ] Update module documentation to reflect Views layer terminology
- [ ] Maintain all existing functionality and organization within the renamed directory
- [ ] Complete the four-layer presentation architecture

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation  
**Module Path**: `src/presentation/views/`  
**Pattern**: Views Layer (MVC Architecture)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../codebase-architecture.md))
- [ ] Maintain Views layer responsibilities only (output formatting, no business logic)
- [ ] Preserve existing well-organized submodule structure (`messages/`, `formatters/`, etc.)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../contributing/module-organization.md))

### Architectural Constraints

- [ ] Views layer contains only output formatting and user interface logic
- [ ] No business logic in views layer (preserve existing clean separation)
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../contributing/error-handling.md))
- [ ] Maintain dual-channel strategy (stdout for results, stderr for progress)

### Anti-Patterns to Avoid

- ❌ Adding business logic to views during refactoring
- ❌ Breaking the existing clean channel separation
- ❌ Changing functionality during the rename (scope creep)

## Specifications

### Current State Analysis

The `user_output/` directory is already well-organized with a clean structure:

```text
src/presentation/user_output/
├── channel.rs           # Output channel abstractions (stdout/stderr)
├── core.rs             # Main UserOutput struct
├── formatters/         # Output formatting logic
│   ├── json.rs         # JSON output formatter
│   └── mod.rs          # Formatter abstractions
├── messages/           # Message type definitions
│   ├── error.rs        # Error messages
│   ├── info_block.rs   # Information blocks
│   ├── progress.rs     # Progress messages
│   ├── result.rs       # Result messages
│   ├── steps.rs        # Step status messages
│   ├── success.rs      # Success messages
│   ├── warning.rs      # Warning messages
│   └── mod.rs          # Message type re-exports
├── sinks/              # Output destination abstractions
│   └── mod.rs          # Output sink definitions
├── test_support.rs     # Testing utilities
├── theme.rs            # Visual theming
├── traits.rs           # Core traits
├── verbosity.rs        # Verbosity level management
└── mod.rs              # Module documentation and re-exports
```

**Assessment**: The current structure is excellent and follows Views layer best practices. The rename is purely cosmetic to align with MVC terminology.

### Target State

After refactoring:

```text
src/presentation/views/
├── channel.rs           # Same functionality
├── core.rs             # Same functionality
├── formatters/         # Same functionality
├── messages/           # Same functionality
├── sinks/              # Same functionality
├── test_support.rs     # Same functionality
├── theme.rs            # Same functionality
├── traits.rs           # Same functionality
├── verbosity.rs        # Same functionality
└── mod.rs              # Updated documentation terminology
```

### Import Path Changes

All imports will change from:

```rust
use crate::presentation::views::{UserOutput, VerbosityLevel};
```

To:

```rust
use crate::presentation::views::{UserOutput, VerbosityLevel};
```

### Documentation Updates

Update module documentation to use Views terminology:

- Replace "user-facing output handling" with "Views layer - user interface output"
- Replace "user_output" references with "views" in code comments
- Align terminology with MVC pattern documentation
- Reference the four-layer presentation architecture

## Implementation Plan

### Phase 1: Directory Rename and Basic Updates (30 minutes)

- [ ] Rename `src/presentation/user_output/` to `src/presentation/views/`
- [ ] Update `src/presentation/mod.rs` to export `views` instead of `user_output`
- [ ] Update main module documentation in `src/presentation/views/mod.rs`

### Phase 2: Import Statement Updates (60 minutes)

- [ ] Update all import statements in `src/presentation/` modules
- [ ] Update all import statements in `src/application/` layer
- [ ] Update all import statements in `src/infrastructure/` layer
- [ ] Update all import statements in `src/bin/` executables
- [ ] Update import statements in `src/main.rs` and `src/lib.rs`

### Phase 3: Test Updates (30 minutes)

- [ ] Update import statements in all test files
- [ ] Update test helper imports and utilities
- [ ] Verify all tests pass after import changes

### Phase 4: Documentation and Integration (30 minutes)

- [ ] Update `src/presentation/mod.rs` architecture documentation
- [ ] Update any remaining documentation references
- [ ] Run full linting and testing suite
- [ ] Verify compilation and functionality

## Acceptance Criteria

### Functional Requirements

- [ ] Directory renamed from `user_output/` to `views/` with all files preserved
- [ ] All import statements updated to use `presentation::views` path
- [ ] Module documentation updated to reflect Views layer terminology
- [ ] All existing functionality preserved (no behavior changes)
- [ ] Four-layer presentation architecture completed (Input → Dispatch → Controllers → Views)

### Technical Requirements

- [ ] All compilation errors resolved from import path changes
- [ ] All tests pass with updated import paths
- [ ] No dead code or unused imports introduced
- [ ] Module exports properly structured in new `views/mod.rs`

### Documentation Requirements

- [ ] `src/presentation/mod.rs` updated to document Views layer
- [ ] `src/presentation/views/mod.rs` uses Views terminology consistently
- [ ] Architecture documentation reflects completed four-layer structure
- [ ] All inline documentation updated for new terminology

### Quality Checks

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`
  - ✅ No unused dependencies (`cargo machete`)
  - ✅ All linters pass (markdown, yaml, toml, clippy, rustfmt, shellcheck)
  - ✅ All unit tests pass (`cargo test`)
  - ✅ Documentation builds successfully (`cargo doc`)
  - ✅ All E2E tests pass (config, provision, full suite)

## Related Documentation

- [Refactor Plan - Proposal #4](../refactors/plans/presentation-layer-reorganization.md#proposal-4-create-views-layer)
- [Codebase Architecture](../codebase-architecture.md)
- [DDD Layer Placement Guide](../contributing/ddd-layer-placement.md)
- [Module Organization](../contributing/module-organization.md)
- [Epic #154 - Presentation Layer Reorganization](./154-epic-presentation-layer-reorganization.md)

## Notes

### Why This Approach?

1. **Minimal Risk**: Pure rename with no functional changes
2. **Standard Terminology**: Aligns with industry MVC conventions
3. **Architecture Completion**: Completes the four-layer presentation pattern
4. **Foundation for Future**: Prepares for `progress.rs` integration in Proposal 5

### Estimated Time Breakdown

- **Directory Rename**: 5 minutes
- **Import Updates**: 60 minutes (automated search/replace)
- **Testing**: 30 minutes
- **Documentation**: 30 minutes
- **Quality Verification**: 25 minutes
- **Total**: ~2.5 hours

### Risk Assessment

**Low Risk**: This is a pure rename operation with no functional changes. The existing `user_output/` structure is well-organized and follows Views layer best practices.

**Main Risks**:

- Missing import statements (mitigated by comprehensive testing)
- Documentation inconsistencies (mitigated by systematic review)

**Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.
