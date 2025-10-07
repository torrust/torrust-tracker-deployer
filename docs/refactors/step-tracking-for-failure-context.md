# Refactor: Step Tracking for Failure Context

**Status**: Proposed  
**Date**: 2025-10-07  
**Author**: AI Assistant (with user guidance)

## Context

Currently, when provisioning or configuration fails, the `build_failure_context()` methods in both `ProvisionCommand` and `ConfigureCommand` must perform "reverse engineering" by pattern matching on error types to determine which step was executing when the failure occurred.

This approach has several issues:

- **Complexity**: Requires maintaining error-to-step mapping logic
- **Fragility**: If errors change or new steps are added, the mapping must be updated
- **Indirection**: The actual step information is inferred rather than directly tracked
- **Maintenance burden**: Pattern matching logic becomes complex with more steps

## Problem Statement

**Current Flow:**

```text
Command → Step 1 → Step 2 → Error!
                            ↓
Command ← build_failure_context() ← Error pattern matching ← Which step was it?
```

**The Issue:**

- Command executes steps sequentially
- Steps don't know their identity in the workflow
- When error occurs, command has lost track of which step was executing
- Must reverse-engineer from error type to step identifier

## Proposed Solution

Track the currently executing step explicitly in the command's execution flow. This provides direct access to step information when building failure context.

### Solution Architecture

**New Flow:**

```text
Command → Track: Step 1 → Execute: Step 1 → Success
       → Track: Step 2 → Execute: Step 2 → Error!
                                          ↓
Command ← build_failure_context() ← Current step known directly
```

### Implementation Strategy

1. **Add Current Step Tracking**: Commands maintain a "current step" variable
2. **Update Before Each Step**: Set current step before executing each step
3. **Use in Failure Context**: Reference current step when building failure context
4. **Remove Pattern Matching**: Eliminate error-to-step reverse engineering logic

### Key Design Decisions

- **Tracking Location**: Command-level (not in steps themselves)

  - Steps remain independent and reusable
  - Command controls workflow context
  - Flexible for future step reordering/composition

- **Step Identity Assignment**: Command assigns step identity

  - Steps don't need to know their enum value
  - No common trait required
  - Clean separation of concerns

- **Error Kind Handling**: Deferred to future refactor
  - This refactor focuses only on step tracking
  - Error kind classification remains as-is for now

## Implementation Plan

### Phase 1: Add Step Tracking Infrastructure

**File**: `src/application/commands/provision.rs`

1. Add `current_step` field to track execution progress
2. Create helper method to update current step with logging
3. Update step execution to track before executing

**Changes:**

```rust
// Add field to ProvisionCommand (inside execute method)
let mut current_step = ProvisionStep::RenderOpenTofuTemplates;

// Before each step execution
current_step = ProvisionStep::RenderOpenTofuTemplates;
self.render_opentofu_templates().await?;

current_step = ProvisionStep::OpenTofuInit;
// ... and so on
```

**File**: `src/application/commands/configure.rs`

Similar changes for `ConfigureCommand` with `ConfigureStep`.

### Phase 2: Simplify Failure Context Building

**File**: `src/application/commands/provision.rs`

1. Update `build_failure_context()` signature to accept `current_step`
2. Remove all pattern matching logic for step extraction
3. Directly use the provided `current_step` parameter
4. Keep error kind pattern matching for now (future refactor)

**Before:**

```rust
fn build_failure_context(
    &self,
    environment: &Environment<Provisioning>,
    error: &ProvisionCommandError,
    started_at: chrono::DateTime<chrono::Utc>,
) -> ProvisionFailureContext {
    let (failed_step, error_kind) = match error {
        ProvisionCommandError::OpenTofuTemplateRendering(_) => (
            ProvisionStep::RenderOpenTofuTemplates,
            ProvisionErrorKind::TemplateRendering,
        ),
        // ... more pattern matching
    };
    // ...
}
```

**After:**

```rust
fn build_failure_context(
    &self,
    environment: &Environment<Provisioning>,
    error: &ProvisionCommandError,
    current_step: ProvisionStep,  // ← Direct parameter
    started_at: chrono::DateTime<chrono::Utc>,
) -> ProvisionFailureContext {
    let error_kind = match error {  // ← Only classify error kind
        ProvisionCommandError::OpenTofuTemplateRendering(_) =>
            ProvisionErrorKind::TemplateRendering,
        // ... simpler pattern matching
    };

    let failed_step = current_step;  // ← Direct assignment
    // ...
}
```

**File**: `src/application/commands/configure.rs`

Similar simplification for `ConfigureCommand`.

### Phase 3: Update Error Handling in Execute Method

**File**: `src/application/commands/provision.rs`

Update the error handling block to pass `current_step`:

```rust
Err(e) => {
    let context = self.build_failure_context(
        &environment,
        &e,
        current_step,  // ← Pass tracked step
        started_at
    );
    let failed = environment.provision_failed(context);
    self.persist_provision_failed_state(&failed);
    Err(e)
}
```

