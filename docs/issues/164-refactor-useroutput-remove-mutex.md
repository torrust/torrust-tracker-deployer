# Refactor UserOutput to Remove Arc<Mutex<>> Pattern

**Issue**: #164
**Parent Epic**: N/A (Architectural refactoring task)
**Related**:

- [ADR: Remove UserOutput Mutex](../decisions/user-output-mutex-removal.md)
- ProgressReporter Deadlock Investigation (2025-11-11)

## Overview

Remove the `Arc<Mutex<UserOutput>>` pattern throughout the codebase and replace it with direct ownership/borrowing to eliminate deadlock risks and simplify the architecture. This addresses a critical reentrancy deadlock discovered in `CreateTemplateCommandController` and implements the architectural decision documented in the ADR.

## Goals

- [ ] Eliminate all deadlock risks in UserOutput access
- [ ] Simplify codebase by removing complex mutex management
- [ ] Improve performance by eliminating lock overhead
- [ ] Establish clear ownership patterns matching single-threaded usage
- [ ] Remove timeout mechanisms and deadlock prevention code
- [ ] Update all command handlers to use direct UserOutput access

## 🏗️ Architecture Requirements

**DDD Layer**: Presentation (primary), Application (secondary)
**Module Path**: `src/presentation/user_output/`, command controllers, progress reporters
**Pattern**: Direct ownership model with mutable borrowing

### Module Structure Requirements

- [ ] Follow DDD layer separation - presentation layer owns UserOutput
- [ ] Respect dependency flow - commands receive `&mut UserOutput` from presentation
- [ ] Use direct ownership patterns instead of synchronization primitives
- [ ] Maintain clear ownership boundaries through type system

### Architectural Constraints

- [ ] No mutexes for UserOutput access (zero `Arc<Mutex<UserOutput>>` usage)
- [ ] Commands and progress reporters use `&mut UserOutput` parameters
- [ ] Single ownership with borrowing for shared access patterns
- [ ] Error types simplified (remove mutex-related error variants)

### Anti-Patterns to Avoid

- ❌ Any `Arc<Mutex<UserOutput>>` usage
- ❌ Runtime synchronization mechanisms for sequential operations
- ❌ Complex lock management or timeout mechanisms
- ❌ Mutex-related error handling (timeouts, poisoning, etc.)

## Specifications

### Current Problem Analysis

#### Deadlock Pattern

The current architecture suffers from reentrancy deadlocks:

```rust
// PROBLEMATIC: Current deadlock pattern
impl CreateTemplateCommandController {
    fn display_success_and_guidance(&self) -> Result<(), Error> {
        // Acquires UserOutput mutex here
        let mut output = self.output.lock().unwrap();
        output.display_success("Template created successfully!");

        // Still holding mutex, calls progress.complete()
        self.progress.complete()?; // ← Tries to acquire same mutex = DEADLOCK
        Ok(())
    }
}
```

#### Architectural Mismatch

- **Complex concurrency**: `Arc<Mutex<>>` for single-threaded, sequential operations
- **Timeout workarounds**: Defensive programming that shouldn't be necessary
- **Error complexity**: Mutex poisoning, timeout errors for simple output operations

### Target Architecture

#### Direct Ownership Model

```rust
// TARGET: Direct ownership with clear boundaries
pub struct ExecutionContext {
    output: UserOutput,  // Direct ownership, no mutex
}

impl ExecutionContext {
    pub fn run_command<C>(&mut self, command: C) -> Result<(), Error>
    where
        C: Command,
    {
        command.execute(&mut self.output)  // Mutable borrow
    }
}

// Commands use direct access
pub trait Command {
    fn execute(&self, output: &mut UserOutput) -> Result<(), Error>;
}
```

#### ProgressReporter Simplification

```rust
// TARGET: Simple progress reporting
pub struct ProgressReporter {
    current_step: String,
    total_steps: usize,
}

impl ProgressReporter {
    pub fn complete(&self, output: &mut UserOutput) -> Result<(), Error> {
        output.display_info("Operation completed.");  // Direct access, no deadlock risk
        Ok(())
    }

    pub fn update(&self, output: &mut UserOutput, message: &str) -> Result<(), Error> {
        output.display_progress(message);  // Direct access
        Ok(())
    }
}
```

### Implementation Phases

#### Phase 1: Core UserOutput Refactor

**Files to Modify:**

- `src/presentation/user_output/mod.rs` - Remove Arc<Mutex<>> wrapper
- `src/presentation/execution_context.rs` - Change to direct ownership
- `src/presentation/progress.rs` - Remove timeout mechanisms

