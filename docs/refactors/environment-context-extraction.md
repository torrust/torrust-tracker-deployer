# Environment Context Extraction - Reducing Pattern Matching Complexity

**Status**: 📋 Planning  
**Target**: `src/domain/environment/mod.rs`, `src/domain/environment/state/mod.rs`  
**Created**: October 8, 2025  
**Impact**: High - Eliminates repetitive pattern matching, improves maintainability  
**Effort**: Medium - Requires careful refactoring of core domain type

## 📋 Overview

### Problem Statement

The `Environment<S>` struct is generic over state `S`, implementing the type-state pattern for compile-time state machine guarantees. While this provides excellent type safety, it creates significant complexity in `AnyEnvironmentState` where accessing common fields requires exhaustive pattern matching across all 13 state variants.

**Current Pain Points:**

```rust
// Every accessor in AnyEnvironmentState requires this pattern:
pub fn name(&self) -> &EnvironmentName {
    match self {
        Self::Created(env) => env.name(),
        Self::Provisioning(env) => env.name(),
        Self::Provisioned(env) => env.name(),
        Self::Configuring(env) => env.name(),
        Self::Configured(env) => env.name(),
        Self::Releasing(env) => env.name(),
        Self::Released(env) => env.name(),
        Self::Running(env) => env.name(),
        Self::ProvisionFailed(env) => env.name(),
        Self::ConfigureFailed(env) => env.name(),
        Self::ReleaseFailed(env) => env.name(),
        Self::RunFailed(env) => env.name(),
        Self::Destroyed(env) => env.name(),
    }
}
```

This pattern is repeated for:

- `name()` - 13 arms
- `instance_name()` - 13 arms
- `profile_name()` - 13 arms
- `ssh_credentials()` - 13 arms
- `ssh_port()` - 13 arms
- `instance_ip()` - 13 arms

**Total**: 6 methods × 13 arms = **78 lines of repetitive pattern matching** just for field accessors.

### Goals

1. **Eliminate Pattern Matching**: Remove repetitive 13-arm match expressions for common field access
2. **Maintain Type Safety**: Preserve compile-time state machine guarantees
3. **Improve Maintainability**: Make it easier to add new fields or states
4. **Simplify Serialization**: Reduce JSON structure complexity
5. **Preserve API Compatibility**: Minimize breaking changes to existing code

### Success Criteria

- ✅ Reduce `AnyEnvironmentState` pattern matching by ~80%
- ✅ All tests pass without modification
- ✅ Compilation time does not increase significantly
- ✅ Zero runtime performance regression
- ✅ Type-state pattern benefits remain intact

## 📊 Progress Tracking

### Summary

- **Total Proposals**: 6
- **Completed**: 0
- **In Progress**: 0
- **Not Started**: 6

### Proposals by Phase

| Phase   | Proposals | Status         |
| ------- | --------- | -------------- |
| Phase 1 | 2         | ⏳ Not Started |
| Phase 2 | 2         | ⏳ Not Started |
| Phase 3 | 2         | ⏳ Not Started |

## 🎯 Solution Analysis

### Evaluation Criteria

Solutions are ranked by:

1. **Impact**: How much pattern matching is eliminated
2. **Simplicity**: Implementation complexity and learning curve
3. **Maintainability**: Long-term code clarity and ease of modification
4. **Compatibility**: Breaking changes required
5. **Performance**: Runtime overhead introduced

### Ranking System

- **Impact**: 🟢🟢🟢 (High), 🟢🟢 (Medium), 🟢 (Low)
- **Simplicity**: 🔵🔵🔵 (Simple), 🔵🔵 (Moderate), 🔵 (Complex)
- **Maintainability**: 🟣🟣🟣 (Excellent), 🟣🟣 (Good), 🟣 (Acceptable)
- **Breaking Changes**: 🔴🔴🔴 (Major), 🔴🔴 (Moderate), 🔴 (Minor), ✅ (None)

---

## Solution Rankings

### 🥇 Solution #1: Extract Common Fields into `EnvironmentContext` (RECOMMENDED)

**Impact**: 🟢🟢🟢 High  
**Simplicity**: 🔵🔵 Moderate  
**Maintainability**: 🟣🟣🟣 Excellent  
**Breaking Changes**: 🔴 Minor  
**Performance**: Zero overhead

#### Design

