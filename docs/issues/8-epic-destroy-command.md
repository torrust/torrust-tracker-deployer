# EPIC: Implement `destroy` Command for Infrastructure Cleanup

**GitHub Issue**: [#8](https://github.com/torrust/torrust-tracker-deployer/issues/8)  
**Epic Type**: Parent Epic (Task 1.2 from Roadmap)
**Related Roadmap**: [Section 1.2](../roadmap.md#1-add-scaffolding-for-main-app)
**Parent Issue**: #2 (Scaffolding for main app)

---

## 📋 Overview

This epic coordinates the implementation of the `torrust-tracker-deployer destroy` command, which enables users to cleanly tear down provisioned infrastructure and remove all associated resources. The implementation follows an **inside-outside approach**, building core functionality first (App Layer) before adding user-facing features (UI Layer).

## 🎯 Business Value

- **Resource Management**: Enable users to completely remove infrastructure when no longer needed
- **Cost Optimization**: Prevent orphaned resources that continue consuming cloud resources
- **Clean Testing**: Allow E2E tests to properly clean up after themselves
- **Development Workflow**: Support rapid iteration by enabling quick environment reset

## 📐 Implementation Strategy

### Approach: Inside-Outside Development

We follow an **inside-outside** approach prioritizing core functionality before UI polish:

1. **Phase 1**: App Layer (Business Logic)

   - Implement destroy functionality in the application/domain layers
   - Add comprehensive E2E tests
   - Ensure reliable infrastructure cleanup

2. **Phase 2**: UI Layer (User Interface)
   - Add CLI command with proper argument parsing
   - Implement user-friendly progress messages
   - Add safety features (confirmation, force flags)

### Why Inside-Outside?

- ✅ No real users yet (pre-MVP stage)
- ✅ Enables thorough testing of core functionality
- ✅ Consistent with our existing development strategy
- ✅ Reduces risk by validating business logic first
- ✅ Allows incremental delivery of value

## 🔄 Child EPICs

This parent epic is composed of two sequential child EPICs:

### 1. App Layer Destroy Command

**Document**: [`9-epic-app-layer-destroy-command.md`](./9-epic-app-layer-destroy-command.md)  
**Status**: Not Started  
**Dependencies**: None (first to implement)

Implements the core destroy functionality in the DDD Application Layer, including:

- Destroy command implementation
- E2E test integration
- Resource cleanup logic
- State management

### 2. UI Layer Destroy Command

**Document**: [`10-epic-ui-layer-destroy-command.md`](./10-epic-ui-layer-destroy-command.md)  
**Status**: Not Started  
**Dependencies**: Child Epic #9 must be completed first

Implements the user-facing CLI interface, including:

- CLI command structure refactoring
- Subcommand implementation
- User progress feedback
- Safety features (confirmation, force flags)

## 🧪 E2E Testing Strategy

The destroy command must be tested against **real infrastructure**, not Docker containers:

### Current E2E Architecture

- **Full Tests** (`e2e_tests_full.rs`): Complete lifecycle on local VMs
- **Provision Tests** (`e2e_provision_tests.rs`): OpenTofu VM provisioning
- **Config Tests** (`e2e_config_tests.rs`): Ansible config on containers

### Updated E2E Architecture

- **Full Tests** (`e2e_tests_full.rs`): Complete lifecycle + destroy on local VMs
- **VM Tests** (`e2e_provision_and_destroy_tests.rs`): Provision + destroy cycle
- **Config Tests** (`e2e_config_tests.rs`): Config only (no destroy needed)

### Why This Approach?

- ✅ Tests destroy command against real OpenTofu-managed infrastructure
- ✅ Docker containers auto-cleanup (no destroy command needed)
- ✅ Maintains separation between VM and container testing
- ✅ Enables CI testing of provision+destroy cycle

## 📊 Success Criteria

### Technical Criteria

- [ ] App Layer destroy command successfully removes all infrastructure
- [ ] E2E tests use destroy command for real VM cleanup
- [ ] CLI command provides clear user feedback
- [ ] Destroy operations are safe (confirmation required by default)
- [ ] All resources are properly cleaned up (no orphans)
- [ ] Documentation covers both developer and end-user use cases

### Quality Criteria

- [ ] All linters pass
- [ ] Unit test coverage for new code
- [ ] Integration tests validate end-to-end workflow
- [ ] E2E tests demonstrate real infrastructure cleanup
- [ ] Documentation is clear and comprehensive

## 🗺️ Roadmap Integration

This epic implements **Task 1.2** from the main roadmap:

```markdown
### 1.2. Create command `torrust-tracker-deployer destroy` (Child EPICs)
```

**Related Issues**:

- Parent: #2 (Scaffolding for main app)
- Child EPICs: #9 (App Layer), #10 (UI Layer)

## ⚠️ Risk Management

### Identified Risks

1. **Resource Orphaning**: Destroy command might fail to clean up all resources

   - **Mitigation**: Comprehensive testing, idempotent cleanup, error recovery

2. **Accidental Deletion**: Users might destroy production environments

   - **Mitigation**: Confirmation prompts, environment naming conventions, `--force` flag

3. **Partial Failures**: Infrastructure cleanup might partially fail

   - **Mitigation**: Cleanup verification, detailed error reporting, retry mechanisms

4. **State Inconsistency**: Environment state might become inconsistent during destroy
   - **Mitigation**: Atomic operations where possible, state validation, rollback support

## 📝 Implementation Notes

### Incremental Delivery

Each child epic delivers independently valuable functionality:

1. **After Epic #9**: Developers can use destroy command in code and tests
2. **After Epic #10**: End users can use destroy command from CLI

### Lean Approach

We prioritize essential functionality first, treating advanced features as improvements:

- **Essential**: Core destroy logic, basic CLI command
- **Improvements**: Force flags, skip confirmation, detailed progress

### Testing Strategy

- **Unit Tests**: Validate individual components
- **Integration Tests**: Verify command interactions
- **E2E Tests**: Validate full infrastructure lifecycle (provision → configure → destroy)

## 🔗 Related Documentation

- [Development Principles](../development-principles.md)
- [Contributing Guidelines](../contributing/README.md)
- [Roadmap](../roadmap.md)
- [Testing Conventions](../contributing/testing.md)
- [Creating Roadmap Issues](../contributing/roadmap-issues.md)

## 📅 Timeline

**Estimated Total Effort**: TBD (to be refined during child epic creation)

- **Epic #9 (App Layer)**: https://github.com/torrust/torrust-tracker-deployer/issues/9
- **Epic #10 (UI Layer)**: https://github.com/torrust/torrust-tracker-deployer/issues/10

## 🎯 Definition of Done

This parent epic is complete when:

- [ ] Both child EPICs are implemented and merged
- [ ] `torrust-tracker-deployer destroy` command works end-to-end
- [ ] E2E tests use destroy command for cleanup
- [ ] Documentation is complete (developer + user guides)
- [ ] All acceptance criteria are met
- [ ] Parent issue #2 task 1.2 is marked complete

---

**Created**: 2025-10-21  
**Status**: Planning  
**Next Steps**: Create GitHub issues for both child EPICs and link them to parent issue #2
