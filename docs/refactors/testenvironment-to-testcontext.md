# Refactor Plan: TestEnvironment to TestContext

## Overview

Rename `TestEnvironment` to `TestContext` throughout the codebase to avoid naming conflicts with the planned multi-environment deployment feature. This refactor will also rename the file `src/e2e/environment.rs` to `src/e2e/context.rs`.

## Scope

### Files to Modify

1. **Primary Module**

   - `src/e2e/environment.rs` → `src/e2e/context.rs`

2. **Files Using TestEnvironment** (found via compilation errors)

   - `src/e2e/tasks/container/preflight_cleanup.rs`
   - `src/e2e/tasks/container/run_provision_simulation.rs`
   - `src/e2e/tasks/preflight_cleanup.rs`
   - `src/e2e/tasks/run_configure_command.rs`
   - `src/e2e/tasks/run_test_command.rs`
   - `src/e2e/tasks/virtual_machine/cleanup_infrastructure.rs`
   - `src/e2e/tasks/virtual_machine/preflight_cleanup.rs`
   - `src/e2e/tasks/virtual_machine/run_provision_command.rs`

3. **Module Exports**
   - `src/e2e/mod.rs` - Update module declaration and re-exports

### Renames Required

| Current Name             | New Name             |
| ------------------------ | -------------------- |
| `TestEnvironment`        | `TestContext`        |
| `TestEnvironmentError`   | `TestContextError`   |
| `TestEnvironmentType`    | `TestContextType`    |
| `environment_type` field | `context_type` field |
| `src/e2e/environment.rs` | `src/e2e/context.rs` |

## Step-by-Step Plan

### Phase 1: Rename Main Types in environment.rs

1. **Rename struct**: `TestEnvironment` → `TestContext`
2. **Rename error enum**: `TestEnvironmentError` → `TestContextError`
3. **Rename type enum**: `TestEnvironmentType` → `TestContextType`
4. **Update field names**: `environment_type` → `context_type`
5. **Fix impl blocks**: Update all `impl TestEnvironment` to `impl TestContext`
6. **Update documentation**: Change all references in doc comments
7. **Update error messages**: Change "environment" to "context" in error strings where appropriate

### Phase 2: Update Import Statements

For each file in the "Files Using TestEnvironment" list:

1. Change `use crate::e2e::environment::TestEnvironment;` to `use crate::e2e::context::TestContext;`
2. Update any usage of `TestEnvironment` to `TestContext`
3. Update any usage of `TestEnvironmentError` to `TestContextError`
4. Update any usage of `TestEnvironmentType` to `TestContextType`

### Phase 3: Rename File and Update Module System

1. **Rename file**: `src/e2e/environment.rs` → `src/e2e/context.rs`
2. **Update module declaration**: In `src/e2e/mod.rs`:
   - Change `pub mod environment;` → `pub mod context;`
   - Update any re-exports from `environment` to `context`

### Phase 4: Update Import Paths

Update all import paths from:

- `crate::e2e::environment::*` → `crate::e2e::context::*`

### Phase 5: Verification

1. **Compile**: Ensure `cargo build` succeeds
2. **Test**: Run `cargo test` to ensure no test breakage
3. **Lint**: Run `cargo run --bin linter all` to ensure code quality
4. **E2E Tests**: Run `cargo run --bin e2e-tests-full` to verify functionality

## Expected Changes Summary

- **8 files** with import statement updates
- **1 file** renamed (`environment.rs` → `context.rs`)
- **1 module file** updated (`src/e2e/mod.rs`)
- **3 type names** changed throughout codebase
- **Documentation and comments** updated for clarity

## Risk Assessment

**Low Risk**: This is a pure rename refactor with no logic changes.

**Potential Issues**:

- Import path mismatches if any imports are missed
- Test breakage if test code uses the old names
- Documentation references that need updating

**Mitigation**:

- Use compiler errors to guide the refactor
- Run full test suite after completion
- Use search tools to find any remaining references

## Testing Strategy

1. **Compilation Check**: Fix all compilation errors step by step
2. **Unit Tests**: Ensure all existing tests pass with new names
3. **E2E Tests**: Verify end-to-end functionality works
4. **Integration Tests**: Check that all components work together

## Post-Refactor Validation

After completion, verify:

- [ ] All files compile without errors
- [ ] All tests pass
- [ ] No references to old names remain in codebase
- [ ] Documentation is consistent with new naming
- [ ] E2E tests function correctly with renamed types

