# Phase 2: State Transition Observability

**Status**: ✅ COMPLETED  
**Priority**: CRITICAL  
**Complexity**: Low  
**Dependencies**: Phase 1 (Foundation)

## 📋 Overview

Implement automatic logging for all environment state transitions to ensure complete observability of the environment lifecycle. This phase adds transparent, structured logging at the core `with_state()` helper method, ensuring every state change is captured without requiring manual logging in each transition method.

## 🎯 Objectives

1. **Automatic Logging**: Log all state transitions at info level with timestamps
2. **Structured Context**: Include environment name, instance name, and state information
3. **Zero-Touch Integration**: No changes required to existing state transition methods
4. **Traceability**: Enable complete lifecycle reconstruction from logs

## 📝 Requirements (from requirements-analysis.md)

From line 196:

> Log all state transitions at info level with timestamps, including:
>
> - Environment name
> - Previous state
> - New state
> - Instance name (when available)
> - Timestamp (automatically provided by tracing)

## 🔧 Technical Approach

### Implementation Location

Modify the existing `with_state<T>()` helper method in `src/domain/environment/mod.rs` (currently lines 192-217).

### Logging Strategy

```rust
impl<S> Environment<S> {
    fn with_state<T>(self, new_state: T) -> Environment<T> {
        // Add structured logging before state transition
        tracing::info!(
            environment_name = %self.name,
            instance_name = %self.instance_name,
            from_state = std::any::type_name::<S>(),
            to_state = std::any::type_name::<T>(),
            "Environment state transition"
        );

        Environment {
            name: self.name,
            instance_name: self.instance_name,
            profile_name: self.profile_name,
            ssh_credentials: self.ssh_credentials,
            build_dir: self.build_dir,
            data_dir: self.data_dir,
            state: new_state,
        }
    }
}
```

### Key Technical Decisions

1. **Use `tracing::info!`**: Leverages existing tracing infrastructure with automatic timestamp
2. **Type Names for States**: Use `std::any::type_name::<T>()` to get state type names
3. **Structured Fields**: Use `tracing` field syntax for parseable logs
4. **Display Trait**: Use `%` formatter for types implementing `Display`
5. **Single Log Point**: All transitions logged from one location (with_state helper)

## 🧪 Testing Strategy

### Unit Tests

Add tests in `src/domain/environment/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[traced_test]
    #[test]
    fn it_should_log_state_transition_from_created_to_provisioning() {
        let env = Environment::new(
            EnvironmentName::new("test-env".to_string()).unwrap(),
            ssh_credentials,
        );

        let _provisioning = env.start_provisioning();

        // Assert log contains expected fields
        assert!(logs_contain("Environment state transition"));
        assert!(logs_contain("environment_name=test-env"));
        assert!(logs_contain("from_state=Created"));
        assert!(logs_contain("to_state=Provisioning"));
    }

    #[traced_test]
    #[test]
    fn it_should_log_state_transition_with_instance_name() {
        let env = Environment::new(
            EnvironmentName::new("test-env".to_string()).unwrap(),
            ssh_credentials,
        );

        let _provisioning = env.start_provisioning();

        assert!(logs_contain("instance_name=torrust-tracker-deployer-test-env"));
    }

    #[traced_test]
    #[test]
    fn it_should_log_failed_state_transitions() {
        let env: Environment<Provisioning> = /* ... */;

        let _failed = env.mark_provision_failed("Timeout".to_string());

        assert!(logs_contain("to_state=ProvisionFailed"));
    }
}
```

### E2E Test Verification

Existing E2E tests will automatically capture state transition logs. Verify log output in:

- `cargo run --bin e2e-provision-and-destroy-tests`
- `cargo run --bin e2e-config-and-release-tests`
- `cargo run --bin e2e-tests-full`

Expected log output:

```text
2025-01-30T10:15:23.456789Z  INFO environment_name=e2e-test instance_name=torrust-tracker-deployer-e2e-test from_state=Created to_state=Provisioning: Environment state transition
```

## 📦 Deliverables

1. **Code Changes**:

   - [x] Update `with_state()` method in `src/domain/environment/mod.rs`
   - [x] Add `tracing::info!` with structured fields
   - [x] Use `std::any::type_name::<T>()` for state names

2. **Tests**:

   - [x] Add unit tests for state transition logging (4 tests added)
   - [x] Test all state transitions (success paths)
   - [ ] E2E tests capture transition logs (blocked by Phase 5 - Command Integration)

3. **Documentation**:
   - [ ] Update logging-guide.md with state transition examples (deferred)
   - [ ] Document log format and fields in user-guide (deferred)

## 📅 Commit Strategy

Single atomic commit after completion:

```bash
git add src/domain/environment/mod.rs
git commit -m "feat: add automatic state transition logging

Implement structured logging for all environment state transitions
at the with_state() helper method level, ensuring complete lifecycle
observability without requiring manual logging in each transition.

Changes:
- Add tracing::info! call in with_state() method
- Include environment name, instance name, and state types in logs
- Use std::any::type_name for automatic state name extraction
- Add unit tests for state transition logging

Addresses critical requirement from requirements-analysis.md:
'Log all state transitions at info level with timestamps'"
```

## ✅ Acceptance Criteria

- [x] All state transitions automatically logged at info level
- [x] Logs include: environment name, instance name, from_state, to_state
- [x] Timestamps automatically provided by tracing infrastructure
- [x] No changes required to existing state transition methods
- [x] Unit tests verify logging for all transition types (4 tests added)
- [ ] E2E tests show transition logs in output (blocked by Phase 5 - Command Integration)
- [ ] Documentation updated with log format examples (deferred)

## 🔗 Related Documentation

- [Development Principles](../../../development-principles.md) - Observability and Traceability
- [Logging Guide](../../contributing/logging-guide.md) - Logging conventions
- [Requirements Analysis](../requirements-analysis.md) - State transition logging requirement
- [Phase 1: Foundation](./phase-1-foundation.md) - with_state() helper implementation

## 💡 Implementation Notes

### Why This is CRITICAL and Easy

1. **CRITICAL**: State transitions are the core of environment lifecycle management. Without logging, we lose visibility into what happened when things fail.

2. **EASY**: Single method modification, leverages existing tracing infrastructure, no complex logic.

3. **HIGH ROI**: Minimal effort (< 1 hour) for maximum observability benefit.

### Future Enhancements (Out of Scope)

- Metrics collection (state transition duration, frequency)
- State transition hooks for custom actions
- Rollback logging with reason tracking
- State transition event streaming

These are deferred to future phases after core persistence (Phase 4) is implemented.