```rust
/// Core environment data that remains constant across all states
///
/// This struct contains all fields that do not change when the environment
/// transitions between states. Extracting these fields eliminates repetitive
/// pattern matching in `AnyEnvironmentState` while maintaining the type-state
/// pattern's compile-time guarantees.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    /// The validated environment name
    name: EnvironmentName,

    /// The instance name for this environment (auto-generated)
    instance_name: InstanceName,

    /// The profile name for this environment (auto-generated)
    profile_name: ProfileName,

    /// SSH credentials for connecting to instances in this environment
    ssh_credentials: SshCredentials,

    /// SSH port for connecting to instances in this environment
    ssh_port: u16,

    /// Build directory for this environment (auto-generated)
    build_dir: PathBuf,

    /// Data directory for this environment (auto-generated)
    data_dir: PathBuf,

    /// Instance IP address (populated after provisioning)
    instance_ip: Option<IpAddr>,
}

/// Environment with type-state pattern for compile-time state guarantees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment<S = Created> {
    /// Core environment data shared across all states
    core: EnvironmentContext,

    /// Current state of the environment in the deployment lifecycle
    state: S,
}
```

#### Implementation in `AnyEnvironmentState`

**Before (78 lines):**

```rust
pub fn name(&self) -> &EnvironmentName {
    match self {
        Self::Created(env) => env.name(),
        Self::Provisioning(env) => env.name(),
        // ... 11 more arms
    }
}

pub fn instance_name(&self) -> &InstanceName {
    match self {
        Self::Created(env) => env.instance_name(),
        Self::Provisioning(env) => env.instance_name(),
        // ... 11 more arms
    }
}
// ... 4 more similar methods
```

**After (13 lines):**

```rust
/// Get a reference to the environment core regardless of state
fn core(&self) -> &EnvironmentContext {
    match self {
        Self::Created(env) => &env.core,
        Self::Provisioning(env) => &env.core,
        // ... 11 more arms (but only once!)
    }
}

// All accessors become one-liners:
pub fn name(&self) -> &EnvironmentName {
    &self.core().name
}

pub fn instance_name(&self) -> &InstanceName {
    &self.core().instance_name
}

pub fn ssh_credentials(&self) -> &SshCredentials {
    &self.core().ssh_credentials
}

pub fn ssh_port(&self) -> u16 {
    self.core().ssh_port
}

pub fn instance_ip(&self) -> Option<IpAddr> {
    self.core().instance_ip
}
```

**Reduction**: From 78 lines to 13 lines = **83% reduction**

#### Benefits

- ✅ **Massive Code Reduction**: 83% reduction in pattern matching code
- ✅ **Single Source of Truth**: One place to match on state for all common fields
- ✅ **Type Safety Preserved**: State machine guarantees remain intact
- ✅ **Easier to Extend**: Adding new fields only requires updating `EnvironmentContext`
- ✅ **Clear Separation**: Distinguishes state-independent data from state-specific data
- ✅ **Improved Serialization**: Cleaner JSON structure with `core` nesting

#### Drawbacks

- 🔴 **Field Access Changes**: Direct field access becomes `env.core.name` instead of `env.name`
- 🔴 **Breaking Change**: Existing code accessing fields directly will need updates
- 🔴 **JSON Structure Change**: Serialized format changes (minor, but requires migration)

#### Migration Strategy

1. Create `EnvironmentContext` struct with all common fields
2. Update `Environment<S>` to contain `core: EnvironmentContext` and `state: S`
3. Update constructor to initialize `EnvironmentContext`
4. Implement `core()` method on `Environment<S>` for easy access: `pub fn core(&self) -> &EnvironmentContext`
5. Add `core_mut()` for mutable access if needed
6. Update all field accessors to delegate to `core`
7. Update `AnyEnvironmentState` with single `core()` helper method
8. Simplify all accessor methods to use `self.core()`
9. Update serialization/deserialization if needed
10. Update all tests and calling code

---

### 🥈 Solution #2: `EnvironmentContext` with `Deref` Trait

**Impact**: 🟢🟢🟢 High  
**Simplicity**: 🔵🔵 Moderate  
**Maintainability**: 🟣🟣🟣 Excellent  
**Breaking Changes**: ✅ None  
**Performance**: Zero overhead (compile-time transformation)

#### Design

Same as Solution #1, but implement `Deref<Target = EnvironmentContext>` for `Environment<S>`:

```rust
impl<S> Deref for Environment<S> {
    type Target = EnvironmentContext;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}
```

#### Benefits Over Solution #1

- ✅ **Zero Breaking Changes**: `env.name` still works, automatically derefs to `env.core.name`
- ✅ **API Compatibility**: Existing code continues to work unchanged
- ✅ **Transparent Access**: Users don't need to know about the `core` field
- ✅ **All Solution #1 Benefits**: Same code reduction and maintainability improvements

#### Drawbacks

- 🔴 **Hidden Complexity**: The `core` abstraction is less explicit
- 🔴 **Potential Confusion**: Deref can be confusing for new contributors
- 🔴 **Method Ambiguity**: If `EnvironmentContext` has methods, they could conflict with `Environment<S>` methods

#### Why This is #2 (Not #1)