**Changes:**

```rust
// BEFORE: Complex mutex pattern
pub struct ExecutionContext {
    output: Arc<Mutex<UserOutput>>,
}

// AFTER: Direct ownership
pub struct ExecutionContext {
    output: UserOutput,
}
```

#### Phase 2: Command Signature Updates

**Files to Modify:**

- All command handlers in `src/presentation/controllers/`
- Command trait definitions in `src/application/commands/`

**Changes:**

```rust
// BEFORE: Mutex parameter
fn execute(&self, output: &Arc<Mutex<UserOutput>>) -> Result<(), Error>

// AFTER: Direct mutable access
fn execute(&self, output: &mut UserOutput) -> Result<(), Error>
```

#### Phase 3: Progress Reporter Refactor

**Files to Modify:**

- `src/presentation/progress.rs` - Remove timeout logic
- All progress reporter usage sites

**Changes:**

- Remove `acquire_output_with_timeout()` method
- Remove `UserOutputMutexTimeout` error variant
- Simplify all progress update methods

#### Phase 4: Error Handling Cleanup

**Files to Modify:**

- Error type definitions throughout presentation layer
- Error handling in command controllers

**Changes:**

- Remove mutex-related error variants
- Simplify error handling paths
- Remove timeout and poisoning error cases

### Testing Strategy

#### Unit Test Updates

**Remove mutex mocking:**

```rust
// BEFORE: Complex mutex mocking
let output = Arc::new(Mutex::new(MockUserOutput::new()));

// AFTER: Direct mock usage
let mut output = MockUserOutput::new();
```

#### Integration Test Validation

- Verify zero deadlock scenarios in new architecture
- Validate performance improvements (no lock contention)
- Ensure all existing functionality preserved

#### E2E Test Verification

- Run full E2E test suite to validate user experience unchanged
- Verify progress reporting still works correctly
- Confirm error messages remain clear and actionable

### Migration Checklist

#### Code Review Points

- [ ] Zero `Arc<Mutex<UserOutput>>` usage anywhere in codebase
- [ ] All command signatures updated to use `&mut UserOutput`
- [ ] No mutex-related timeout or deadlock prevention code
- [ ] ProgressReporter uses direct access patterns
- [ ] Error types simplified (no mutex-related variants)

#### Performance Validation

- [ ] No lock contention overhead
- [ ] Simplified call stacks (no lock acquisition)
- [ ] Memory usage reduction (no Arc overhead)

#### Functional Validation

- [ ] All existing user-facing functionality preserved
- [ ] Progress reporting works correctly
- [ ] Error messages remain clear and actionable
- [ ] Command execution order unchanged

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

**Quality Checks:**

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Architecture Validation:**

- [ ] Zero `Arc<Mutex<UserOutput>>` usage in entire codebase (verified by grep search)
- [ ] All command handlers use `&mut UserOutput` or owned `UserOutput` parameters
- [ ] No mutex-related timeout, deadlock prevention, or error handling code
- [ ] `ProgressReporter` uses direct access patterns with no blocking operations

**Functional Validation:**

- [ ] All existing functionality preserved (user experience identical)
- [ ] All unit tests pass with no mutex mocking requirements
- [ ] Integration tests demonstrate no deadlock scenarios possible
- [ ] E2E tests pass with improved performance (no lock contention)

**Documentation:**

- [ ] ADR implementation section updated with final results
- [ ] Code comments explain new ownership patterns where complex
- [ ] Breaking changes documented (if any internal API changes)
- [ ] Performance improvements quantified (if measurable)

**Regression Prevention:**

- [ ] Linting rules or compile-time checks prevent reintroduction of `Arc<Mutex<UserOutput>>`
- [ ] Test coverage ensures deadlock scenarios cannot re-emerge
- [ ] Architecture documentation updated to reflect new ownership patterns

## Risk Assessment

### Low Risk

- **Functional Changes**: User experience remains identical
- **Test Coverage**: Existing tests validate behavior preservation
- **Rollback**: Changes are architectural, easy to revert if needed

### Medium Risk

- **Breaking Changes**: Internal API changes may affect future development
- **Ownership Complexity**: Rust borrow checker may require careful lifetime management

### Mitigation Strategies

- **Incremental Implementation**: Phase-by-phase approach reduces risk
- **Comprehensive Testing**: Unit, integration, and E2E tests validate each phase
- **Documentation**: Clear ownership patterns documented for future contributors
