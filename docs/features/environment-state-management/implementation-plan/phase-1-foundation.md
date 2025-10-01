# Phase 1: Foundation - Implementation Plan

> **📋 Detailed Plan**  
> Breaking down Phase 1 into three manageable, testable subtasks.

## 🎯 Phase 1 Overview

**Goal**: Establish the foundation for type-safe state management using the type-state pattern at compile time.

**Why We Need This**: Enable compile-time verification of valid state transitions to catch bugs early and make invalid states unrepresentable.

**Approach**: Use Rust's type system to encode state as a type parameter, making invalid state transitions impossible to compile.

## 📋 Implementation Subtasks

### Subtask 1: Create State Marker Types Module ✅

**Purpose**: Define all possible states as distinct types for the type-state pattern.

**Changes**:

- Create new file `src/domain/environment_state.rs`
- Define all state marker types as distinct types:
  - Success states: `Created`, `Provisioning`, `Provisioned`, `Configuring`, `Configured`, `Releasing`, `Released`, `Running`, `Destroyed`
  - Error states: `ProvisionFailed`, `ConfigureFailed`, `ReleaseFailed`, `RunFailed` (with `failed_step: String`)
- Derive `Debug`, `Clone`, `Serialize`, `Deserialize` for all state types
- Add `PartialEq`, `Eq` for testing
- Add module to `src/domain/mod.rs`

**Implementation Details**:

```rust
use serde::{Deserialize, Serialize};

// Success states
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Created;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provisioning;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provisioned;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Configuring;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Configured;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Releasing;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Released;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Running;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Destroyed;

// Error states with context
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvisionFailed {
    pub failed_step: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigureFailed {
    pub failed_step: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseFailed {
    pub failed_step: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunFailed {
    pub failed_step: String,
}
```

**Tests to Add**:

- Test creation of each state type
- Test `Clone` works for all states
- Test `PartialEq` and `Eq` for all states
- Test serialization/deserialization for success states
- Test serialization/deserialization for error states with failed_step
- Test `Debug` formatting

**Success Criteria**:

- ✅ All 13 state types defined (9 success + 4 error)
- ✅ All derives work correctly
- ✅ Module exports in `mod.rs`
- ✅ All linters pass
- ✅ All tests pass

**Commit**: `feat: add state marker types for environment state machine`

**Status**: ✅ Completed (Commit: a7317f5)

---

### Subtask 2: Convert Environment to Generic Environment<S> ✅

**Purpose**: Make `Environment` generic over state type to enable compile-time state tracking.

**Changes**:

- Modify `Environment` struct in `src/domain/environment.rs` to be generic over state type `S`
- Add default type parameter `S = Created` for backward compatibility
- Add `state: S` field to the struct
- Update `Environment::new()` to return `Environment<Created>` (initial state)
- Add generic implementations for common methods (getters for name, ssh_credentials, directories)

**Implementation Details**:

```rust
use crate::domain::environment_state::Created;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment<S = Created> {
    name: EnvironmentName,
    instance_name: InstanceName,
    profile_name: ProfileName,
    ssh_credentials: SshCredentials,
    build_dir: PathBuf,
    data_dir: PathBuf,
    state: S,
}

// Common methods available for all states
impl<S> Environment<S> {
    pub fn name(&self) -> &EnvironmentName {
        &self.name
    }

    pub fn instance_name(&self) -> &InstanceName {
        &self.instance_name
    }

    pub fn profile_name(&self) -> &ProfileName {
        &self.profile_name
    }

    pub fn ssh_credentials(&self) -> &SshCredentials {
        &self.ssh_credentials
    }

    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    pub fn build_dir(&self) -> &PathBuf {
        &self.build_dir
    }

    pub fn state(&self) -> &S {
        &self.state
    }

    // ... other directory methods
}

// Constructor creates Environment in Created state
impl Environment<Created> {
    pub fn new(name: EnvironmentName, ssh_credentials: SshCredentials) -> Self {
        let data_dir = PathBuf::from("./data").join(name.as_str());
        let build_dir = PathBuf::from("./build").join(name.as_str());
        let instance_name = InstanceName::from_environment_name(&name);
        let profile_name = ProfileName::from_environment_name(&name);

        Self {
            name,
            instance_name,
            profile_name,
            ssh_credentials,
            build_dir,
            data_dir,
            state: Created,
        }
    }
}
```

**Tests to Update**:

- Update existing tests to use `Environment<Created>` explicitly or rely on default
- Verify all existing tests still pass without modification (backward compatibility)
- Add test to verify `state()` getter returns correct state

**Success Criteria**:

- ✅ `Environment` is generic over state type `S`
- ✅ Default type parameter maintains backward compatibility
- ✅ All existing tests pass without modification
- ✅ Common methods work for all state types
- ✅ All linters pass
- ✅ All tests pass (468 tests)

**Commit**: `refactor: convert Environment to generic type-state struct`

**Status**: ✅ Completed (Commit: 6b57708)

---

### Subtask 3: Implement State Transition Methods ✅

**Purpose**: Add state-specific transition methods that consume `self` and return new typed `Environment` with compile-time validation.

**Changes**:

