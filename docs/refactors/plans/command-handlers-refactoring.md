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
**Completed**: 7
**In Progress**: 0
**Not Started**: 0

### Phase Summary

- **Phase 0 - Quick Wins (High Impact, Low Effort)**: ✅ 4/4 completed (100%)
- **Phase 1 - Structural Improvements (High Impact, Medium Effort)**: ✅ 2/2 completed (100%)
- **Phase 2 - Consistency & Polish (Medium Impact, Low Effort)**: ✅ 2/2 completed (100%)

## 🎉 Refactoring Complete!

All active proposals have been successfully implemented. The command handlers codebase is now:

- Free of significant code duplication
- Following consistent patterns and conventions
- Well-tested and maintainable
- Properly documented

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

**Status**: ✅ Completed  
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

~~Create a persistence helper in `src/application/command_handlers/common/persistence.rs`:~~ (Original approach)

**Original approach attempted**: Simple helper function with generic error type:

```rust
pub fn persist_state<S, E>(
    repository: &Arc<dyn EnvironmentRepository>,
    environment: &Environment<S>,
) -> Result<(), E>
where
    E: From<crate::domain::environment::repository::RepositoryError>,
{
    // ...
}
```

**Problem with original approach**: Required verbose turbofish syntax at call sites:

```rust
persist_state::<_, HandlerError>(&self.repository, &environment)?;
```

**Final Solution Implemented**: `TypedEnvironmentRepository` wrapper in `src/domain/environment/repository.rs`

Created a wrapper repository that provides type-safe, state-specific save methods:

```rust
pub struct TypedEnvironmentRepository {
    repository: std::sync::Arc<dyn EnvironmentRepository>,
}

impl TypedEnvironmentRepository {
    pub fn new(repository: std::sync::Arc<dyn EnvironmentRepository>) -> Self {
        Self { repository }
    }

    pub fn inner(&self) -> &std::sync::Arc<dyn EnvironmentRepository> {
        &self.repository
    }
}

// Macro-generated methods for each state type:
// - save_provisioning(&Environment<Provisioning>)
// - save_provisioned(&Environment<Provisioned>)
// - save_configured(&Environment<Configured>)
// etc. for all 15 state types
```

Usage in handlers:

```rust
// Before (verbose, requires manual conversion):
self.repository.save(&environment.clone().into_any())?;

// After (clean, type-safe):
self.repository.save_provisioning(&environment)?;
self.repository.save_provisioned(&provisioned)?;
self.repository.save_provision_failed(&failed)?;
```

#### Rationale

**Why the wrapper approach is better than the helper function:**

1. **No turbofish syntax**: Clean API without verbose type annotations
2. **Type safety**: Compiler ensures correct state method is called
3. **Better ergonomics**: Natural method call syntax vs helper function
4. **DDD alignment**: Repository wrapper follows adapter pattern
5. **Encapsulation**: Conversion logic hidden inside the wrapper
6. **Extensibility**: Easy to add typed `load` methods in the future
7. **Logging built-in**: All persistence operations automatically logged

**Architectural decision**: Moved from application layer helper to domain layer wrapper because:

- Persistence concerns belong in the repository layer
- Type conversion is a repository responsibility
- Logging state changes is part of repository's job
- Follows single responsibility principle

#### Benefits

**Achieved**:

- ✅ Eliminated 9 instances of `.clone().into_any()` conversion across 3 handlers
- ✅ Added consistent debug logging for all state persistence operations
- ✅ Clean, type-safe API without turbofish syntax
- ✅ Compiler-enforced state correctness (can't accidentally save wrong state type)
- ✅ Single place to add instrumentation or metrics in the future
- ✅ Better separation of concerns (repository handles conversion)
- ✅ More maintainable than original helper approach

**Metrics**:

- Lines of duplicated code eliminated: 27 (9 × 3 lines each: clone, into_any, save)
- Method calls simplified: 9 persistence operations across provision/configure/destroy handlers
- State types supported: 15 (all possible environment states)
- Test coverage: 991 unit tests + E2E tests all passing

#### Implementation Checklist

- [x] Create TypedEnvironmentRepository wrapper in repository.rs
- [x] Implement state-specific save methods using macro
- [x] Add logging to save methods
- [x] Update provision handler to use typed repository
- [x] Update configure handler to use typed repository
- [x] Update destroy handler to use typed repository
- [x] Verify all tests pass (991 unit tests)
- [x] Run linter and fix documentation issues
- [x] Update documentation if needed

#### Testing Strategy

- ✅ Verify typed repository wrapper works with all state types
- ✅ Ensure all existing integration tests pass unchanged (991 tests passing)
- ✅ Verify logging is present in repository operations
- ✅ Run E2E tests to ensure end-to-end functionality (all passed)

#### Implementation Summary

**What Changed from Original Plan**:

The original proposal suggested a simple helper function in `common/persistence.rs`:

```rust
pub fn persist_state<S, E>(repository: &Arc<dyn EnvironmentRepository>, environment: &Environment<S>) -> Result<(), E>
```

**Why We Changed It**:

During implementation, we discovered this approach had significant ergonomic issues:

1. Required verbose turbofish syntax: `persist_state::<_, HandlerError>(&repo, &env)?`
2. Leaked type conversion concerns to the application layer
3. Didn't align well with DDD principles (conversion should be repository's job)