While `Deref` eliminates breaking changes, it adds "magic" that may surprise developers. The explicit `core` field in Solution #1 makes the architecture more discoverable and maintainable long-term. However, if **API compatibility is critical**, this is the better choice.

---

### 🥉 Solution #3: Shared Reference Pattern with `Arc<EnvironmentContext>`

**Impact**: 🟢🟢🟢 High  
**Simplicity**: 🔵 Complex  
**Maintainability**: 🟣🟣 Good  
**Breaking Changes**: 🔴🔴 Moderate  
**Performance**: Slight overhead (atomic reference counting)

#### Design

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment<S = Created> {
    /// Shared reference to core environment data
    core: Arc<EnvironmentContext>,

    /// Current state of the environment in the deployment lifecycle
    state: S,
}
```

#### Benefits

- ✅ **Efficient State Transitions**: Only clone the `Arc`, not all data
- ✅ **Memory Efficiency**: Single allocation shared across state transitions
- ✅ **Thread Safety**: `Arc` enables sharing across threads

#### Drawbacks

- 🔴 **Mutation Complexity**: Requires `Arc::make_mut()` for modifications
- 🔴 **Reference Counting Overhead**: Atomic operations on every clone/drop
- 🔴 **Over-Engineering**: Unnecessary for current use case (environments are not heavily cloned)
- 🔴 **Serialization Complexity**: `Arc` requires custom serialization

#### When to Use

Only if:

- Environments are cloned very frequently
- Memory usage becomes a concern
- Thread-safety is required

**Recommendation**: Not needed for current codebase.

---

### Solution #4: Macro-Based Field Access Generation

**Impact**: 🟢🟢 Medium  
**Simplicity**: 🔵 Complex  
**Maintainability**: 🟣 Acceptable  
**Breaking Changes**: ✅ None  
**Performance**: Zero overhead

#### Design

```rust
macro_rules! impl_common_field_access {
    ($method:ident, $field:ident, $type:ty) => {
        pub fn $method(&self) -> $type {
            match self {
                Self::Created(env) => env.$field(),
                Self::Provisioning(env) => env.$field(),
                Self::Provisioned(env) => env.$field(),
                Self::Configuring(env) => env.$field(),
                Self::Configured(env) => env.$field(),
                Self::Releasing(env) => env.$field(),
                Self::Released(env) => env.$field(),
                Self::Running(env) => env.$field(),
                Self::ProvisionFailed(env) => env.$field(),
                Self::ConfigureFailed(env) => env.$field(),
                Self::ReleaseFailed(env) => env.$field(),
                Self::RunFailed(env) => env.$field(),
                Self::Destroyed(env) => env.$field(),
            }
        }
    };
}

// Usage:
impl_common_field_access!(name, name, &EnvironmentName);
impl_common_field_access!(instance_name, instance_name, &InstanceName);
// ...
```

#### Benefits

- ✅ **No Breaking Changes**: Existing API unchanged
- ✅ **DRY**: Reduces repetition through code generation

#### Drawbacks

- 🔴 **Doesn't Solve Root Problem**: Still 13-arm match, just hidden in macro
- 🔴 **Harder to Debug**: Macro expansion obscures actual code
- 🔴 **Maintenance Burden**: Macros are harder to modify and understand
- 🔴 **Poor IDE Support**: Less useful autocomplete and error messages

**Recommendation**: Avoid - masks the problem without solving it.

---

### Solution #5: Trait-Based Common Access

**Impact**: 🟢 Low  
**Simplicity**: 🔵🔵 Moderate  
**Maintainability**: 🟣 Acceptable  
**Breaking Changes**: 🔴🔴 Moderate  
**Performance**: Zero overhead

#### Design

```rust
pub trait EnvironmentCommon {
    fn name(&self) -> &EnvironmentName;
    fn instance_name(&self) -> &InstanceName;
    fn ssh_credentials(&self) -> &SshCredentials;
    fn ssh_port(&self) -> u16;
    fn instance_ip(&self) -> Option<IpAddr>;
}

impl<S> EnvironmentCommon for Environment<S> {
    fn name(&self) -> &EnvironmentName {
        &self.name
    }
    // ... other methods
}

