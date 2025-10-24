# Add Missing Transition States for Destroy Operation

**Issue**: #29
**Parent Epic**: N/A - [Standalone Enhancement]
**Related**: [Environment State Documentation](../src/domain/environment/state/mod.rs), [Command State Return Pattern ADR](../docs/decisions/command-state-return-pattern.md)

## Overview

Add the missing `Destroying` transition state and `DestroyFailed` error state to the environment state machine. Currently, the destroy operation jumps directly from any state to `Destroyed` without proper state transitions, which doesn't align with the pattern used by other operations (provisioning, configuring, releasing) and doesn't represent partial failure scenarios.

This gap prevents proper error handling when infrastructure destruction fails partially, leaving the system in an ambiguous state where we can't determine if the environment is in its previous state or partially destroyed.

## Goals

- [ ] Add `Destroying` transition state to represent active destruction workflow
- [ ] Add `DestroyFailed` error state to represent failed destruction attempts
- [ ] Update the destroy command handler to properly transition through states
- [ ] Maintain consistency with other operation patterns (provision, configure, release)
- [ ] Ensure proper error context is captured for failed destroy operations

## 🏗️ Architecture Requirements

**DDD Layer**: Domain  
**Module Path**: `src/domain/environment/state/`  
**Pattern**: Value Object (State Marker Types)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] Respect dependency flow rules (dependencies flow toward domain)
- [ ] Use appropriate module organization (see [docs/contributing/module-organization.md](../docs/contributing/module-organization.md))

### Architectural Constraints

- [ ] No business logic in presentation layer
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Testing strategy aligns with layer responsibilities
- [ ] State transitions must be type-safe and compile-time enforced
- [ ] Follow existing state pattern established by `Provisioning`/`ProvisionFailed`

### Anti-Patterns to Avoid

- ❌ Mixing concerns across layers
- ❌ Domain layer depending on infrastructure
- ❌ Monolithic modules with multiple responsibilities
- ❌ Breaking type-state pattern consistency

## Specifications

### Current State Transition Problem

Currently, the destroy operation transitions directly from any state to `Destroyed`:

```rust
// In src/application/command_handlers/destroy.rs
let destroyed = environment.destroy()?; // Direct transition - no intermediate state
```

This doesn't follow the pattern used by other operations:

```rust
// Provisioning pattern (correct)
let environment = environment.start_provisioning(); // -> Provisioning state
// ... execute steps ...
let provisioned = environment.provisioned(); // -> Provisioned state
// OR on error:
let failed = environment.provision_failed(context); // -> ProvisionFailed state
```

### Required State Files

#### 1. Destroying State (`src/domain/environment/state/destroying.rs`)

```rust
//! Destroying State
//!
//! Intermediate state - Infrastructure destruction in progress
//!
//! The environment is actively being destroyed (VM deletion, resource cleanup, etc.).
//! This state indicates that the destroy command has started but not yet completed.
//!
//! **Valid Transitions:**
//! - Success: `Destroyed`
//! - Failure: `DestroyFailed`

use serde::{Deserialize, Serialize};

use crate::domain::environment::state::{
    AnyEnvironmentState, DestroyFailed, Destroyed, StateTypeError,
};
use crate::domain::environment::Environment;

/// Intermediate state - Infrastructure destruction in progress
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Destroying;

impl Environment<Destroying> {
    /// Transitions from Destroying to Destroyed state
    #[must_use]
    pub fn destroyed(self) -> Environment<Destroyed> {
        self.with_state(Destroyed)
    }

    /// Transitions from Destroying to DestroyFailed state
    #[must_use]
    pub fn destroy_failed(
        self,
        context: crate::domain::environment::state::DestroyFailureContext,
    ) -> Environment<DestroyFailed> {
        self.with_state(DestroyFailed { context })
    }
}

// Type Erasure and Restoration implementations
// ... (similar to other states)
```

#### 2. DestroyFailed State (`src/domain/environment/state/destroy_failed.rs`)

