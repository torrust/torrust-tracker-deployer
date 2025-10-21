# EPIC: App Layer Destroy Command

**Epic Type**: Child Epic #9 (Phase 1 of Task 1.2)
**Parent Epic**: #8 ([`8-epic-destroy-command.md`](./8-epic-destroy-command.md))
**Related Roadmap**: [Section 1.2](../roadmap.md#1-add-scaffolding-for-main-app)
**Parent Issue**: #2 (Scaffolding for main app)  
**Roadmap Section**: [1.2 - Create command `torrust-tracker-deployer destroy`](../roadmap.md#12-create-command-torrust-tracker-deployer-destroy)  
**Type**: Epic  
**Priority**: High

---

## 📋 Epic Overview

Implement the core destroy command functionality in the DDD Application Layer. This epic focuses on the internal business logic for tearing down deployed environments, without the user-facing CLI interface.

This follows an **inside-outside approach**: build the core application logic first, validate it with E2E tests, then add the UI layer in a separate epic.

## 🎯 Goals

1. Add `DestroyCommand` in the DDD Application Layer
2. Implement resource cleanup logic for all infrastructure components
3. Update E2E test infrastructure to use the new destroy command
4. Ensure clean teardown of OpenTofu and Ansible-managed resources
5. Provide comprehensive testing and documentation

## 🚫 Non-Goals

- User-facing CLI interface (handled in Epic #10)
- CLI command structure refactoring (handled in Epic #10)
- Advanced UI/UX features (handled in Epic #10)

## 📦 Sub-Issues

### Issue #11: Add DestroyCommand in Application Layer

**Description**: Create the `DestroyCommand` struct and implementation in `src/application/commands/destroy/` with complete infrastructure teardown functionality.

**Scope**:

- Create command structure following existing command patterns (`ProvisionCommand`, `ConfigCommand`)
- Implement complete destroy execution logic using existing infrastructure services
- Add command error types with proper error handling
- Leverage existing OpenTofu client (already used in manual cleanup)
- Leverage existing Ansible services (if needed for cleanup)
- Integrate destroy command into `src/bin/e2e_tests_full.rs` for immediate testing

**Rationale**: We already have the infrastructure teardown logic in `src/testing/e2e/tasks/virtual_machine/cleanup_infrastructure.rs` that calls OpenTofu client and other services. This subtask combines command creation with infrastructure teardown because they're closely coupled and we need the full functionality to test it properly.

**Acceptance Criteria**:

- [ ] `DestroyCommand` exists in `src/application/commands/destroy/`
- [ ] Command follows DDD Application Layer patterns
- [ ] Complete infrastructure teardown logic implemented (OpenTofu destroy, state cleanup, build directory cleanup)
- [ ] Proper error types defined with `thiserror`
- [ ] Error handling for partial failures (following existing cleanup patterns)
- [ ] Unit tests for command logic
- [ ] Integration with `src/bin/e2e_tests_full.rs` to test the feature locally
- [ ] Code follows project conventions (module organization, error handling)

**Estimated Effort**: 6-8 hours

---

### Issue #12: Update E2E Provision Tests to Include Destroy

**Description**: Modify `src/bin/e2e_provision_tests.rs` binary to use the new destroy command instead of manual cleanup, ensuring CI can run complete provision+destroy cycles.

**Scope**:

- Replace manual cleanup in `e2e_provision_tests.rs` with `DestroyCommand`
- Keep manual cleanup as fallback for destroy command failures
- Ensure the binary works in GitHub Actions environment
- Update binary name to reflect new functionality (optional: rename to `e2e_provision_and_destroy_tests.rs`)

**Rationale**: This ensures we have automated testing of the destroy functionality in CI, preventing regressions. The provision tests are already running real OpenTofu infrastructure, so adding destroy testing here gives us confidence the feature works end-to-end.

**Acceptance Criteria**:

- [ ] `e2e_provision_tests.rs` uses `DestroyCommand` for cleanup
- [ ] Manual cleanup preserved as fallback for destroy failures
- [ ] Binary works correctly in GitHub Actions
- [ ] Proper error handling and logging for destroy operations
- [ ] CI configuration updated if binary is renamed
- [ ] E2E test documentation updated

**Estimated Effort**: 3-4 hours

---

### Issue #13: Add Developer Documentation

**Description**: Document the destroy command implementation for developers.

**Scope**:

- Add section to `docs/contributing/` about destroy command
- Document command architecture and design decisions
- Add examples of using `DestroyCommand` in code
- Document error handling patterns
- Update E2E testing documentation with new destroy functionality

**Acceptance Criteria**:

- [ ] Developer documentation created in `docs/contributing/`
- [ ] Architecture and design decisions documented
- [ ] Code examples provided for using `DestroyCommand`
- [ ] Error handling patterns documented
- [ ] E2E testing guide updated with destroy functionality
- [ ] All markdown linting passes

**Estimated Effort**: 2-3 hours

---

## 📊 Epic Summary

**Total Estimated Effort**: 11-15 hours

**Sub-Issues**:

1. Issue #11: Add DestroyCommand in Application Layer (6-8h)
2. Issue #12: Update E2E Provision Tests to Include Destroy (3-4h)
3. Issue #13: Add Developer Documentation (2-3h)

## 🔗 Dependencies

- Requires completed: Epic/Issue #2 (Scaffolding for main app)
- Blocks: Epic #10 (UI Layer Destroy Command)

## 📝 Technical Notes

### Design Considerations

1. **State Management**: How do we track which resources exist and need destruction?
2. **Idempotency**: Can we safely run destroy multiple times?
3. **Partial Failures**: How do we handle cases where some resources are destroyed but others fail?
4. **Orphaned Resources**: How do we detect and handle resources that exist but aren't in the state?

### Open Questions

1. Should destroy command remove state files or preserve them for auditing?
2. Should we validate the environment exists before attempting destruction?
3. How do we handle concurrent access (e.g., another process provisioning while we destroy)?

### Testing Strategy

- **Unit Tests**: Command logic, error handling, state transitions
- **Integration Tests**: OpenTofu and Ansible integration
- **E2E Tests**:
  - **Full Tests** (`e2e_tests_full.rs`): Complete lifecycle including destroy (implemented in subtask 1)
  - **VM Tests** (`e2e_provision_tests.rs`): Provision + destroy cycle on real infrastructure (updated in subtask 2)
  - **Container Tests** (`e2e_config_tests.rs`): Configuration only (no destroy needed)
- **Failure Tests**: Partial destruction scenarios, error recovery, fallback to manual cleanup

## 🚀 Next Steps After Completion

After completing this epic:

1. Review and validate the application layer implementation
2. Proceed to Epic #10: UI Layer Destroy Command
3. Consider future improvements:
   - Force flag for automated environments
   - Skip confirmation flag
   - Selective resource destruction
   - Destroy specific environments by name

---

## 📋 Related Documentation

- [Roadmap](../roadmap.md)
- [Parent Issue #2](https://github.com/torrust/torrust-tracker-deployer/issues/2)
- [Development Principles](../development-principles.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