impl EnvironmentCommon for AnyEnvironmentState {
    fn name(&self) -> &EnvironmentName {
        match self {
            // ... still 13 arms
        }
    }
}
```

#### Benefits

- ✅ **Explicit Interface**: Clear contract for common access
- ✅ **Polymorphism**: Can work with `dyn EnvironmentCommon`

#### Drawbacks

- 🔴 **Doesn't Eliminate Matching**: Still need 13-arm match in impl
- 🔴 **Added Complexity**: New trait without clear benefit
- 🔴 **Breaking Changes**: Adds trait bound requirements

**Recommendation**: Provides minimal benefit for added complexity.

---

### Solution #6: Helper Method with Internal Matching

**Impact**: 🟢 Low  
**Simplicity**: 🔵🔵🔵 Simple  
**Maintainability**: 🟣🟣 Good  
**Breaking Changes**: ✅ None  
**Performance**: Zero overhead

#### Design

```rust
impl AnyEnvironmentState {
    /// Get a reference to the inner typed environment
    ///
    /// This helper centralizes state matching and enables field access
    /// without repeating the match expression.
    fn inner_env(&self) -> &dyn EnvironmentAccessor {
        match self {
            Self::Created(env) => env,
            Self::Provisioning(env) => env,
            // ... 11 more arms
        }
    }

    pub fn name(&self) -> &EnvironmentName {
        self.inner_env().name()
    }
}
```

#### Benefits

- ✅ **No Breaking Changes**: API unchanged
- ✅ **Simple Implementation**: Just adds helper method
- ✅ **No Structural Changes**: `Environment<S>` unchanged

#### Drawbacks

- 🔴 **Requires Trait Object**: Needs `dyn EnvironmentAccessor` trait
- 🔴 **Still Has Pattern Matching**: Just moved to one place
- 🔴 **Doesn't Scale**: Adding fields still requires trait updates

**Recommendation**: Quick fix but doesn't address root cause.

---

## 🏆 Final Recommendation

### Winner: Solution #1 (EnvironmentContext Extraction)

**Primary Choice**: Extract common fields into `EnvironmentContext`

**Rationale**:

1. **Biggest Impact**: 83% reduction in pattern matching
2. **Clear Architecture**: Explicit separation of concerns
3. **Maintainability**: Easy to understand and extend
4. **Scalability**: Adding fields or states becomes trivial

**Runner-Up**: Solution #2 (with `Deref`) if API compatibility is absolutely critical.

---

## 📅 Implementation Plan

### Phase 1: Core Extraction (Sprint 1, Week 1)

#### Proposal #1: Create `EnvironmentContext` Struct

**Status**: ✅ Complete  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P0  
**Estimated Time**: 2-3 hours  
**Actual Time**: ~2 hours

##### Problem

The `Environment<S>` struct contains 8 fields, but only 1 (`state: S`) changes between states. The other 7 fields are copied through every state transition, and accessing them in `AnyEnvironmentState` requires 13-arm pattern matching.

##### Proposed Solution

Extract all state-independent fields into a new `EnvironmentContext` struct:

```rust
/// Core environment data that remains constant across all states
///
/// This struct contains all fields that do not change when the environment
/// transitions between states. It represents the immutable identity and
/// configuration of an environment, separate from its mutable lifecycle state.
///
/// # Design Rationale
///
/// By separating state-independent data from the state machine, we:
/// - Eliminate repetitive pattern matching in `AnyEnvironmentState`
/// - Make it clear which data is constant vs. state-dependent
/// - Simplify state transitions (only the state field changes)
/// - Enable easier extension of environment configuration
///
/// # Field Overview
///
/// - **Identity**: `name`, `instance_name`, `profile_name`
/// - **Configuration**: `ssh_credentials`, `ssh_port`
/// - **Paths**: `build_dir`, `data_dir`
/// - **Runtime State**: `instance_ip` (populated after provisioning)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentContext {
    /// The validated environment name
    pub(crate) name: EnvironmentName,

    /// The instance name for this environment (auto-generated)
    pub(crate) instance_name: InstanceName,

    /// The profile name for this environment (auto-generated)
    pub(crate) profile_name: ProfileName,

    /// SSH credentials for connecting to instances in this environment
    pub(crate) ssh_credentials: SshCredentials,

    /// SSH port for connecting to instances in this environment
    pub(crate) ssh_port: u16,

    /// Build directory for this environment (auto-generated)
    pub(crate) build_dir: PathBuf,

    /// Data directory for this environment (auto-generated)
    pub(crate) data_dir: PathBuf,

    /// Instance IP address (populated after provisioning)
    ///
    /// This field stores the IP address of the provisioned instance and is
    /// `None` until the environment has been successfully provisioned.
    /// Once set, it's carried through all subsequent state transitions.
    pub(crate) instance_ip: Option<IpAddr>,
}
```

##### Implementation Checklist

Core Structure:

- [x] Create `EnvironmentContext` struct in `src/domain/environment/mod.rs`
- [x] Add all 8 state-independent fields
- [x] Derive `Debug`, `Clone`, `Serialize`, `Deserialize`, `PartialEq`
- [x] Add comprehensive documentation explaining purpose and design rationale
- [x] Extract to dedicated module `src/domain/environment/context.rs`

Field Visibility:

- [x] Use `pub(crate)` for fields (internal to domain module)
- [x] Keep implementation details hidden from external modules

#### Proposal #2: Refactor `Environment<S>` to Use Core

**Status**: ✅ Complete  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵🔵 Medium  
**Priority**: P0  
**Depends On**: Proposal #1  
**Estimated Time**: 2-3 hours  
**Actual Time**: ~2 hours

##### Problem

After creating `EnvironmentContext`, we need to refactor `Environment<S>` to use it, which requires updating field access patterns and state transition logic.

##### Proposed Solution

```rust
/// Environment with type-state pattern for compile-time state guarantees
///
/// The environment is now composed of two parts:
/// - `core`: Immutable data shared across all states
/// - `state`: The current lifecycle state (changes through transitions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment<S = Created> {
    /// Core environment data shared across all states
    core: EnvironmentContext,

    /// Current state of the environment in the deployment lifecycle
    state: S,
}

