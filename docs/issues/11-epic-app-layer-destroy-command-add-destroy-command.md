# Add DestroyCommand in Application Layer

**Issue Type**: Sub-issue (9.1)  
**Parent Epic**: #9 ([`epic-app-layer-destroy-command.md`](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/issues/9-epic-app-layer-destroy-command.md))  
**Related Roadmap**: [Section 1.2](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/roadmap.md#12-create-command-torrust-tracker-deployer-destroy)  
**Priority**: High  
**Estimated Effort**: 6-8 hours

---

## 📋 Issue Overview

Create the `DestroyCommand` struct and implementation in `src/application/commands/destroy/` with complete infrastructure teardown functionality.

This is the core subtask that implements the business logic for destroying deployed environments. It follows the DDD Application Layer patterns established by existing commands like `ProvisionCommand` and `ConfigCommand`.

## 🎯 Goals

1. Create `DestroyCommand` following existing command patterns
2. Implement complete infrastructure teardown logic using existing services
3. Add proper error handling with `thiserror` integration
4. Integrate with E2E test infrastructure for immediate validation
5. Provide comprehensive unit testing

## 📦 Scope

### Core Implementation

- Create command structure following existing command patterns (`ProvisionCommand`, `ConfigCommand`)
- Implement complete destroy execution logic using existing infrastructure services
- Add command error types with proper error handling
- Leverage existing OpenTofu client (already used in manual cleanup)
- Leverage existing Ansible services (if needed for cleanup)
- Integrate destroy command into `src/bin/e2e_tests_full.rs` for immediate testing

### Infrastructure Integration

We already have the infrastructure teardown logic in `src/testing/e2e/tasks/virtual_machine/cleanup_infrastructure.rs` that calls:

- OpenTofu client for infrastructure destruction
- State file cleanup
- Build directory cleanup
- Other cleanup services

This subtask combines command creation with infrastructure teardown because they're closely coupled and we need the full functionality to test it properly.

## 🏗️ Technical Design

### Command Structure

Follow the established DDD Application Layer patterns:

```rust
// src/application/commands/destroy/mod.rs
pub mod command;
pub mod error;

pub use command::DestroyCommand;
pub use error::DestroyError;
```

### Error Handling

Define proper error types with `thiserror`:

```rust
#[derive(thiserror::Error, Debug)]
pub enum DestroyError {
    #[error("Environment '{environment}' not found")]
    EnvironmentNotFound { environment: String },

    #[error("OpenTofu destroy failed: {source}")]
    OpenTofuFailed { source: Box<dyn std::error::Error + Send + Sync> },

    #[error("State cleanup failed: {source}")]
    StateCleanupFailed { source: Box<dyn std::error::Error + Send + Sync> },

    // ... other error variants
}
```

### Integration Points

1. **OpenTofu Client**: Use existing `OpenTofuClient` for infrastructure destruction
2. **State Management**: Clean up state files and build directories
3. **E2E Integration**: Add to `src/bin/e2e_tests_full.rs` for immediate testing
4. **Error Recovery**: Handle partial failures following existing cleanup patterns

## 📋 Acceptance Criteria

- [ ] `DestroyCommand` exists in `src/application/commands/destroy/`
- [ ] Command follows DDD Application Layer patterns
- [ ] Complete infrastructure teardown logic implemented (OpenTofu destroy, state cleanup, build directory cleanup)
- [ ] Proper error types defined with `thiserror`
- [ ] Error handling for partial failures (following existing cleanup patterns)
- [ ] Unit tests for command logic
- [ ] Integration with `src/bin/e2e_tests_full.rs` to test the feature locally
- [ ] Code follows project conventions (module organization, error handling)

## 🧪 Testing Strategy

### Unit Tests

- Command initialization and configuration
- Error handling scenarios
- State management logic
- Mock integration with infrastructure services

### Integration Testing

- OpenTofu client integration
- State file cleanup operations
- Build directory cleanup
- Error recovery scenarios

### E2E Testing

- Full destroy workflow in `e2e_tests_full.rs`
- Real infrastructure teardown
- Validation that resources are completely removed

## 🔗 Dependencies

- **Requires**: Existing OpenTofu client implementation
- **Requires**: Existing cleanup infrastructure in `src/testing/e2e/tasks/virtual_machine/cleanup_infrastructure.rs`
- **Blocks**: Issue 9.2 (Update E2E Provision Tests)
- **Blocks**: Issue 9.3 (Developer Documentation)

## 📝 Implementation Notes

### Design Considerations

1. **Idempotency**: Can we safely run destroy multiple times?
2. **State Management**: How do we track which resources exist and need destruction?
3. **Partial Failures**: How do we handle cases where some resources are destroyed but others fail?
4. **Resource Discovery**: How do we detect and handle orphaned resources?

### Open Questions

1. Should destroy command remove state files or preserve them for auditing?
2. Should we validate the environment exists before attempting destruction?
3. How do we handle concurrent access (e.g., another process provisioning while we destroy)?

### Existing Infrastructure to Leverage

- `src/testing/e2e/tasks/virtual_machine/cleanup_infrastructure.rs` - Contains working teardown logic
- OpenTofu client implementation - Already handles infrastructure operations
- Error handling patterns from existing commands
- State management patterns from provision/config commands

## 🚀 Next Steps

After completing this issue:

1. Validate the implementation with E2E tests
2. Proceed to Issue 9.2: Update E2E Provision Tests
3. Document the implementation in Issue 9.3

## 📊 Related Documentation

- [Parent Epic #9](https://github.com/torrust/torrust-tracker-deployer/issues/9)
- [Development Principles](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/development-principles.md)
- [Error Handling Guide](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/error-handling.md)
- [Module Organization](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/module-organization.md)
- [Testing Conventions](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/contributing/testing.md)

---

**Issue Document**: [docs/issues/epic-app-layer-destroy-command-add-destroy-command.md](https://github.com/torrust/torrust-tracker-deployer/blob/main/docs/issues/epic-app-layer-destroy-command-add-destroy-command.md)