```rust
//! `DestroyFailed` State
//!
//! Error state - Infrastructure destruction failed
//!
//! The destroy command failed during execution. The `context` field
//! contains structured error information including the failed step, error kind,
//! timing information, and a reference to the detailed trace file.
//!
//! **Recovery Options:**
//! - Retry the destroy operation
//! - Manual cleanup of remaining resources
//! - Review trace file for detailed error information

use serde::{Deserialize, Serialize};

/// Error context for destroy command failures
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DestroyFailureContext {
    /// Which step failed during destruction
    pub failed_step: DestroyStep,

    /// Error category for type-safe handling
    pub error_kind: ErrorKind,

    /// Base failure context with common fields
    #[serde(flatten)]
    pub base: BaseFailureContext,
}

/// Steps in the destroy workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DestroyStep {
    /// Loading environment state
    LoadEnvironment,
    /// Destroying infrastructure via OpenTofu
    DestroyInfrastructure,
    /// Cleaning up state files
    CleanupStateFiles,
}

/// Error state - Infrastructure destruction failed
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DestroyFailed {
    /// Error context with failure details
    pub context: DestroyFailureContext,
}

// Type Erasure and Restoration implementations
// ... (similar to ProvisionFailed)
```

### State Machine Integration

#### 1. Update Environment Universal Methods

Add transition methods to handle destroy operations from any state:

```rust
// In src/domain/environment/mod.rs or appropriate location
impl<S> Environment<S> {
    /// Transitions from any state to Destroying state
    #[must_use]
    pub fn start_destroying(self) -> Environment<Destroying> {
        self.with_state(Destroying)
    }
}
```

#### 2. Update AnyEnvironmentState Enum

```rust
// In src/domain/environment/state/mod.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnyEnvironmentState {
    // ... existing states ...

    /// Environment in `Destroying` state
    Destroying(Environment<Destroying>),

    /// Environment in `DestroyFailed` error state
    DestroyFailed(Environment<DestroyFailed>),

    // ... existing states ...
}
```

### Command Handler Updates

Update the destroy command handler to use proper state transitions:

```rust
// In src/application/command_handlers/destroy.rs
pub async fn execute(
    &self,
    env_name: &EnvironmentName,
) -> Result<Environment<Destroyed>, DestroyCommandHandlerError> {
    // 1. Load environment
    let environment = self.repository.load(env_name)?;

    // 2. Transition to Destroying state
    let destroying_env = environment.start_destroying();

    // 3. Persist intermediate state
    self.repository.save(&destroying_env.clone().into_any())?;

    // 4. Execute destruction with error tracking
    match self.execute_destruction_with_tracking(&destroying_env).await {
        Ok(()) => {
            // Success: transition to Destroyed
            let destroyed = destroying_env.destroyed();
            self.repository.save(&destroyed.clone().into_any())?;
            Ok(destroyed)
        }
        Err((error, failed_step)) => {
            // Failure: transition to DestroyFailed
            let context = self.build_failure_context(&destroying_env, &error, failed_step);
            let failed = destroying_env.destroy_failed(context);
            self.repository.save(&failed.clone().into_any())?;
            Err(error)
        }
    }
}
```

## Implementation Plan

### Phase 1: Create State Files (2 hours)

- [ ] Create `destroying.rs` state file with transition methods
- [ ] Create `destroy_failed.rs` state file with error context
- [ ] Add new states to `mod.rs` exports
- [ ] Update `AnyEnvironmentState` enum with new variants

### Phase 2: Update State Integration (1 hour)

- [ ] Add type erasure (`into_any`) methods for new states
- [ ] Add type restoration (`try_into_*`) methods for new states
- [ ] Add state introspection methods (`state_name`, etc.)
- [ ] Update universal transition methods

### Phase 3: Update Command Handler (2 hours)

- [ ] Modify destroy command handler to use `start_destroying()` transition
- [ ] Implement error tracking pattern similar to provision command
- [ ] Add `DestroyFailureContext` building logic
- [ ] Update error handling to use `destroy_failed()` transition

### Phase 4: Testing and Documentation (1 hour)

- [ ] Add unit tests for new state transitions
- [ ] Add integration tests for destroy command error scenarios
- [ ] Update state lifecycle documentation
- [ ] Verify type-state pattern consistency

## Manual E2E Testing

After implementing the changes, follow this manual testing procedure to verify the destroy state transitions work correctly:

### Step 1: Provision Test Infrastructure

Run the E2E provisioning tests with the `--keep` flag to maintain infrastructure for manual testing:

```bash
cargo run --bin e2e-provision-and-destroy-tests -- --keep
```

This will:

- Create a VM/container with environment name `e2e-provision`
- Complete the provisioning workflow
- Skip automatic cleanup due to `--keep` flag
- Leave infrastructure ready for manual destroy testing

### Step 2: Verify Initial State

Check that the environment is in a provisioned state:

```bash
# Check environment state file exists
ls -la data/e2e-provision/environment.json

# Check LXD VM exists
lxc list | grep e2e-provision

# Verify build directory exists
ls -la build/e2e-provision/
```

### Step 3: Test Production Destroy Command

Run the production destroy command to test the new state transitions:

```bash
cargo run -- destroy e2e-provision
```

With the new implementation, this should:

1. Load the environment from `data/e2e-provision/environment.json`
2. Transition to `Destroying` state (should be logged)
3. Execute infrastructure destruction via OpenTofu
4. Transition to `Destroyed` state on success
5. Clean up state files and directories

### Step 4: Verify Complete Cleanup

Confirm that all resources have been properly destroyed:

```bash
# Verify LXD VM is destroyed
lxc list | grep e2e-provision
# Should return no results

# Verify data directory is cleaned up
ls -la data/e2e-provision/
# Should not exist or be empty

# Verify build directory is cleaned up
ls -la build/e2e-provision/
# Should not exist

# Check that environment.json is removed
ls -la data/e2e-provision/environment.json
# Should not exist
```

### Step 5: Test Error Scenarios (Optional)

To test the `DestroyFailed` state transition:

1. **Create another test environment**:

   ```bash
   cargo run --bin e2e-provision-and-destroy-tests -- --keep
   ```

2. **Manually break the infrastructure** (simulate partial failure):

   ```bash
   # Remove the VM manually but leave state files
   lxc delete e2e-provision --force
   ```

3. **Run destroy command**:

   ```bash
   cargo run -- destroy e2e-provision
   ```

4. **Verify error state**:

   - Should transition to `DestroyFailed` state
   - Should preserve error context in state file
   - Should log appropriate error messages

### Expected Log Output

With proper state transitions, you should see logs like:

```text
INFO destroy_command: Starting complete infrastructure destruction workflow
INFO destroy_command: Transitioning to Destroying state
INFO destroy_command: Executing infrastructure destruction
INFO destroy_command: Infrastructure destruction completed successfully
INFO destroy_command: Transitioning to Destroyed state
```

## Acceptance Criteria

- [ ] `Destroying` state exists and follows same pattern as `Provisioning`
- [ ] `DestroyFailed` state exists and follows same pattern as `ProvisionFailed`
- [ ] Both states are properly integrated into `AnyEnvironmentState` enum
- [ ] Destroy command handler transitions through `Destroying` state before final state
- [ ] Failed destroy operations transition to `DestroyFailed` with proper error context
- [ ] State transitions are type-safe and compile-time enforced
- [ ] All existing tests continue to pass
- [ ] New tests cover the destroy state transition scenarios

## Related Documentation

- [Environment State Documentation](../src/domain/environment/state/mod.rs) - Current state machine
- [Command State Return Pattern ADR](../docs/decisions/command-state-return-pattern.md) - Type-state pattern
- [Error Handling Guide](../docs/contributing/error-handling.md) - Error context patterns
- [Provision Command Handler](../src/application/command_handlers/provision.rs) - Reference implementation

## Notes

### Why This Change Is Important

1. **Consistency**: All other operations (provision, configure, release) follow the transition → success/fail pattern
2. **Error Representation**: Partial destroy failures leave the system in an ambiguous state without proper error states
3. **User Experience**: Users need to know if a destroy operation is in progress or has failed
4. **Debugging**: Error context helps users understand what went wrong during destruction
5. **Type Safety**: The current direct transition bypasses the type-state pattern benefits

### Backwards Compatibility

This change maintains backwards compatibility because:

- The `destroyed()` method result remains the same for successful operations
- Failed operations that previously threw exceptions will now return typed error states
- Existing serialized environments will continue to deserialize correctly

### Future Considerations

This change enables future enhancements like:

- Retry mechanisms for failed destroy operations
- Progress tracking during long-running destroy operations
- Partial cleanup recovery procedures
- Better error reporting and user guidance