impl Environment {
    pub fn new(
        name: EnvironmentName,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
    ) -> Environment<Created> {
        let env_str = name.as_str();

        // Generate instance name
        let instance_name_str = format!("torrust-tracker-vm-{env_str}");
        let instance_name = InstanceName::new(instance_name_str)
            .expect("Generated instance name should always be valid");

        // Generate profile name
        let profile_name_str = format!("torrust-profile-{env_str}");
        let profile_name = ProfileName::new(profile_name_str)
            .expect("Generated profile name should always be valid");

        // Generate paths
        let data_dir = PathBuf::from("data").join(env_str);
        let build_dir = PathBuf::from("build").join(env_str);

        let core = EnvironmentContext {
            name,
            instance_name,
            profile_name,
            ssh_credentials,
            ssh_port,
            build_dir,
            data_dir,
            instance_ip: None,
        };

        Environment {
            core,
            state: Created,
        }
    }
}

impl<S> Environment<S> {
    /// Get a reference to the environment core
    ///
    /// Provides access to all state-independent environment data.
    #[must_use]
    pub fn core(&self) -> &EnvironmentContext {
        &self.core
    }

    /// Get a mutable reference to the environment core
    ///
    /// Used for operations that need to modify core data, such as
    /// setting the instance IP after provisioning.
    fn core_mut(&mut self) -> &mut EnvironmentContext {
        &mut self.core
    }

    /// Internal helper: Creates a new environment with a different state
    fn with_state<T>(self, new_state: T) -> Environment<T> {
        tracing::info!(
            environment_name = %self.core.name,
            instance_name = %self.core.instance_name,
            from_state = std::any::type_name::<S>(),
            to_state = std::any::type_name::<T>(),
            "Environment state transition"
        );

        Environment {
            core: self.core,  // Move the core (no clone needed!)
            state: new_state,
        }
    }
}
```

##### Implementation Checklist

Struct Update:

- [x] Replace 8 individual fields with single `context: EnvironmentContext` field (renamed from 'core' to 'context')
- [x] Keep `state: S` field as-is
- [x] Update struct documentation

Constructor:

- [x] Update `Environment::new()` to create `EnvironmentContext` instance
- [x] Ensure all field initialization logic is preserved
- [x] Verify generated values (instance_name, profile_name, paths)

Core Access:

- [x] Implement `context(&self) -> &EnvironmentContext` method
- [x] Implement `context_mut(&mut self) -> &mut EnvironmentContext` method (private)
- [x] Add documentation explaining usage

State Transitions:

- [x] Update `with_state<T>()` to move `context` instead of copying fields
- [x] Verify state transition logging still works
- [x] Ensure no performance regression

Field Accessors:

- [x] Update all `pub fn field_name(&self)` methods to delegate to `self.context.field_name`
- [x] Maintain existing method signatures (no breaking changes at method level)
- [x] Update documentation if needed

Mutators:

- [x] Update `with_instance_ip()` to modify `context`
- [x] Verify any other mutation methods

Tests:

- [x] Run all existing unit tests (758 passing)
- [x] Run all doctests (107 passing)
- [x] Ensure no test modifications needed
- [x] Update test builders to use new structure
- [x] Fix doctest to avoid accessing private fields

---

### Phase 2: Simplify `AnyEnvironmentState` (Sprint 1, Week 2)

#### Proposal #3: Add `context()` Helper to `AnyEnvironmentState`

**Status**: ✅ Complete  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P1  
**Depends On**: Proposal #2  
**Estimated Time**: 1 hour  
**Actual Time**: ~30 minutes

##### Problem

After `Environment<S>` uses `EnvironmentContext`, we still have 6 methods in `AnyEnvironmentState` that each do 13-arm pattern matching to access fields.

##### Proposed Solution

Add a single private `core()` helper method that does the 13-arm match once:

```rust
impl AnyEnvironmentState {
    /// Get a reference to the environment core regardless of current state
    ///
    /// This helper method centralizes state matching for accessing
    /// state-independent data. Instead of pattern matching 6 times
    /// (once per field accessor), we match once and reuse the result.
    ///
    /// This is a private implementation detail that simplifies the
    /// public accessor methods.
    fn core(&self) -> &EnvironmentContext {
        match self {
            Self::Created(env) => env.core(),
            Self::Provisioning(env) => env.core(),
            Self::Provisioned(env) => env.core(),
            Self::Configuring(env) => env.core(),
            Self::Configured(env) => env.core(),
            Self::Releasing(env) => env.core(),
            Self::Released(env) => env.core(),
            Self::Running(env) => env.core(),
            Self::ProvisionFailed(env) => env.core(),
            Self::ConfigureFailed(env) => env.core(),
            Self::ReleaseFailed(env) => env.core(),
            Self::RunFailed(env) => env.core(),
            Self::Destroyed(env) => env.core(),
        }
    }
}
```

##### Implementation Checklist

- [x] Add private `context()` method to `AnyEnvironmentState`
- [x] Implement 13-arm match expression
- [x] Call `env.context()` in each arm
- [x] Add inline documentation
- [x] Verify it compiles
- [x] Add `#[allow(dead_code)]` temporarily (will be removed in Phase 2.2)

