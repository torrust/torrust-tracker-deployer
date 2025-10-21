# Rename and Update E2E Provision Tests to Include Destroy

**Issue Type**: Sub-issue (9.2)  
**Parent Epic**: #9 ([`epic-app-layer-destroy-command.md`](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/issues/9-epic-app-layer-destroy-command.md))  
**Related Roadmap**: [Section 1.2](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md#12-create-command-torrust-tracker-deployer-destroy)  
**Dependencies**: Issue 9.1 (Add DestroyCommand in Application Layer) must be completed first  
**Priority**: High  
**Estimated Effort**: 3-4 hours

---

## 📋 Issue Overview

Modify and rename `src/bin/e2e_provision_tests.rs` to `src/bin/e2e_provision_and_destroy_tests.rs`, updating it to use the new destroy command instead of manual cleanup, ensuring CI can run complete provision+destroy cycles.

This ensures we have automated testing of the destroy functionality in CI, preventing regressions. The provision tests are already running real OpenTofu infrastructure, so adding destroy testing here gives us confidence the feature works end-to-end.

## 🎯 Goals

1. Replace manual cleanup with `DestroyCommand` in E2E provision tests
2. Maintain fallback to manual cleanup for destroy command failures
3. Ensure compatibility with GitHub Actions CI environment
4. Provide comprehensive E2E testing of the full provision+destroy lifecycle

## 📦 Scope

### Core Changes

- Rename `e2e_provision_tests.rs` to `e2e_provision_and_destroy_tests.rs`
- Replace manual cleanup with `DestroyCommand`
- Keep manual cleanup as fallback for destroy command failures
- Ensure the binary works in GitHub Actions environment

### Integration Points

1. **DestroyCommand Integration**: Use the new application layer command
2. **Error Handling**: Graceful fallback to manual cleanup on destroy failures
3. **CI Compatibility**: Ensure tests work in GitHub Actions
4. **Logging**: Proper error handling and logging for destroy operations

## 🏗️ Technical Design

### Test Flow Integration

Current flow:

```text
Provision → Manual Cleanup
```

New flow:

```text
Provision → DestroyCommand → (Fallback: Manual Cleanup on failure)
```

### Error Handling Strategy

```rust
// Pseudo-code for destroy integration
match destroy_command.execute(&environment).await {
    Ok(_) => {
        info!("Environment destroyed successfully using DestroyCommand");
    }
    Err(e) => {
        warn!("DestroyCommand failed: {e}, falling back to manual cleanup");
        manual_cleanup(&environment).await?;
    }
}
```

### Binary Renaming

Rename `e2e_provision_tests.rs` to `e2e_provision_and_destroy_tests.rs` to reflect the expanded functionality:

- More descriptive of actual test coverage
- Clearly indicates full lifecycle testing
- Aligns with the enhanced scope
- Better represents the complete provision+destroy workflow

## 📋 Acceptance Criteria

- [ ] Binary renamed from `e2e_provision_tests.rs` to `e2e_provision_and_destroy_tests.rs`
- [ ] Binary uses `DestroyCommand` for cleanup instead of manual cleanup
- [ ] Manual cleanup preserved as fallback for destroy failures
- [ ] Binary works correctly in GitHub Actions
- [ ] Proper error handling and logging for destroy operations
- [ ] CI configuration updated with new binary name
- [ ] E2E test documentation updated with new binary name and functionality

## 🧪 Testing Strategy

### Local Testing

- Run the updated binary locally to verify destroy command integration
- Test both success and failure scenarios
- Validate fallback to manual cleanup works correctly

### CI Testing

- Ensure the updated binary works in GitHub Actions
- Verify no regressions in existing provision test functionality
- Confirm destroy command is properly tested in CI environment

### Error Scenarios

- Test destroy command failures and fallback behavior
- Validate proper logging and error reporting
- Ensure manual cleanup still works as expected

## 🔗 Dependencies

- **Requires**: Issue 9.1 (Add DestroyCommand in Application Layer) - must be completed first
- **Requires**: Existing E2E provision test infrastructure
- **Blocks**: Issue 9.3 (Developer Documentation)

## 📝 Implementation Notes

### Current E2E Provision Test Structure

The current `src/bin/e2e_provision_tests.rs` binary:

1. Provisions infrastructure using OpenTofu
2. Validates deployment
3. Performs manual cleanup using existing cleanup functions

After this issue, the renamed `src/bin/e2e_provision_and_destroy_tests.rs` binary will:

1. Provision infrastructure using OpenTofu
2. Validate deployment
3. Destroy infrastructure using `DestroyCommand` (with manual cleanup fallback)

### Integration Approach

1. **Rename Binary**: Move `e2e_provision_tests.rs` to `e2e_provision_and_destroy_tests.rs`
2. **Import DestroyCommand**: Add the new command from the application layer
3. **Replace Cleanup Logic**: Use `DestroyCommand` instead of manual cleanup functions
4. **Add Fallback**: Preserve manual cleanup as backup for destroy failures
5. **Update Logging**: Enhance logging to show destroy command usage
6. **Test Integration**: Validate the integration works in both local and CI environments

### CI Configuration Updates

Required updates for the binary rename:

- Update `.github/workflows/` references from `e2e_provision_tests` to `e2e_provision_and_destroy_tests`
- Update documentation references to the new binary name
- Update any scripts that call the binary
- Update `Cargo.toml` binary configuration if needed

## 🚀 Next Steps

After completing this issue:

1. Validate E2E tests work in both local and CI environments
2. Document the updated E2E testing approach in Issue 9.3
3. Consider expanding destroy testing to other E2E test binaries

## 📊 Related Documentation

- [Parent Epic #9](https://github.com/torrust/torrust-tracker-deployer/issues/9)
- [E2E Testing Guide](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/e2e-testing.md)
- [GitHub Actions Configuration](https://github.com/torrust/torrust-tracker-deployer/tree/main/.github/workflows)
- [Development Principles](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/development-principles.md)

---

**Issue Document**: [docs/issues/epic-app-layer-destroy-command-update-e2e-provision-tests.md](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/issues/epic-app-layer-destroy-command-update-e2e-provision-tests.md)