**Final Implementation**:

Created `TypedEnvironmentRepository` wrapper in the domain layer (`src/domain/environment/repository.rs`):

- **Architecture**: Wrapper around `EnvironmentRepository` following adapter pattern
- **Type Safety**: State-specific methods (e.g., `save_provisioning()`, `save_configured()`)
- **Conversion**: Handles `Environment<S>` → `AnyEnvironmentState` internally
- **Generation**: Uses macro to create 15 state-specific save methods
- **Logging**: Built-in debug logging for all persistence operations
- **API**: Clean syntax: `self.repository.save_provisioning(&environment)?`

**Key Technical Details**:

- Macro `impl_save_for_state!` generates save methods for each state
- Each method clones environment, calls `.into_any()`, and saves via base repository
- Wrapper provides `.inner()` accessor for operations like load/delete/list
- Handler constructors wrap base repository: `TypedEnvironmentRepository::new(repository)`

**Design Decisions**:

1. **Domain vs Application Layer**: Moved to domain layer because persistence logic belongs with repository
2. **Macro vs Manual**: Used macro to avoid copy-paste for 15 state types
3. **Wrapper vs Trait**: Wrapper pattern simpler than trait implementation
4. **Logging Location**: Logging in repository methods (not application layer) for better observability

---

### Proposal #2: Extract Step Execution Result Type

**Status**: ✅ Completed  
**Commit**: c04ef8b  
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

- [x] Create type alias in `src/application/command_handlers/common/mod.rs`
- [x] Define `StepResult<T, E, S>` type alias with documentation
- [x] Update provision handler signatures (1 method)
- [x] Update configure handler signatures (1 method)
- [x] Update destroy handler signatures (1 method)
- [x] Verify all tests pass (991 unit tests + 48 integration tests)
- [x] Run linter (all linters pass)
- [x] Update documentation (comprehensive doc comments added)

#### Testing Strategy

- Verify compilation succeeds
- Ensure all existing tests pass unchanged

---

## Phase 1: Structural Improvements (High Impact, Medium Effort)

These changes improve the overall structure but require more careful implementation.

### Proposal #3: Standardize Error Handling with Help Methods

**Status**: ✅ Completed  
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

- [x] Add `.help()` to `ProvisionCommandHandlerError`
- [x] Add `.help()` to `ConfigureCommandHandlerError`
- [x] Add `.help()` to `DestroyCommandHandlerError`
- [x] Write tests for each help method
- [x] Run linters
- [x] Fixed compilation errors with correct error variants
- [x] Fixed doctests to use correct error variant constructors
- [ ] Add `.help()` to `TestCommandHandlerError` (if needed in future)
- [ ] Update CLI to show help on errors (future enhancement)
- [ ] Update error handling documentation (optional)

#### Testing Strategy

- Unit test each help method ✅
- Verify help text contains actionable guidance ✅
- Test CLI displays help appropriately (future work)

---

### Proposal #4: Remove pub(crate) Test Exposure

**Status**: ✅ Completed  
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