### Phase 4: Update All Unit Tests

**Files:**

- `src/application/commands/provision.rs` (tests module)
- `src/application/commands/configure.rs` (tests module)

**Changes Required:**

1. Update test calls to `build_failure_context()` to pass `current_step` parameter
2. Remove tests that validate step extraction from error types (no longer needed)
3. Add new tests to verify step tracking works correctly
4. Update assertions to match new behavior

**Example Test Update:**

**Before:**

```rust
#[test]
fn it_should_build_failure_context_from_opentofu_template_error() {
    // ... setup ...
    let error = ProvisionCommandError::OpenTofuTemplateRendering(...);
    let context = command.build_failure_context(&environment, &error, started_at);
    assert_eq!(context.failed_step, ProvisionStep::RenderOpenTofuTemplates);
}
```

**After:**

```rust
#[test]
fn it_should_build_failure_context_with_provided_step() {
    // ... setup ...
    let error = ProvisionCommandError::OpenTofuTemplateRendering(...);
    let current_step = ProvisionStep::RenderOpenTofuTemplates;
    let context = command.build_failure_context(
        &environment,
        &error,
        current_step,
        started_at
    );
    assert_eq!(context.failed_step, ProvisionStep::RenderOpenTofuTemplates);
}
```

### Phase 5: Clean Up Obsolete Code

Remove the `extract_opentofu_step()` helper method and any other step extraction logic that's no longer needed.

## Benefits

### Immediate Benefits

✅ **Simplicity**: Direct step tracking eliminates reverse engineering  
✅ **Clarity**: Explicit step tracking makes execution flow visible  
✅ **Maintainability**: Adding/removing steps requires no mapping updates  
✅ **Reliability**: No risk of incorrect step inference from error types  
✅ **Observability**: Step tracking provides execution context in logs

### Future Benefits (Nice-to-Have)

🔮 **Enhanced User Feedback**: Show users which step failed with accurate timing  
🔮 **Step Retry Capability**: Foundation for implementing step-level retry logic  
🔮 **Progress Tracking**: Can show users real-time progress through steps  
🔮 **Performance Analysis**: Track duration of individual steps  
🔮 **Partial Resume**: Foundation for resuming from failed step

## Testing Strategy

### Unit Tests

1. **Step Tracking Verification**

   - Test that current step is correctly set before each step execution
   - Verify failure context includes the correct step

2. **Error Kind Classification**

   - Test that error kind is still correctly classified
   - Verify all error variants map to appropriate error kinds

3. **Integration with State Transitions**
   - Test that failed states receive correct step information
   - Verify trace files include accurate step data

### E2E Tests

Run full E2E test suite to ensure:

- Provision failures include correct step information
- Configure failures include correct step information
- Trace files contain accurate step tracking data

## Implementation Checklist

- [ ] Phase 1: Add step tracking in `ProvisionCommand::execute_provisioning_steps()`
- [ ] Phase 1: Add step tracking in `ConfigureCommand::execute_configuration_steps()`
- [ ] Phase 2: Simplify `ProvisionCommand::build_failure_context()`
- [ ] Phase 2: Simplify `ConfigureCommand::build_failure_context()`
- [ ] Phase 3: Update error handling to pass `current_step`
- [ ] Phase 4: Update all unit tests in `provision.rs`
- [ ] Phase 4: Update all unit tests in `configure.rs`
- [ ] Phase 5: Remove obsolete `extract_opentofu_step()` method
- [ ] Phase 5: Remove obsolete code comments
- [ ] Run linters: `cargo run --bin linter all`
- [ ] Run unit tests: `cargo test`
- [ ] Run E2E tests: `cargo run --bin e2e-tests-full`
- [ ] Review changes and commit

## Migration Path

### Backward Compatibility

Not applicable - project is in PoC phase with no production users.

### Breaking Changes

- `build_failure_context()` method signature changes (internal API only)
- Test helper functions may need updates

### Rollout Strategy

1. Implement changes in feature branch
2. Verify all tests pass
3. Review refactored code for clarity
4. Merge to main branch

## Future Enhancements

After this refactor is complete, consider:

1. **Error Kind Tracking**: Improve error kind classification (separate refactor)
2. **Step Retry Logic**: Add ability to retry failed steps
3. **Progress Reporting**: Show users step-by-step progress
4. **Step Duration Tracking**: Measure and report step execution times
5. **Partial Resume**: Resume provisioning from failed step

## Success Criteria

✅ All unit tests pass  
✅ All E2E tests pass  
✅ All linters pass  
✅ `build_failure_context()` no longer does pattern matching for step extraction  
✅ Step tracking is explicit and visible in code  
✅ Code is simpler and easier to understand  
✅ Trace files include correct step information

## Notes

- This refactor focuses exclusively on step tracking
- Error kind classification remains as-is (future work)
- Steps remain independent with no common trait
- Commands maintain full control over workflow context
