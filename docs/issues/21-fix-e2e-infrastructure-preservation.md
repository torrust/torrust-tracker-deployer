# Fix E2E Infrastructure Preservation

**GitHub Issue**: [#21](https://github.com/torrust/torrust-tracker-deployer/issues/21)
**Type**: Task  
**Priority**: High  
**Parent Epic**: #10 ([`10-epic-ui-layer-destroy-command.md`](./10-epic-ui-layer-destroy-command.md))
**Estimated Effort**: 1-2 hours

---

## 📋 Issue Overview

Restore the `--keep` flag functionality in E2E test binaries to preserve infrastructure for manual testing workflows. This is a prerequisite for testing the destroy CLI command implementation.

## 🎯 Problem Statement

During E2E binary refactoring, the new `run_infrastructure_destroy` function doesn't respect the `--keep` CLI argument. This breaks the ability to preserve test infrastructure for manual verification, which is essential for testing the destroy CLI command.

### Current Broken Workflow

```bash
# This should preserve infrastructure but doesn't
cargo run --bin e2e-tests-full -- --keep
cargo run --bin e2e-provision-and-destroy-tests -- --keep

# Infrastructure gets destroyed despite --keep flag
```

### Required Manual Testing Workflow

```bash
# 1. Provision infrastructure and keep it
cargo run --bin e2e-provision-and-destroy-tests -- --keep

# Verify infrastructure is preserved
ls data/e2e-provision/          # Should exist
ls build/e2e-provision/         # Should exist
lxc list | grep e2e-provision   # Should show VM running

# 2. Manually test destroy CLI command (when implemented)
torrust-tracker-deployer destroy e2e-provision

# 3. Verify complete cleanup using LXD commands (see docs/tech-stack/lxd.md)
ls data/e2e-provision/          # Should not exist
ls build/e2e-provision/         # Should not exist
lxc list | grep e2e-provision   # Should return nothing (VM destroyed)
```

## 🎯 Goals

1. Fix `--keep` flag functionality in both E2E test binaries
2. Ensure `run_infrastructure_destroy` function respects the `keep` parameter
3. Enable reliable manual testing workflow for destroy CLI command
4. Maintain existing E2E test behavior when `--keep` is not used

## 📦 Scope

### Files to Fix

- `src/bin/e2e_tests_full.rs` - Main E2E test binary
- `src/bin/e2e_provision_and_destroy_tests.rs` - Provision/destroy E2E binary
- Any helper functions that handle infrastructure cleanup

### Key Components

1. **CLI Argument Parsing**: Ensure `--keep` flag is properly parsed and passed through
2. **Infrastructure Destroy Function**: Make `run_infrastructure_destroy` respect the `keep` parameter
3. **Conditional Cleanup**: Only destroy infrastructure when `keep=false`
4. **Test Verification**: Ensure existing E2E tests still work correctly

## 🔧 Implementation Plan

### Step 1: Audit Current `--keep` Implementation

```bash
# Check current CLI argument handling
grep -r "keep" src/bin/e2e_*.rs

# Check infrastructure destroy function signatures
grep -r "run_infrastructure_destroy" src/
```

### Step 2: Fix CLI Argument Passing

Ensure the `--keep` flag is properly passed from CLI arguments to the infrastructure destroy function.

**Expected Pattern**:

```rust
// CLI argument parsing
let keep_infrastructure = matches.get_flag("keep");

// Pass to destroy function
if !keep_infrastructure {
    run_infrastructure_destroy(&context).await?;
}
```

### Step 3: Fix Infrastructure Destroy Function

Ensure `run_infrastructure_destroy` function checks the `keep` parameter before proceeding with cleanup.

**Expected Signature**:

```rust
async fn run_infrastructure_destroy(
    context: &TestContext,
    keep: bool
) -> Result<(), Box<dyn Error>> {
    if keep {
        info!("Keeping infrastructure due to --keep flag");
        return Ok(());
    }

    // Proceed with destruction...
}
```

### Step 4: Update Both E2E Binaries

Apply fixes to both:

- `e2e_tests_full.rs`
- `e2e_provision_and_destroy_tests.rs`

Ensure consistent behavior across both binaries.

## ✅ Acceptance Criteria

### Functional Requirements

- [ ] `--keep` flag in `e2e_tests_full.rs` preserves infrastructure as intended
- [ ] `--keep` flag in `e2e_provision_and_destroy_tests.rs` preserves infrastructure as intended
- [ ] `run_infrastructure_destroy` function respects the `keep` parameter correctly
- [ ] Manual testing workflow works:
  - [ ] Run E2E test with `--keep` → infrastructure preserved (data/, build/, LXD VM)
  - [ ] Can manually test destroy CLI command on preserved infrastructure
  - [ ] Can verify complete cleanup after manual destroy command using LXD commands
  - [ ] LXD VM verification works: `lxc list | grep e2e-provision` shows VM when preserved, nothing when destroyed
- [ ] All E2E tests still pass when `--keep` flag is not used (default behavior unchanged)

### Technical Requirements

- [ ] No breaking changes to existing E2E test APIs
- [ ] Consistent `--keep` flag behavior across all E2E binaries
- [ ] Proper error handling for infrastructure preservation scenarios
- [ ] Logging messages clearly indicate when infrastructure is being preserved

### Testing Requirements

- [ ] Unit tests for CLI argument parsing with `--keep` flag
- [ ] Integration test verifying infrastructure preservation
- [ ] E2E test verifying default destruction behavior (no `--keep`)
- [ ] Manual testing verification:

  ```bash
  # Test with --keep
  cargo run --bin e2e-provision-and-destroy-tests -- --keep
  # Verify infrastructure exists in data/ and build/
  lxc list | grep e2e-provision  # Should show VM running

  # Test without --keep (default)
  cargo run --bin e2e-provision-and-destroy-tests
  # Verify infrastructure is cleaned up
  lxc list | grep e2e-provision  # Should return nothing
  ```

## 🚫 Non-Goals

- Changing the existing E2E test infrastructure beyond the `--keep` flag fix
- Adding new CLI flags or options
- Modifying the core infrastructure provisioning logic
- Implementing the actual destroy CLI command (that's Issue 10.3)

## 🔍 Testing Strategy

### Manual Verification Steps

1. **Test Infrastructure Preservation**:

   ```bash
   # Run with --keep
   cargo run --bin e2e-provision-and-destroy-tests -- --keep

   # Verify infrastructure preserved
   ls -la data/e2e-provision/     # Should exist
   ls -la build/e2e-provision/    # Should exist

   # Verify LXD VM still exists (see docs/tech-stack/lxd.md)
   lxc list | grep e2e-provision  # Should show the VM as RUNNING
   ```

2. **Test Default Cleanup**:

   ```bash
   # Run without --keep
   cargo run --bin e2e-provision-and-destroy-tests

   # Verify infrastructure cleaned up
   ls data/e2e-provision/         # Should not exist
   ls build/e2e-provision/        # Should not exist

   # Verify LXD VM is destroyed (see docs/tech-stack/lxd.md)
   lxc list | grep e2e-provision  # Should return nothing (VM destroyed)
   ```

3. **Test Both Binaries**:

   ```bash
   # Test both E2E binaries with --keep
   cargo run --bin e2e-tests-full -- --keep
   cargo run --bin e2e-provision-and-destroy-tests -- --keep

   # Both should preserve infrastructure
   ```

### Automated Testing

- Run existing E2E test suite to ensure no regressions
- Add specific tests for `--keep` flag functionality
- Verify CLI argument parsing with and without `--keep`

## 📋 Implementation Checklist

### Phase 1: Analysis

- [ ] Audit current `--keep` flag implementation in E2E binaries
- [ ] Identify where `run_infrastructure_destroy` is called
- [ ] Map the flow from CLI argument to destruction function

### Phase 2: Implementation

- [ ] Fix CLI argument passing in `e2e_tests_full.rs`
- [ ] Fix CLI argument passing in `e2e_provision_and_destroy_tests.rs`
- [ ] Update `run_infrastructure_destroy` to respect `keep` parameter
- [ ] Add proper logging for infrastructure preservation

### Phase 3: Testing

- [ ] Manual testing of `--keep` flag functionality
- [ ] Manual testing of default destruction behavior
- [ ] Run full E2E test suite to verify no regressions
- [ ] Document the corrected manual testing workflow

### Phase 4: Documentation

- [ ] Update E2E testing documentation if needed
- [ ] Update troubleshooting guides with correct `--keep` usage
- [ ] Ensure manual testing workflow is properly documented

## 🔗 Related Issues

- **Parent Epic**: #10 (UI Layer Destroy Command)
- **Follows**: Epic #9 (App Layer Destroy Command) - completed
- **Blocks**: Issue 10.3 (Add Clap Subcommand Configuration) - needs manual testing workflow

## 📚 Related Documentation

- [E2E Testing Guide](../e2e-testing.md)
- [Development Principles](../development-principles.md)
- [Manual Testing Procedures](../user-guide/)

## 🚀 Success Criteria

This issue is complete when:

1. **`--keep` flag works correctly** in both E2E binaries
2. **Manual testing workflow is restored** for destroy CLI command development
3. **Existing E2E tests continue to pass** without the `--keep` flag
4. **Infrastructure preservation is reliable** and predictable

The successful completion of this issue enables the development team to properly test the destroy CLI command implementation in subsequent issues.