- [x] Ensure common helpers are extracted (Proposal #0)
- [x] Add tests for common helpers
- [x] Remove pub(crate) from build_failure_context methods
- [x] Update integration tests to test through public API (removed tests that tested internal methods)
- [x] Verify all tests pass
- [x] Run linters
- [x] Update testing documentation
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

**Status**: ✅ Completed  
**Impact**: 🟢 Low  
**Effort**: 🔵 Low  
**Priority**: P2  
**Depends On**: None

#### Problem

Logging patterns varied across handlers:

- Create handler used `environment_name` field while others used `environment`
- Create handler had verbose step-by-step logging while others only logged start/end
- Log messages lacked consistent structured fields
- No documentation for command handler logging patterns

#### Implemented Solution

1. **Standardized Field Naming**: All handlers now use `environment` field (not `environment_name`)
2. **Minimal Logging Pattern**: All handlers log only at start and completion (not individual steps)
3. **Consistent Structured Fields**: All logs include `command` and `environment` fields
4. **Documentation Added**: New "Command Handler Logging Patterns" section in `docs/contributing/logging-guide.md`

#### Changes Made

- Updated `create` handler to use `environment` field instead of `environment_name`
- Removed verbose step-by-step logging from `create` handler
- Added comprehensive logging patterns section to logging-guide.md with examples for all handlers
- Documented anti-patterns to avoid

#### Rationale

- Consistent logs are easier to parse and analyze
- Makes debugging more predictable
- Improves observability
- Reduces log noise while maintaining visibility

#### Benefits

- ✅ Better observability through consistent patterns
- ✅ Easier to debug issues with predictable log structure
- ✅ Consistent log format for tooling and aggregation
- ✅ Clear documentation for future handlers

#### Implementation Checklist

- [x] Document logging patterns in `docs/contributing/logging-guide.md`
- [x] Update create handler logging (standardize field naming, remove verbose logging)
- [x] Verify provision handler follows standard pattern
- [x] Verify configure handler follows standard pattern
- [x] Verify destroy handler follows standard pattern
- [x] Run full test suite
- [x] Run all linters

#### Testing Strategy

- ✅ All 1002 unit tests passing
- ✅ All 203 integration tests passing (including AI enforcement)
- ✅ All linters passing (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)
- ✅ Manual verification: create handler now uses consistent field naming

---

### Proposal #6: Standardize Method Ordering

**Status**: ✅ Completed  
**Impact**: 🟢 Low  
**Effort**: 🔵 Low  
**Priority**: P2  
**Depends On**: None

#### Problem

Methods within the destroy handler were ordered inconsistently:

- `pub(crate)` methods (`should_destroy_infrastructure`, `cleanup_state_files`) were placed after private methods
- This violated the project's module organization conventions (see `docs/contributing/module-organization.md`)
- Other handlers (provision, configure, create) already followed correct ordering

#### Implemented Solution

Reordered methods in destroy handler to follow standard pattern:

1. Public API (`new`, `execute`)
2. `pub(crate)` helper methods (for testing business logic)
3. Private main methods (`execute_*_with_tracking`)
4. Private helper methods (`build_failure_context`, `destroy_infrastructure`)

```rust
impl DestroyCommandHandler {
    // 1. Public API
    pub fn new(...) -> Self { ... }
    pub fn execute(...) -> Result<...> { ... }

    // 2. pub(crate) helper methods for testing business logic
    pub(crate) fn should_destroy_infrastructure(...) -> bool { ... }
    pub(crate) fn cleanup_state_files<S>(...) -> Result<...> { ... }

    // 3. Private main execution methods
    fn execute_destruction_with_tracking(...) -> StepResult<...> { ... }

    // 4. Private helper methods
    fn build_failure_context(...) -> FailureContext { ... }
    fn destroy_infrastructure(...) -> Result<...> { ... }
}
```

#### Rationale

- Follows project conventions from `docs/contributing/module-organization.md`
- Improves code navigation by grouping similar visibility levels
- Makes structure predictable across all handlers
- Easier for new contributors to understand code organization

#### Benefits

- ✅ Consistent code organization across all handlers
- ✅ Follows established project standards
- ✅ Easier to navigate and understand
- ✅ Better developer experience

#### Implementation Checklist

- [x] Provision handler - Already well-organized
- [x] Configure handler - Already well-organized
- [x] Reorder methods in destroy handler
- [x] Create handler - Already well-organized
- [x] Verify code compiles (cargo check)
- [x] Update refactoring plan

#### Testing Strategy

- ✅ Code compiles successfully (cargo check passed)
- ✅ No functional changes - pure reorganization
- ✅ All method signatures and implementations unchanged

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
