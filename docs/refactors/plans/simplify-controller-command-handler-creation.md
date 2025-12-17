# Simplify Controller Command Handler Creation

## 📋 Overview

Remove unnecessary progress step for application command handler creation across all presentation layer controllers. Handler creation is instantaneous (just object construction with no I/O), so users don't need visibility into this internal implementation detail.

**Target Files:**

- `src/presentation/controllers/provision/handler.rs`
- `src/presentation/controllers/configure/handler.rs`
- `src/presentation/controllers/register/handler.rs`
- `src/presentation/controllers/release/handler.rs`
- `src/presentation/controllers/run/handler.rs`
- Other controllers following similar pattern

**Scope:**

- Remove `CreateCommandHandler` step from `ProvisionStep` enum (and similar in other controllers)
- Move handler creation inside the method that uses it (e.g., `provision_infrastructure`)
- Simplify execute methods to focus on user-visible operations
- Maintain consistent pattern across all controllers

## 📊 Progress Tracking

**Total Active Proposals**: 1
**Total Postponed**: 0
**Total Discarded**: 0
**Completed**: 0
**In Progress**: 0
**Not Started**: 1

### Phase Summary

- **Phase 0 - Simplify All Controllers (Medium Impact, Low Effort)**: ⏳ 0/1 completed (0%)

### Discarded Proposals

None

### Postponed Proposals

None

## 🎯 Key Problems Identified

### 1. Unnecessary Progress Visibility

Controllers currently report handler creation as a separate step, but:

- Handler creation is instantaneous (no I/O, just object construction)
- Users don't need visibility into this implementation detail
- It clutters the progress output with non-meaningful steps
- Creates coupling between execute method and internal implementation

### 2. Inconsistent Encapsulation

The handler is created in the `execute` method but only used within a single sub-method:

```rust
pub async fn execute(&mut self, environment_name: &str) -> Result<...> {
    let env_name = self.validate_environment_name(environment_name)?;

    let handler = self.create_command_handler()?;  // Created here

    let provisioned = self.provision_infrastructure(&handler, &env_name).await?;  // Only used here

    // ...
}
```

Better encapsulation would move handler creation inside `provision_infrastructure`.

## 🚀 Refactoring Phases

---

## Phase 0: Simplify All Controllers (Highest Priority)

Remove unnecessary progress steps and improve encapsulation by moving handler creation closer to where it's used.

### Proposal #1: Consolidate Handler Creation Across All Controllers

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P0  
**Depends On**: None

#### Problem

All controllers follow the same pattern of creating application command handlers as a separate step with progress reporting:

**Before (current pattern in provision controller):**

```rust
/// Steps in the provision workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProvisionStep {
    ValidateEnvironment,
    CreateCommandHandler,      // ← Unnecessary step
    ProvisionInfrastructure,
}

pub async fn execute(&mut self, environment_name: &str) -> Result<...> {
    let env_name = self.validate_environment_name(environment_name)?;

    let handler = self.create_command_handler()?;  // ← Separate method

    let provisioned = self.provision_infrastructure(&handler, &env_name).await?;

    self.complete_workflow(environment_name)?;
    self.display_connection_details(&provisioned)?;

    Ok(provisioned)
}

fn create_command_handler(&mut self) -> Result<...> {
    self.progress.start_step(ProvisionStep::CreateCommandHandler.description())?;
    let handler = ProvisionCommandHandler::new(self.clock.clone(), self.repository.clone());
    self.progress.complete_step(None)?;
    Ok(handler)
}

async fn provision_infrastructure(
    &mut self,
    handler: &ProvisionCommandHandler,  // ← Handler passed as parameter
    env_name: &EnvironmentName,
) -> Result<...> {
    self.progress.start_step(ProvisionStep::ProvisionInfrastructure.description())?;
    let provisioned = handler.execute(env_name).await?;
    self.progress.complete_step(Some("Infrastructure provisioned"))?;
    Ok(provisioned)
}
```

**Issues:**

- Handler creation is not a meaningful user-visible step
- Handler is created in one place but only used in one other place
- Progress output is cluttered with internal details
- Similar pattern repeated across all controllers

#### Proposed Solution