#### Proposal #4: Simplify All Field Accessors

**Status**: ✅ Complete  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P1  
**Depends On**: Proposal #3  
**Estimated Time**: 1 hour  
**Actual Time**: ~30 minutes

##### Problem

Currently, each of the 6 field accessor methods has a 13-arm match (78 total lines). With the `core()` helper, we can reduce each to a one-liner.

##### Proposed Solution

**Before:**

```rust
pub fn name(&self) -> &EnvironmentName {
    match self {
        Self::Created(env) => env.name(),
        Self::Provisioning(env) => env.name(),
        Self::Provisioned(env) => env.name(),
        Self::Configuring(env) => env.name(),
        Self::Configured(env) => env.name(),
        Self::Releasing(env) => env.name(),
        Self::Released(env) => env.name(),
        Self::Running(env) => env.name(),
        Self::ProvisionFailed(env) => env.name(),
        Self::ConfigureFailed(env) => env.name(),
        Self::ReleaseFailed(env) => env.name(),
        Self::RunFailed(env) => env.name(),
        Self::Destroyed(env) => env.name(),
    }
}
```

**After:**

```rust
pub fn name(&self) -> &EnvironmentName {
    &self.core().name
}
```

Apply this pattern to all 6 methods:

- `name()` → `&self.core().name`
- `instance_name()` → `&self.core().instance_name`
- `profile_name()` → `&self.core().profile_name`
- `ssh_credentials()` → `&self.core().ssh_credentials`
- `ssh_port()` → `self.core().ssh_port`
- `instance_ip()` → `self.core().instance_ip`

##### Implementation Checklist

- [x] Replace `name()` implementation (13 lines → 1 line)
- [x] Replace `instance_name()` implementation (13 lines → 1 line)
- [x] Replace `profile_name()` implementation (13 lines → 1 line)
- [x] Replace `ssh_credentials()` implementation (13 lines → 1 line)
- [x] Replace `ssh_port()` implementation (13 lines → 1 line)
- [x] Replace `instance_ip()` implementation (13 lines → 1 line)
- [x] Remove `#[allow(dead_code)]` from `context()` method
- [x] Verify all tests pass (758 unit + 107 doc tests)
- [x] Run `cargo test` to confirm no regressions

##### Metrics

- **Lines Removed**: 72 (6 methods × 12 redundant match arms)
- **Lines Added**: 6 (6 one-liner implementations)
- **Net Reduction**: 66 lines (**-92% code**)

---

### Phase 3: Polish and Documentation (Sprint 1, Week 2)

#### Proposal #5: Add `EnvironmentContext` Helper Methods

**Status**: ✅ Complete  
**Impact**: 🟢🟢 Medium  
**Effort**: 🔵 Low  
**Priority**: P2  
**Depends On**: Proposal #2  
**Estimated Time**: 2 hours  
**Actual Time**: ~30 minutes

##### Problem

Many `Environment<S>` methods are derived path calculations (e.g., `templates_dir()`, `ansible_build_dir()`, `tofu_templates_dir()`). These should be available on `EnvironmentContext` for consistency.

##### Proposed Solution

Move derived path methods to `EnvironmentContext`:

