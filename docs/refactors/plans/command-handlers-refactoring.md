# Command Handlers Refactoring

**Issue**: [#61](https://github.com/torrust/torrust-tracker-deployer/issues/61)

## 📋 Overview

This refactoring addresses code quality issues in `src/application/command_handlers` by reducing duplication, improving maintainability, and enhancing testability while preserving existing behavior.

**Target Files:**

- `src/application/command_handlers/provision/handler.rs`
- `src/application/command_handlers/configure/handler.rs`
- `src/application/command_handlers/destroy/handler.rs`
- `src/application/command_handlers/create/handler.rs`
- `src/application/command_handlers/test.rs`

**Scope:**

- Extract common failure context building logic
- Reduce code duplication in step tracking and execution
- Standardize error handling patterns
- Improve testability by reducing pub(crate) exposure
- Simplify complex execute methods
- Maintain all existing behavior (no functional changes)

## 📊 Progress Tracking

**Total Active Proposals**: 7
**Total Postponed**: 2
**Total Discarded**: 2
**Completed**: 1
**In Progress**: 0
**Not Started**: 6

### Phase Summary

- **Phase 0 - Quick Wins (High Impact, Low Effort)**: ⏳ 1/3 completed (33%)
- **Phase 1 - Structural Improvements (High Impact, Medium Effort)**: ⏳ 0/2 completed (0%)
- **Phase 2 - Consistency & Polish (Medium Impact, Low Effort)**: ⏳ 0/2 completed (0%)

### Discarded Proposals

- **Create shared command handler base class**: Rust doesn't have inheritance, and trying to force a base class pattern would introduce unnecessary complexity. Using traits and helper functions is more idiomatic.
- **Extract execute method template with trait**: With only ~5 current commands and a maximum of 10 planned, the overhead of creating and maintaining a trait-based template pattern outweighs the benefits. Will reevaluate after implementing a couple more commands if the duplication becomes more problematic.

### Postponed Proposals

- **Macro-based step execution**: While this could reduce boilerplate, macros make debugging harder and reduce code readability. Revisit if duplication becomes significantly worse.
- **Generic state machine framework**: The current type-state pattern works well. A generic framework would add complexity without clear benefits. Revisit if we add many more commands.

## 🎯 Key Problems Identified

### 1. Code Duplication

The `build_failure_context` method is duplicated across provision, configure, and destroy handlers with ~90% identical code. The only differences are:

- The failure context type (ProvisionFailureContext vs ConfigureFailureContext vs DestroyFailureContext)
- The trace writer type (if applicable)

### 2. Complex Execute Methods

Execute methods have multiple responsibilities:

- State transitions
- State persistence
- Step execution orchestration
- Error handling with context building
- Logging

This violates Single Responsibility Principle and makes methods hard to test.

### 3. Inconsistent Patterns

- Create handler doesn't follow the same pattern as provision/configure/destroy
- Test handler has a completely different structure
- Some handlers have trace file generation, others don't

### 4. Testing Challenges

Methods like `build_failure_context` are marked `pub(crate)` solely for testing purposes, which exposes internal implementation details.

### 5. State Persistence Duplication

State persistence logic (`self.repository.save()`) is duplicated across handlers at multiple points (intermediate state, success state, error state).

## 🚀 Refactoring Phases

---

## Phase 0: Quick Wins (High Impact, Low Effort)

These improvements provide immediate value with minimal risk and effort.

### Proposal #0: Extract Common Failure Context Builder

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0

#### Problem

The `build_failure_context` method is duplicated across provision, configure, and destroy handlers:

```rust
// In provision/handler.rs
pub(crate) fn build_failure_context(
    &self,
    environment: &Environment<Provisioning>,
    error: &ProvisionCommandHandlerError,
    current_step: ProvisionStep,
    started_at: chrono::DateTime<chrono::Utc>,
) -> ProvisionFailureContext {
    let failed_step = current_step;
    let error_kind = error.error_kind();
    let now = self.clock.now();
    let trace_id = TraceId::new();
    let execution_duration = now.signed_duration_since(started_at).to_std().unwrap_or_default();
    // ... context building
}

// Nearly identical in configure/handler.rs and destroy/handler.rs
```

#### Proposed Solution

Create a generic helper in `src/application/command_handlers/common/failure_context.rs`:

```rust
//! Common failure context building utilities

use std::time::Duration;
use chrono::{DateTime, Utc};
use crate::domain::environment::TraceId;
use crate::domain::environment::state::BaseFailureContext;
use crate::shared::Clock;

/// Build base failure context with common fields
///
/// This helper extracts the common logic for building failure context
/// that is shared across all command handlers (provision, configure, destroy).
///
/// # Arguments
///
/// * `error_summary` - Human-readable error description
/// * `error_kind` - Classified error kind for filtering/reporting
/// * `started_at` - When the operation started
/// * `clock` - Clock service for getting current time
///
/// # Returns
///
/// A `BaseFailureContext` with timing, trace ID, and error summary
pub fn build_base_failure_context(
    error_summary: String,
    error_kind: crate::shared::ErrorKind,
    started_at: DateTime<Utc>,
    clock: &dyn Clock,
) -> (BaseFailureContext, DateTime<Utc>, TraceId) {
    let now = clock.now();
    let trace_id = TraceId::new();
    let execution_duration = now
        .signed_duration_since(started_at)
        .to_std()
        .unwrap_or_default();

    let base = BaseFailureContext {
        error_summary,
        failed_at: now,
        execution_started_at: started_at,
        execution_duration,
        trace_id,
        trace_file_path: None, // Will be set by caller if needed
    };

    (base, now, trace_id)
}
```

Then update handlers to use it:

```rust
// In provision/handler.rs
pub(crate) fn build_failure_context(
    &self,
    environment: &Environment<Provisioning>,
    error: &ProvisionCommandHandlerError,
    current_step: ProvisionStep,
    started_at: chrono::DateTime<chrono::Utc>,
) -> ProvisionFailureContext {
    use crate::application::command_handlers::common::failure_context::build_base_failure_context;

    let failed_step = current_step;
    let error_kind = error.error_kind();

    let (mut base, _now, _trace_id) = build_base_failure_context(
        error.to_string(),
        error_kind,
        started_at,
        self.clock.as_ref(),
    );

    let mut context = ProvisionFailureContext {
        failed_step,
        error_kind,
        base,
    };

    // Generate trace file (logging handled by trace writer)
    let traces_dir = environment.traces_dir();
    let writer = ProvisionTraceWriter::new(traces_dir, Arc::clone(&self.clock));

    if let Ok(trace_file) = writer.write_trace(&context, error) {
        context.base.trace_file_path = Some(trace_file);
    }

    context
}
```

#### Rationale

- Eliminates 90% duplication in failure context building
- Makes the pattern more discoverable and consistent
- Reduces chance of bugs when updating common logic
- Still allows handler-specific trace file generation

#### Benefits

- ✅ Reduces ~60 lines of duplicated code across 3 handlers
- ✅ Single source of truth for base context building
- ✅ Easier to maintain and extend
- ✅ No behavior changes (pure refactoring)

#### Implementation Checklist

- [x] Create `src/application/command_handlers/common/mod.rs`
- [x] Create `src/application/command_handlers/common/failure_context.rs`
- [x] Implement `build_base_failure_context` helper
- [x] Add unit tests for the helper
- [x] Update provision handler to use helper
- [x] Update configure handler to use helper
- [x] Update destroy handler to use helper
- [x] Verify all existing tests still pass
- [x] Run linter and fix any issues
- [x] Update documentation if needed

#### Testing Strategy

- Unit test the helper function with various inputs
- Ensure all existing integration tests pass unchanged
- Verify error context fields are identical to before

---

### Proposal #1: Extract State Persistence Helper

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P0

#### Problem

State persistence logic is duplicated at multiple points in each handler:

```rust
// Repeated ~3 times per handler
self.repository.save(&environment.clone().into_any())?;
```

Error handling for persistence failures is also duplicated:

```rust
self.repository.save(&environment.clone().into_any())
    .map_err(DestroyCommandHandlerError::StatePersistence)?;
```

#### Proposed Solution

Create a persistence helper in `src/application/command_handlers/common/persistence.rs`:

```rust
//! State persistence helper utilities

use std::sync::Arc;
use tracing::debug;
use crate::domain::environment::{Environment, repository::EnvironmentRepository};

/// Persist environment state to repository
///
/// Helper that handles the common pattern of:
/// 1. Converting typed environment to AnyEnvironmentState
/// 2. Saving via repository
/// 3. Logging the operation
///
/// # Type Parameters
///
/// * `S` - The environment state type
/// * `E` - The error type that can be converted from RepositoryError
///
/// # Arguments
///
/// * `repository` - The environment repository
/// * `environment` - The environment to persist
///
/// # Errors
///
/// Returns an error if repository save fails
pub fn persist_state<S, E>(
    repository: &Arc<dyn EnvironmentRepository>,
    environment: &Environment<S>,
) -> Result<(), E>
where
    E: From<crate::domain::environment::repository::RepositoryError>,
{
    debug!(
        environment = %environment.name(),
        state = std::any::type_name::<S>(),
        "Persisting environment state"
    );

    repository.save(&environment.clone().into_any())?;
    Ok(())
}
```

Then use it in handlers:

```rust
// Instead of:
self.repository.save(&environment.clone().into_any())?;

// Use:
persist_state(&self.repository, &environment)?;
```

#### Rationale

- Centralizes persistence logic
- Adds consistent logging for state changes
- Reduces boilerplate code
- Makes it easier to add future enhancements (e.g., persistence metrics)

#### Benefits

- ✅ Eliminates repeated persistence pattern
- ✅ Adds consistent debug logging
- ✅ Single place to add instrumentation or metrics
- ✅ Reduces lines of code

#### Implementation Checklist

- [ ] Create `src/application/command_handlers/common/persistence.rs`
- [ ] Implement `persist_state` helper
- [ ] Add unit tests
- [ ] Update all handlers to use helper
- [ ] Verify all tests pass
- [ ] Run linter
- [ ] Update documentation

#### Testing Strategy

- Mock repository to verify save is called
- Test error conversion
- Ensure all existing tests pass

---

### Proposal #2: Extract Step Execution Result Type

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P0

#### Problem

The step tracking methods return inconsistent result types:

```rust
// Provision handler
Result<(Environment<Provisioned>, IpAddr), (ProvisionCommandHandlerError, ProvisionStep)>

// Configure handler
Result<Environment<Configured>, (ConfigureCommandHandlerError, ConfigureStep)>

// Destroy handler
Result<(), (DestroyCommandHandlerError, DestroyStep)>
```

The tuple `(Error, Step)` pattern is repeated but not named, making it less clear what the error handling strategy is.

#### Proposed Solution

Create a type alias in `src/application/command_handlers/common/step_tracking.rs`:

```rust
//! Step execution tracking utilities

/// Result type for step execution that includes failed step information
///
/// When executing a sequence of steps, this result type allows us to return
/// both the error that occurred AND the step that was executing, enabling
/// accurate failure context generation.
///
/// # Type Parameters
///
/// * `T` - The success result type
/// * `E` - The error type
/// * `S` - The step enum type
pub type StepResult<T, E, S> = Result<T, (E, S)>;
```

Then use it in handlers:

```rust
// Before
async fn execute_provisioning_with_tracking(
    &self,
    environment: &Environment<Provisioning>,
) -> Result<(Environment<Provisioned>, IpAddr), (ProvisionCommandHandlerError, ProvisionStep)>

// After
async fn execute_provisioning_with_tracking(
    &self,
    environment: &Environment<Provisioning>,
) -> StepResult<(Environment<Provisioned>, IpAddr), ProvisionCommandHandlerError, ProvisionStep>
```

#### Rationale

- Makes the error handling pattern explicit and self-documenting
- Reduces cognitive load when reading signatures
- Easier to understand the "step tracking" strategy
- No runtime overhead (just a type alias)

#### Benefits

- ✅ Improves code readability
- ✅ Makes pattern more explicit
- ✅ Self-documenting code
- ✅ Zero runtime cost

#### Implementation Checklist

- [ ] Create `src/application/command_handlers/common/step_tracking.rs`
- [ ] Define `StepResult` type alias
- [ ] Update provision handler signatures
- [ ] Update configure handler signatures
- [ ] Update destroy handler signatures
- [ ] Verify all tests pass
- [ ] Run linter
- [ ] Update documentation

#### Testing Strategy

- Verify compilation succeeds
- Ensure all existing tests pass unchanged

---

## Phase 1: Structural Improvements (High Impact, Medium Effort)

These changes improve the overall structure but require more careful implementation.

### Proposal #3: Standardize Error Handling with Help Methods

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P1

#### Problem

Only CreateCommandHandlerError has a `.help()` method. Other handlers lack this actionable error guidance:

```rust
// Create handler has it
impl CreateCommandHandlerError {
    pub fn help(&self) -> &'static str { ... }
}

// Provision, configure, destroy handlers don't
```

This violates the project's actionability principle (see `docs/development-principles.md`).

#### Proposed Solution

Add `.help()` methods to all command handler errors:

```rust
// In provision/errors.rs
impl ProvisionCommandHandlerError {
    pub fn help(&self) -> &'static str {
        match self {
            Self::OpenTofuTemplateRendering(_) => {
                "OpenTofu Template Rendering Failed - Troubleshooting:

1. Check that template source files exist in the fixtures directory
2. Verify template syntax is valid (use template validation tools)
3. Ensure all required template variables are provided
4. Check file permissions on template directories

For template syntax issues, see the Tera documentation."
            }
            Self::OpenTofu(_) => {
                "OpenTofu Command Failed - Troubleshooting:

1. Check OpenTofu is installed: tofu version
2. Verify LXD is running: lxc version
3. Check LXD permissions: lxc list
4. Review OpenTofu error output above for specific issues
5. Try manually running: cd build/<env> && tofu init && tofu plan

For LXD setup issues, see docs/vm-providers.md"
            }
            // ... other variants
        }
    }
}
```

#### Rationale

- Aligns with project actionability principles
- Provides users with clear next steps
- Reduces support burden
- Consistent with create handler

#### Benefits

- ✅ Better user experience
- ✅ Aligns with project principles
- ✅ Self-documenting errors
- ✅ Reduced support requests

#### Implementation Checklist

- [ ] Add `.help()` to `ProvisionCommandHandlerError`
- [ ] Add `.help()` to `ConfigureCommandHandlerError`
- [ ] Add `.help()` to `DestroyCommandHandlerError`
- [ ] Add `.help()` to `TestCommandHandlerError`
- [ ] Write tests for each help method
- [ ] Update CLI to show help on errors
- [ ] Run linters
- [ ] Update error handling documentation

#### Testing Strategy

- Unit test each help method
- Verify help text contains actionable guidance
- Test CLI displays help appropriately

---

### Proposal #4: Remove pub(crate) Test Exposure

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P1  
**Depends On**: Proposal #0

#### Problem

Methods like `build_failure_context` are marked `pub(crate)` solely for testing:

```rust
// In provision/handler.rs
pub(crate) fn build_failure_context(...) { ... }

// Only used in tests:
// tests/provision/tests/integration.rs
let context = command_handler.build_failure_context(&environment, &error, step, started_at);
```

This exposes internal implementation details that shouldn't be public.

#### Proposed Solution

After extracting common helpers (Proposal #0), test those helpers directly instead of testing through the command handler:

```rust
// Test the extracted helper directly
#[test]
fn it_should_build_base_failure_context() {
    use crate::application::command_handlers::common::failure_context::build_base_failure_context;

    let clock = MockClock::new(...);
    let error_summary = "Test error".to_string();
    let error_kind = ErrorKind::CommandExecution;
    let started_at = clock.now();

    let (base, now, trace_id) = build_base_failure_context(
        error_summary,
        error_kind,
        started_at,
        &clock,
    );

    assert_eq!(base.error_summary, "Test error");
    // ... other assertions
}

// Make build_failure_context private
fn build_failure_context(...) { ... }  // No pub(crate)
```

#### Rationale

- Reduces public API surface
- Tests implementation, not internal methods
- Forces better separation of concerns
- More maintainable tests

#### Benefits

- ✅ Cleaner public API
- ✅ Better encapsulation
- ✅ Tests more focused on behavior
- ✅ Easier to refactor internals

#### Implementation Checklist

- [ ] Ensure common helpers are extracted (Proposal #0)
- [ ] Add tests for common helpers
- [ ] Remove pub(crate) from build_failure_context methods
- [ ] Update integration tests to test through public API
- [ ] Verify all tests pass
- [ ] Run linters
- [ ] Update testing documentation

#### Testing Strategy

- Test common helpers directly
- Test command handlers through public execute method
- Ensure coverage doesn't decrease

---

## Phase 2: Consistency & Polish (Medium Impact, Low Effort)

These changes improve consistency and code quality without major restructuring.

### Proposal #5: Consistent Logging Patterns

**Status**: ⏳ Not Started  
**Impact**: 🟢 Low  
**Effort**: 🔵 Low  
**Priority**: P2  
**Depends On**: None

#### Problem

Logging patterns vary across handlers:

```rust
// Some handlers log start and end
info!("Starting...");
// ... work ...
info!("Completed successfully");

// Others only log start or only log end
// Log messages have inconsistent formats
```

#### Proposed Solution

Standardize logging at key points:

```rust
// At start of execute
info!(
    command = "provision",
    environment = %environment.name(),
    "Starting {} command",
    "provision"
);

// At success
info!(
    command = "provision",
    environment = %environment.name(),
    duration = ?execution_duration,
    "Command completed successfully"
);

// At failure (already handled by error state)
```

Create logging guidelines in documentation.

#### Rationale

- Consistent logs are easier to parse and analyze
- Makes debugging more predictable
- Improves observability

#### Benefits

- ✅ Better observability
- ✅ Easier to debug issues
- ✅ Consistent log format for tooling
- ✅ Minimal effort required

#### Implementation Checklist

- [ ] Document logging patterns in `docs/contributing/logging-guide.md`
- [ ] Update provision handler logging
- [ ] Update configure handler logging
- [ ] Update destroy handler logging
- [ ] Update create handler logging
- [ ] Update test handler logging
- [ ] Verify log output manually
- [ ] Run linters

#### Testing Strategy

- Manual verification of log output
- Ensure log format is consistent
- Check structured logging fields

---

### Proposal #6: Standardize Method Ordering

**Status**: ⏳ Not Started  
**Impact**: 🟢 Low  
**Effort**: 🔵 Low  
**Priority**: P2  
**Depends On**: None

#### Problem

Methods within handlers are ordered inconsistently:

- Some have public methods first, then private
- Others mix public and private methods
- Helper methods are in different positions

This violates the project's module organization conventions (see `docs/contributing/module-organization.md`).

#### Proposed Solution

Standardize method ordering according to project conventions:

1. Public API (`new`, `execute`)
2. Private main methods (`execute_*_with_tracking`)
3. Private helper methods (grouped by functionality)
4. Private utility methods (`build_failure_context`, etc.)

```rust
impl ProvisionCommandHandler {
    // 1. Public API
    pub fn new(...) -> Self { ... }
    pub async fn execute(...) -> Result<...> { ... }

    // 2. Private main execution methods
    async fn execute_provisioning_with_tracking(...) -> StepResult<...> { ... }

    // 3. Private helper methods - grouped
    async fn render_opentofu_templates(...) -> Result<...> { ... }
    fn create_instance(...) -> Result<...> { ... }
    fn get_instance_info(...) -> Result<...> { ... }

    // 4. Private utility methods
    fn build_failure_context(...) -> FailureContext { ... }
}
```

#### Rationale

- Follows project conventions
- Improves code navigation
- Makes structure predictable
- Easier for new contributors

#### Benefits

- ✅ Consistent code organization
- ✅ Follows project standards
- ✅ Easier to navigate
- ✅ Better developer experience

#### Implementation Checklist

- [ ] Reorder methods in provision handler
- [ ] Reorder methods in configure handler
- [ ] Reorder methods in destroy handler
- [ ] Reorder methods in create handler
- [ ] Reorder methods in test handler
- [ ] Verify all tests pass
- [ ] Run linters
- [ ] Update module organization docs with examples

#### Testing Strategy

- Ensure all tests still pass (no functional changes)
- Verify code compiles

---

## 📈 Timeline

- **Start Date**: October 28, 2025
- **Estimated Duration**: 2-3 weeks
- **Target Completion**: Mid-November 2025

### Sprint Planning

- **Week 1**: Phase 0 (Proposals #0, #1, #2)
- **Week 2**: Phase 1 (Proposals #3, #4)
- **Week 3**: Phase 2 (Proposals #5, #6) + Buffer

## 🔍 Review Process

### Approval Criteria

- [ ] All proposals reviewed by maintainers
- [ ] Technical feasibility validated
- [ ] Aligns with [Development Principles](../../development-principles.md)
- [ ] Implementation plan is clear and actionable
- [ ] No behavior changes (pure refactoring)

### Completion Criteria

- [ ] All active proposals implemented
- [ ] All tests passing (including E2E)
- [ ] All linters passing
- [ ] Documentation updated
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch
- [ ] Zero behavioral changes verified

## 📚 Related Documentation

- [Development Principles](../../development-principles.md)
- [Contributing Guidelines](../../contributing/README.md)
- [Error Handling Guide](../../contributing/error-handling.md)
- [Testing Conventions](../../contributing/testing.md)
- [Module Organization](../../contributing/module-organization.md)

## 💡 Notes

### Key Constraints

- **No behavior changes**: All refactoring must preserve existing behavior
- **Test coverage**: Must maintain or improve test coverage
- **Backwards compatibility**: Public APIs must remain unchanged
- **Performance**: No performance regressions

### Risks & Mitigation

1. **Risk**: Breaking existing functionality

   - **Mitigation**: Run full test suite (including E2E) after each proposal

2. **Risk**: Introducing new patterns that conflict with existing code

   - **Mitigation**: Review with maintainers before implementation

3. **Risk**: Making code more complex in attempt to reduce duplication
   - **Mitigation**: Follow "Rule of Three" - only extract after 3+ duplications

### Success Metrics

- Lines of code reduced: Target 150-200 lines (reduced from 200-300 due to discarding trait-based template)
- Duplication eliminated: Target 80%+ of identified duplication
- Test coverage: Maintain 100% of current coverage
- Build time: No degradation
- All linters pass with no new warnings

---

**Created**: October 28, 2025  
**Last Updated**: October 28, 2025  
**Status**: 📋 Planning