**After (simplified pattern):**

```rust
/// Steps in the provision workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProvisionStep {
    ValidateEnvironment,
    ProvisionInfrastructure,  // ← Only user-visible steps
}

pub async fn execute(&mut self, environment_name: &str) -> Result<...> {
    let env_name = self.validate_environment_name(environment_name)?;

    // Handler creation moved inside provision_infrastructure
    let provisioned = self.provision_infrastructure(&env_name).await?;

    self.complete_workflow(environment_name)?;
    self.display_connection_details(&provisioned)?;

    Ok(provisioned)
}

// create_command_handler method removed

async fn provision_infrastructure(
    &mut self,
    env_name: &EnvironmentName,  // ← Simpler signature
) -> Result<...> {
    self.progress.start_step(ProvisionStep::ProvisionInfrastructure.description())?;

    // Handler creation encapsulated here
    let handler = ProvisionCommandHandler::new(self.clock.clone(), self.repository.clone());

    let provisioned = handler.execute(env_name).await?;
    self.progress.complete_step(Some("Infrastructure provisioned"))?;
    Ok(provisioned)
}
```

#### Rationale

1. **Better Encapsulation**: Handler creation is moved closer to where it's used
2. **Cleaner Progress**: Only meaningful steps visible to users
3. **Simpler Code**: Fewer methods, simpler signatures, clearer intent
4. **Consistent Pattern**: Apply same simplification across all controllers

#### Benefits

- ✅ Reduces clutter in progress output
- ✅ Improves code organization and encapsulation
- ✅ Simplifies controller execute methods
- ✅ Makes the code more maintainable
- ✅ Reduces line count across all controllers

#### Implementation Checklist

Apply to all controllers in this order (to catch any issues early):

- [ ] Provision controller (`src/presentation/controllers/provision/handler.rs`)
- [ ] Configure controller (`src/presentation/controllers/configure/handler.rs`)
- [ ] Register controller (`src/presentation/controllers/register/handler.rs`)
- [ ] Release controller (`src/presentation/controllers/release/handler.rs`)
- [ ] Run controller (`src/presentation/controllers/run/handler.rs`)
- [ ] Destroy controller (if it follows the same pattern)
- [ ] Test controller (if it follows the same pattern)

For each controller:

1. Remove `CreateCommandHandler` variant from the step enum
2. Update `count()` method to reflect new step count
3. Remove `create_command_handler` method
4. Move handler creation inside the method that uses it
5. Update progress reporter step count in constructor
6. Run tests to verify behavior unchanged
7. Verify progress output still clear and informative

- [ ] Verify all tests pass after each controller update
- [ ] Run linter and fix any issues
- [ ] Update any affected documentation

#### Testing Strategy

For each controller:

1. Run existing unit tests to verify behavior unchanged
2. Manually test progress output to ensure it's clear
3. Verify step numbering is correct (e.g., "Step 1/2" instead of "Step 1/3")

No new tests needed - existing tests validate the behavior remains correct.

#### Expected Results

Per controller:

- **Lines Removed**: ~15-20 (entire method + enum variant)
- **Lines Added**: ~1 (handler creation in existing method)
- **Net Change**: -14 to -19 lines per controller
- **Total Net Change**: ~-70 to -95 lines across 5 controllers

---

## 📈 Timeline

- **Start Date**: TBD
- **Target Completion**: 1-2 hours (simple refactoring)
- **Approach**: Incremental - one controller at a time with test verification

## 🎓 Principles Alignment

This refactoring aligns with:

- **Observability**: Users see only meaningful progress steps
- **Maintainability**: Simpler code with better encapsulation
- **Clean Code**: Each method has clear, focused responsibility

## 📚 Related Documentation

- [Development Principles](../../development-principles.md) - Observability and maintainability
- [DDD Layer Placement](../../contributing/ddd-layer-placement.md) - Controller responsibilities

## 💡 Notes

- This is a pure refactoring - no behavior changes
- Pattern discovered during implementation of issue #242 (connection details display)
- Consider documenting this pattern in controller guidelines after implementation

---

**Created**: December 17, 2025  
**Status**: 📋 Planning  
**Related Issues**: None yet - discovered during #242 implementation