```rust
impl EnvironmentContext {
    /// Returns the templates directory for this environment
    ///
    /// Path: `data/{env_name}/templates/`
    #[must_use]
    pub fn templates_dir(&self) -> PathBuf {
        self.data_dir.join("templates")
    }

    /// Returns the traces directory for this environment
    ///
    /// Path: `data/{env_name}/traces/`
    #[must_use]
    pub fn traces_dir(&self) -> PathBuf {
        self.data_dir.join(TRACES_DIR_NAME)
    }

    /// Returns the ansible build directory
    ///
    /// Path: `build/{env_name}/ansible`
    #[must_use]
    pub fn ansible_build_dir(&self) -> PathBuf {
        self.build_dir.join("ansible")
    }

    /// Returns the tofu build directory
    ///
    /// Path: `build/{env_name}/tofu`
    #[must_use]
    pub fn tofu_build_dir(&self) -> PathBuf {
        self.build_dir.join("tofu")
    }

    /// Returns the ansible templates directory
    ///
    /// Path: `data/{env_name}/templates/ansible`
    #[must_use]
    pub fn ansible_templates_dir(&self) -> PathBuf {
        self.templates_dir().join("ansible")
    }

    /// Returns the tofu templates directory
    ///
    /// Path: `data/{env_name}/templates/tofu`
    #[must_use]
    pub fn tofu_templates_dir(&self) -> PathBuf {
        self.templates_dir().join("tofu")
    }

    /// Returns the SSH username for this environment
    #[must_use]
    pub fn ssh_username(&self) -> &Username {
        &self.ssh_credentials.ssh_username
    }

    /// Returns the SSH private key path
    #[must_use]
    pub fn ssh_private_key_path(&self) -> &PathBuf {
        &self.ssh_credentials.ssh_priv_key_path
    }

    /// Returns the SSH public key path
    #[must_use]
    pub fn ssh_public_key_path(&self) -> &PathBuf {
        &self.ssh_credentials.ssh_pub_key_path
    }
}
```

Then delegate from `Environment<S>`:

```rust
impl<S> Environment<S> {
    pub fn templates_dir(&self) -> PathBuf {
        self.core.templates_dir()
    }

    pub fn traces_dir(&self) -> PathBuf {
        self.core.traces_dir()
    }

    // ... etc for all derived methods
}
```

##### Implementation Checklist

- [x] Move derived path methods to `EnvironmentContext` impl block (8 methods total)
- [x] Add comprehensive documentation for each method
- [x] Delegate from `Environment<S>` to `context` methods
- [x] Ensure consistency in method naming and return types
- [x] Update or add unit tests for `EnvironmentContext` methods
- [x] Run all tests to verify functionality (758 unit + 107 doc tests passing)

##### Benefits

- ✅ Cleaner API: Related methods live on the same type
- ✅ Reusability: Can call core methods directly when holding `&EnvironmentContext`
- ✅ Consistency: All core-related logic in one place

#### Proposal #6: Update Documentation and Examples

**Status**: ✅ Complete  
**Impact**: 🟢 Low  
**Effort**: 🔵 Low  
**Priority**: P3  
**Depends On**: Proposals #1-5  
**Estimated Time**: 1-2 hours  
**Actual Time**: ~30 minutes

##### Problem

After structural changes, documentation and examples may reference outdated field access patterns or explain the old structure.

##### Proposed Solution

Update documentation to explain:

1. **The Two-Part Structure**: Core + State
2. **When to Use What**: Direct core access vs. Environment methods
3. **State Transitions**: How core is moved, not cloned
4. **AnyEnvironmentState**: How pattern matching is minimized

Example documentation addition:

````rust
/// # Architecture: Core + State Design
///
/// The `Environment` is composed of two distinct parts:
///
/// ## `EnvironmentContext` - Immutable Identity
///
/// Contains all data that does not change during the environment's lifecycle:
/// - Identity: `name`, `instance_name`, `profile_name`
/// - Configuration: `ssh_credentials`, `ssh_port`
/// - Paths: `build_dir`, `data_dir`, `templates_dir()`, etc.
/// - Runtime state: `instance_ip` (set once after provisioning)
///
/// ## `state: S` - Mutable Lifecycle State
///
/// Tracks the current phase in the deployment lifecycle:
/// - Success states: `Created`, `Provisioning`, `Provisioned`, `Configuring`, etc.
/// - Error states: `ProvisionFailed`, `ConfigureFailed`, etc.
///
/// ## Benefits of This Design
///
/// - **Compile-time safety**: Invalid state transitions caught at compile time
/// - **Reduced pattern matching**: Access common fields without matching on state
/// - **Clear separation**: Identity vs. lifecycle are distinct concerns
/// - **Easy extension**: Adding fields or states is straightforward
///
/// ## Usage Patterns
///
/// ```rust
/// // Access core data directly
/// let env_name = environment.core().name;
///
/// // Or use convenience methods that delegate to core
/// let env_name = environment.name();
///
/// // State-specific operations
/// let env = env.start_provisioning();  // Moves core, changes state
/// ```
````