- Implement state-specific transition methods:
  - `Environment<Created>::start_provisioning() -> Environment<Provisioning>`
  - `Environment<Provisioning>::provisioned() -> Environment<Provisioned>`
  - `Environment<Provisioning>::provision_failed(String) -> Environment<ProvisionFailed>`
  - `Environment<Provisioned>::start_configuring() -> Environment<Configuring>`
  - `Environment<Configuring>::configured() -> Environment<Configured>`
  - `Environment<Configuring>::configure_failed(String) -> Environment<ConfigureFailed>`
  - `Environment<Configured>::start_releasing() -> Environment<Releasing>`
  - `Environment<Releasing>::released() -> Environment<Released>`
  - `Environment<Releasing>::release_failed(String) -> Environment<ReleaseFailed>`
  - `Environment<Released>::start_running() -> Environment<Running>`
  - `Environment<Running>::run_failed(String) -> Environment<RunFailed>`
  - `Environment<S>::destroy() -> Environment<Destroyed>` (for any state)

**Implementation Details**:

```rust
// State-specific implementations
impl Environment<Created> {
    #[must_use]
    pub fn start_provisioning(self) -> Environment<Provisioning> {
        Environment {
            name: self.name,
            instance_name: self.instance_name,
            profile_name: self.profile_name,
            ssh_credentials: self.ssh_credentials,
            build_dir: self.build_dir,
            data_dir: self.data_dir,
            state: Provisioning,
        }
    }
}

impl Environment<Provisioning> {
    #[must_use]
    pub fn provisioned(self) -> Environment<Provisioned> {
        Environment {
            name: self.name,
            instance_name: self.instance_name,
            profile_name: self.profile_name,
            ssh_credentials: self.ssh_credentials,
            build_dir: self.build_dir,
            data_dir: self.data_dir,
            state: Provisioned,
        }
    }

    #[must_use]
    pub fn provision_failed(self, failed_step: String) -> Environment<ProvisionFailed> {
        Environment {
            name: self.name,
            instance_name: self.instance_name,
            profile_name: self.profile_name,
            ssh_credentials: self.ssh_credentials,
            build_dir: self.build_dir,
            data_dir: self.data_dir,
            state: ProvisionFailed { failed_step },
        }
    }
}

// ... similar for all other states

// Universal destroy method
impl<S> Environment<S> {
    #[must_use]
    pub fn destroy(self) -> Environment<Destroyed> {
        Environment {
            name: self.name,
            instance_name: self.instance_name,
            profile_name: self.profile_name,
            ssh_credentials: self.ssh_credentials,
            build_dir: self.build_dir,
            data_dir: self.data_dir,
            state: Destroyed,
        }
    }
}
```

**Tests to Add**:

- Test each state transition individually
- Test full happy path: Created → Provisioning → Provisioned → Configuring → Configured → Releasing → Released → Running
- Test error transitions from each intermediate state
- Test destroy from multiple states
- Test that all fields are preserved during transitions

**Success Criteria**:

- ✅ All state transition methods implemented
- ✅ Transitions consume self (move semantics)
- ✅ All fields preserved during transitions
- ✅ Comprehensive unit tests for all transitions
- ✅ All linters pass
- ✅ All tests pass (468 + 15 new = 483 tests)

**Commit**: `feat: implement type-safe state transition methods`

**Status**: ✅ Completed (Commit: f8cd563)

---

## 🎯 Phase 1 Completion Criteria

When all three subtasks are complete, we should have:

- [x] All state marker types defined and tested
- [x] Environment struct is generic over state type
- [x] All state transitions are type-safe and compile-time validated
- [x] Existing tests pass without modification (using `Environment<Created>`)
- [x] All linters pass
- [x] All unit tests pass

## 📊 Test Results

- **Total Tests**: 468 unit tests + 4 integration tests + 61 doc tests = **533 tests passing**
- **New Tests Added**: 15 state transition tests
- **Linters**: All passing (markdown, yaml, toml, cspell, clippy, rustfmt, shellcheck)

## 🔄 What Comes Next

Phase 1 provides the compile-time foundation. Phase 2 will add runtime capabilities:

- Type erasure with `AnyEnvironmentState` enum
- Bidirectional type conversion
- State introspection helpers
- Full serialization/deserialization support

## 🚀 Key Achievements

### Type Safety

The type-state pattern provides compile-time guarantees:

```rust
let env = Environment::new(env_name, ssh_creds);

// ✅ This compiles - valid transition
let env = env.start_provisioning();

// ❌ This doesn't compile - invalid transition
// let env = env.configured(); // ERROR: method not found

// ✅ This compiles - valid transition
let env = env.provisioned();

// ✅ This compiles - valid transition
let env = env.start_configuring();
```

### Backward Compatibility

All existing code continues to work:

```rust
// Before Phase 1:
let env = Environment::new(name, creds);

// After Phase 1 (still works due to default type parameter):
let env = Environment::new(name, creds); // Environment<Created>
```

### Move Semantics

State transitions use move semantics to prevent reuse of old states:

```rust
let env = Environment::new(name, creds);
let env = env.start_provisioning();
// env is moved, can't be used anymore ✅

let env = env.provisioned();
// previous env is moved, can't be used anymore ✅
```

## 📚 Related Documentation

- [Main Implementation Plan](./README.md) - Overall feature roadmap
- [Phase 2 Plan](./phase-2-state-transition-logging.md) - State Transition Observability
- [Phase 3 Plan](./phase-3-serialization.md) - Serialization & Type Erasure
- [Feature Spec](../README.md) - Overall feature goals and motivation
- [Error Handling Guide](../../../contributing/error-handling.md) - Error handling principles