##### Implementation Checklist

Module Documentation:

- [x] Update `src/domain/environment/mod.rs` module-level documentation
- [x] Explain the Context + State architecture in module docs
- [x] Update usage examples to reflect new structure

Struct Documentation:

- [x] Update `Environment<S>` documentation with Context + State explanation
- [x] Update `EnvironmentContext` documentation (already comprehensive)
- [x] Update `AnyEnvironmentState` documentation (context() helper documented)

Code Examples:

- [x] Review and update all doctests in `mod.rs` (all 107 passing)
- [x] Ensure examples compile and pass
- [x] Fix clippy doc_markdown warnings (add backticks to code terms)

ADR Reference:

- [ ] Add a note referencing this refactoring plan
- [ ] Link to decision rationale in relevant documentation

---

## 📈 Expected Outcomes

### Code Metrics

| Metric                               | Before | After | Change |
| ------------------------------------ | ------ | ----- | ------ |
| `AnyEnvironmentState` accessor lines | 78     | 13    | -83%   |
| Total pattern match arms (accessors) | 78     | 13    | -83%   |
| Struct field count (`Environment`)   | 9      | 2     | -78%   |
| Lines in `state/mod.rs`              | ~1833  | ~1770 | -3.4%  |

### Maintainability Improvements

- ✅ **Adding a new field**: Just update `EnvironmentContext` (1 place vs. 13 places)
- ✅ **Adding a new state**: Just add enum variant and 1 match arm in `core()`
- ✅ **Field accessor changes**: Modify `EnvironmentContext` impl only
- ✅ **Debugging**: Clearer separation of concerns

### Performance

- ✅ **Zero runtime overhead**: Same compiled code, just reorganized
- ✅ **State transitions**: Actually more efficient (move vs. copy fields)
- ✅ **Compilation time**: No measurable impact expected

---

## 🧪 Testing Strategy

### Unit Tests

All existing tests should pass without modification:

```bash
cargo test --lib
```

Key test areas:

- ✅ `Environment` construction and initialization
- ✅ State transitions preserve core data
- ✅ Field accessors return correct values
- ✅ `AnyEnvironmentState` introspection methods
- ✅ Serialization/deserialization round-trips

### Integration Tests

```bash
cargo test --test '*'
```

Verify:

- ✅ Template generation uses correct paths
- ✅ Repository operations work with new structure
- ✅ Commands can access environment data

### E2E Tests

```bash
cargo run --bin e2e-tests-full
```

Ensure:

- ✅ Full deployment workflow works
- ✅ State persistence and loading
- ✅ Error states serialize correctly

---

## 🚨 Risk Assessment

### Low Risk

- **Compiler Enforcement**: Type system prevents most errors
- **No Behavioral Changes**: Same logic, different structure
- **Extensive Tests**: Existing test suite validates correctness

### Potential Issues

1. **Serialization Format Change**

   - **Risk**: JSON structure changes from flat to nested
   - **Mitigation**: Version state files, provide migration if needed
   - **Impact**: Low (E2E environments are ephemeral)

2. **Field Access Pattern Change**

   - **Risk**: Some code might directly access fields
   - **Mitigation**: Compilation errors will catch all cases
   - **Impact**: Low (fields were private, accessed via methods)

3. **Breaking Change in Public API**
   - **Risk**: External code might be affected
   - **Mitigation**: This is internal domain code, not public API
   - **Impact**: None (no external consumers)

---

## 📚 References

### Related Documentation

- [Type-State Pattern ADR](../decisions/type-erasure-for-environment-states.md)
- [Development Principles](../development-principles.md)
- [Module Organization](../contributing/module-organization.md)

### Background Reading

- **Rust Design Patterns**: Type-State Pattern
  - <https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html>
- **Separation of Concerns**: Domain-Driven Design
  - Distinguish identity, configuration, and state
- **Effective Rust**: Composition over inheritance
  - Use struct composition to separate concerns

### Similar Refactorings

- **File Lock Improvements** - Similar pattern matching reduction
- **Command State Return Pattern** - Type-state pattern application
- **Error Context Strategy** - Structural improvements for maintainability

---

## 🎓 Lessons Learned

### What Worked Well

To be filled during implementation:

- Compile-time safety prevented errors
- Test suite caught regressions early
- Clear separation improved understanding

### What Could Be Improved

To be filled during implementation:

- Migration strategy refinements
- Documentation completeness
- Test coverage for edge cases

### Recommendations for Future

To be filled during implementation:

- Consider extraction patterns for other domain types
- Document architectural patterns more explicitly
- Balance type safety with ergonomics

---

**Last Updated**: October 8, 2025  
**Author**: Development Team  
**Reviewers**: TBD  
**Status**: Awaiting approval to begin implementation
