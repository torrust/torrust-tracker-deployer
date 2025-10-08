# Environment Context Three-Way Split - Semantic Field Separation

**Status**: 📋 Planning  
**Target**: `src/domain/environment/mod.rs`  
**Created**: October 8, 2025  
**Impact**: Medium - Improves semantic clarity and separation of concerns  
**Effort**: Low - Simple refactoring building on the existing `EnvironmentContext` structure

> **📝 NOTE**: This refactoring builds on the completed Environment Context Extraction refactoring. The `EnvironmentContext` struct now exists and can be split into three semantic types.

## 📋 Overview

### Problem Statement

The `EnvironmentContext` struct (created in the previous refactoring) groups all non-state fields together, but these fields have **three distinct semantic purposes**:

1. **User-Provided Inputs** (`name`, `instance_name`, `profile_name`, `ssh_credentials`, `ssh_port`)

   - Provided by the user when creating the environment
   - Immutable throughout the environment lifecycle
   - Represents user configuration choices

2. **Internal Configuration** (`build_dir`, `data_dir`)

   - Derived automatically from user inputs
   - Not directly controlled by users
   - Internal implementation details for organizing artifacts

3. **Runtime Outputs** (`instance_ip`, and more expected in the future)
   - Generated during deployment operations
   - Mutable as operations progress
   - Represents runtime state of deployed infrastructure

**Current Structure** (After EnvironmentContext Extraction):

```rust
pub struct EnvironmentContext {
    // Type 1: User inputs
    name: EnvironmentName,
    instance_name: InstanceName,
    profile_name: ProfileName,
    ssh_credentials: SshCredentials,
    ssh_port: u16,

    // Type 2: Internal config
    build_dir: PathBuf,
    data_dir: PathBuf,

    // Type 3: Runtime outputs
    instance_ip: Option<IpAddr>,
    // Future: container_id, deployment_timestamp, resource_metrics, etc.
}
```

While this is much better than having 8 individual fields, it doesn't make the semantic distinction between these three categories explicit in the type system.

### Goals

1. **Semantic Clarity**: Make the three categories of fields explicit in the type system
2. **Future-Proofing**: Prepare for more runtime outputs (container IDs, timestamps, metrics)
3. **Separation of Concerns**: Group related fields by their semantic purpose
4. **Maintainability**: Make it obvious where new fields should be added
5. **Documentation**: Types self-document the purpose of each field category

### Success Criteria

- ✅ Three distinct types representing the three semantic categories
- ✅ `EnvironmentContext` composes these three types
- ✅ All tests pass without modification
- ✅ API remains backward compatible (accessors work the same)
- ✅ Clear documentation explaining each type's purpose

### Non-Goals

- ❌ Changing the public API of `Environment<S>`
- ❌ Modifying serialization behavior
- ❌ Adding new functionality (pure refactoring)

## 📊 Progress Tracking

### Summary

- **Total Proposals**: 4
- **Completed**: 0
- **In Progress**: 0
- **Not Started**: 4

### Proposals by Phase

| Phase   | Proposals | Status         |
| ------- | --------- | -------------- |
| Phase 1 | 2         | ⏳ Not Started |
| Phase 2 | 2         | ⏳ Not Started |

## 🎯 Target Architecture

### After This Refactoring

```rust
/// User-provided configuration when creating an environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInputs {
    pub name: EnvironmentName,
    pub instance_name: InstanceName,
    pub profile_name: ProfileName,
    pub ssh_credentials: SshCredentials,
    pub ssh_port: u16,
}

/// Internal paths and configuration derived from user inputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalConfig {
    pub build_dir: PathBuf,
    pub data_dir: PathBuf,
}

/// Runtime outputs generated during deployment operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOutputs {
    pub instance_ip: Option<IpAddr>,
    // Future fields:
    // pub container_id: Option<String>,
    // pub deployment_timestamp: Option<DateTime<Utc>>,
    // pub resource_metrics: Option<ResourceMetrics>,
}

/// Complete environment context composed of three semantic types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    pub user_inputs: UserInputs,
    pub internal_config: InternalConfig,
    pub runtime_outputs: RuntimeOutputs,
}

/// Environment with type-state pattern
pub struct Environment<S = Created> {
    context: EnvironmentContext,
    state: S,
}
```

### Benefits

1. **Self-Documenting Code**: Types clearly indicate the purpose of each field
2. **Guided Development**: Developers know exactly where to add new fields
3. **Separation of Concerns**: Each type has a single, clear responsibility
4. **Future-Ready**: Easy to add more runtime outputs as deployment evolves
5. **Type System Enforcement**: Can't accidentally mix user inputs with runtime outputs

## 🚀 Implementation Plan

### Phase 1: Create Three Semantic Types (⏳ Not Started)

#### Proposal #1: Create UserInputs, InternalConfig, and RuntimeOutputs Structs

**Priority**: High  
**Effort**: Low  
**Impact**: High - Foundation for semantic separation

##### Description

Create three new structs to represent the three semantic categories of fields currently in `EnvironmentContext`.

##### Implementation

````rust
// src/domain/environment/mod.rs

/// User-provided configuration when creating an environment
///
/// This struct contains all fields that are provided by the user when creating
/// an environment. These fields are immutable throughout the environment lifecycle
/// and represent the user's configuration choices.
///
/// # Examples
///
/// ```rust
/// let user_inputs = UserInputs {
///     name: EnvironmentName::new("production".to_string())?,
///     instance_name: InstanceName::new("torrust-tracker-vm-production".to_string())?,
///     profile_name: ProfileName::new("torrust-profile-production".to_string())?,
///     ssh_credentials: SshCredentials::new(
///         PathBuf::from("keys/prod_rsa"),
///         PathBuf::from("keys/prod_rsa.pub"),
///         Username::new("torrust".to_string())?,
///     ),
///     ssh_port: 22,
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInputs {
    /// The validated environment name
    pub name: EnvironmentName,

    /// The instance name for this environment (auto-generated from name)
    pub instance_name: InstanceName,

    /// The profile name for this environment (auto-generated from name)
    pub profile_name: ProfileName,

    /// SSH credentials for connecting to instances in this environment
    pub ssh_credentials: SshCredentials,

    /// SSH port for connecting to instances in this environment
    pub ssh_port: u16,
}

/// Internal paths and configuration derived from user inputs
///
/// This struct contains fields that are derived automatically from user inputs
/// and are not directly controlled by users. These represent internal
/// implementation details for organizing build artifacts and data.
///
/// # Examples
///
/// ```rust
/// let internal_config = InternalConfig {
///     build_dir: PathBuf::from("build/production"),
///     data_dir: PathBuf::from("data/production"),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalConfig {
    /// Build directory for this environment (derived from environment name)
    pub build_dir: PathBuf,

    /// Data directory for this environment (derived from environment name)
    pub data_dir: PathBuf,
}

/// Runtime outputs generated during deployment operations
///
/// This struct contains fields that are generated during deployment operations
/// and represent the runtime state of deployed infrastructure. These fields
/// are mutable as operations progress.
///
/// # Future Fields
///
/// This struct is expected to grow as deployment operations become more complex:
/// - `container_id: Option<String>` - Container/VM identifier
/// - `deployment_timestamp: Option<DateTime<Utc>>` - When the environment was deployed
/// - `resource_metrics: Option<ResourceMetrics>` - CPU, memory, disk usage
/// - `service_endpoints: Option<Vec<ServiceEndpoint>>` - HTTP/TCP service URLs
///
/// # Examples
///
/// ```rust
/// use std::net::{IpAddr, Ipv4Addr};
///
/// let runtime_outputs = RuntimeOutputs {
///     instance_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeOutputs {
    /// Instance IP address (populated after provisioning)
    ///
    /// This field stores the IP address of the provisioned instance and is
    /// `None` until the environment has been successfully provisioned.
    pub instance_ip: Option<IpAddr>,
}
````

##### Checklist

**Definition:**

- [ ] Create `UserInputs` struct with 5 fields (name, instance_name, profile_name, ssh_credentials, ssh_port)
- [ ] Create `InternalConfig` struct with 2 fields (build_dir, data_dir)
- [ ] Create `RuntimeOutputs` struct with 1 field (instance_ip)
- [ ] Add comprehensive rustdoc for each struct explaining its purpose
- [ ] Add `#[derive(Debug, Clone, Serialize, Deserialize)]` to all structs

**Documentation:**

- [ ] Document that `UserInputs` are immutable user choices
- [ ] Document that `InternalConfig` is derived automatically
- [ ] Document that `RuntimeOutputs` grows during deployment
- [ ] Add future field examples in `RuntimeOutputs` documentation

**Testing:**

- [ ] Ensure all three structs can be serialized/deserialized
- [ ] Verify Debug output is readable

##### Notes

- Keep all fields public (`pub`) for simplicity
- These are pure data structs without behavior (behavior stays on `EnvironmentContext`)

##### Benefits

- ✅ Clear semantic boundaries between field categories
- ✅ Easy to understand where new fields belong
- ✅ Self-documenting code structure

---

#### Proposal #2: Refactor EnvironmentContext to Compose Three Types

**Priority**: High  
**Effort**: Low  
**Impact**: High - Completes the semantic separation

##### Description

Refactor `EnvironmentContext` to compose the three new semantic types instead of having individual fields.

##### Implementation

````rust
/// Complete environment context composed of three semantic types
///
/// The context is split into three logical categories:
/// 1. **User Inputs**: Configuration provided by users
/// 2. **Internal Config**: Derived paths for organizing artifacts
/// 3. **Runtime Outputs**: Data generated during deployment
///
/// This separation makes it clear where each piece of information comes from
/// and helps developers understand where to add new fields.
///
/// # Examples
///
/// ```rust
/// let context = EnvironmentContext {
///     user_inputs: UserInputs {
///         name: EnvironmentName::new("staging".to_string())?,
///         instance_name: InstanceName::new("torrust-tracker-vm-staging".to_string())?,
///         profile_name: ProfileName::new("torrust-profile-staging".to_string())?,
///         ssh_credentials: /* ... */,
///         ssh_port: 22,
///     },
///     internal_config: InternalConfig {
///         build_dir: PathBuf::from("build/staging"),
///         data_dir: PathBuf::from("data/staging"),
///     },
///     runtime_outputs: RuntimeOutputs {
///         instance_ip: None, // Populated after provisioning
///     },
/// };
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    /// User-provided configuration
    pub user_inputs: UserInputs,

    /// Internal paths and derived configuration
    pub internal_config: InternalConfig,

    /// Runtime outputs from deployment operations
    pub runtime_outputs: RuntimeOutputs,
}

impl EnvironmentContext {
    /// Creates a new environment context from user inputs
    ///
    /// This constructor automatically derives internal configuration from
    /// the provided user inputs.
    pub fn new(
        name: EnvironmentName,
        instance_name: InstanceName,
        profile_name: ProfileName,
        ssh_credentials: SshCredentials,
        ssh_port: u16,
    ) -> Self {
        let env_str = name.as_str();

        Self {
            user_inputs: UserInputs {
                name,
                instance_name,
                profile_name,
                ssh_credentials,
                ssh_port,
            },
            internal_config: InternalConfig {
                build_dir: PathBuf::from("build").join(env_str),
                data_dir: PathBuf::from("data").join(env_str),
            },
            runtime_outputs: RuntimeOutputs {
                instance_ip: None,
            },
        }
    }

    // Convenience accessors (backward compatibility)

    pub fn name(&self) -> &EnvironmentName {
        &self.user_inputs.name
    }

    pub fn instance_name(&self) -> &InstanceName {
        &self.user_inputs.instance_name
    }

    pub fn profile_name(&self) -> &ProfileName {
        &self.user_inputs.profile_name
    }

    pub fn ssh_credentials(&self) -> &SshCredentials {
        &self.user_inputs.ssh_credentials
    }

    pub fn ssh_port(&self) -> u16 {
        self.user_inputs.ssh_port
    }

    pub fn build_dir(&self) -> &PathBuf {
        &self.internal_config.build_dir
    }

    pub fn data_dir(&self) -> &PathBuf {
        &self.internal_config.data_dir
    }

    pub fn instance_ip(&self) -> Option<IpAddr> {
        self.runtime_outputs.instance_ip
    }

    pub fn set_instance_ip(&mut self, ip: IpAddr) {
        self.runtime_outputs.instance_ip = Some(ip);
    }

    // Derived path methods remain on EnvironmentContext
    pub fn templates_dir(&self) -> PathBuf {
        self.internal_config.data_dir.join("templates")
    }

    // ... other derived path methods
}
````

##### Checklist

**Refactoring:**

- [ ] Replace individual fields in `EnvironmentContext` with three composed structs
- [ ] Update `EnvironmentContext::new()` to create all three sub-structs
- [ ] Keep existing accessor methods for backward compatibility
- [ ] Update `set_instance_ip()` to modify `runtime_outputs.instance_ip`

**Testing:**

- [ ] Run all existing tests - they should pass without modification
- [ ] Verify serialization still works correctly
- [ ] Check that JSON structure is as expected

**Documentation:**

- [ ] Update `EnvironmentContext` rustdoc to explain the three-way split
- [ ] Document that accessor methods are for backward compatibility

##### Notes

- Keep all existing public methods for backward compatibility
- The three sub-structs are public, so direct field access is possible
- This is a pure refactoring - no functional changes

##### Benefits

- ✅ Clear semantic structure visible in the type system
- ✅ Backward compatible - all existing code continues to work
- ✅ Easy to extend with new runtime outputs

---

### Phase 2: Update Documentation and Examples (⏳ Not Started)

#### Proposal #3: Update Module Documentation

**Priority**: Medium  
**Effort**: Low  
**Impact**: Medium - Improves developer understanding

##### Description

Update the module-level documentation to explain the three-way semantic split and when to add fields to each type.

##### Implementation

````rust
//! Environment Domain Module
//!
//! This module contains all environment-related domain entities and types.
//!
//! ## Architecture
//!
//! The environment domain follows a clear separation of concerns:
//!
//! ### Three Semantic Categories
//!
//! Environment data is split into three distinct types based on their purpose:
//!
//! 1. **`UserInputs`** - Configuration provided when creating an environment
//!    - Immutable throughout environment lifecycle
//!    - Examples: name, SSH credentials, port numbers
//!    - Add new fields here when: User needs to configure something
//!
//! 2. **`InternalConfig`** - Derived configuration for internal use
//!    - Calculated from user inputs
//!    - Examples: build directory, data directory
//!    - Add new fields here when: Need internal paths or derived config
//!
//! 3. **`RuntimeOutputs`** - Data generated during deployment
//!    - Mutable as operations progress
//!    - Examples: IP addresses, container IDs, timestamps
//!    - Add new fields here when: Operations produce new data
//!
//! ### Composition
//!
//! These three types are composed in `EnvironmentContext`, which is then
//! paired with a state type parameter `S` in the `Environment<S>` struct
//! to implement the type-state pattern.
//!
//! ### Example
//!
//! ```rust
//! let env = Environment::new(
//!     EnvironmentName::new("production".to_string())?,
//!     ssh_credentials,
//!     22,
//! );
//!
//! // Access user inputs
//! let name = env.context.user_inputs.name;
//!
//! // Access internal config
//! let build_dir = env.context.internal_config.build_dir;
//!
//! // Access runtime outputs (populated during deployment)
//! if let Some(ip) = env.context.runtime_outputs.instance_ip {
//!     println!("Instance IP: {}", ip);
//! }
//! ```
````

##### Checklist

- [ ] Update module-level rustdoc (`//!` comments)
- [ ] Explain the three semantic categories
- [ ] Provide guidelines for when to add fields to each type
- [ ] Add example showing how to access each category
- [ ] Update any existing diagrams or documentation

##### Benefits

- ✅ Developers understand the architecture
- ✅ Clear guidance for adding new fields
- ✅ Reduces cognitive load when navigating code

---

#### Proposal #4: Add Unit Tests for Semantic Types

**Priority**: Medium  
**Effort**: Low  
**Impact**: Low - Documentation and verification

##### Description

Add unit tests that demonstrate and verify the three-way semantic split.

##### Implementation

```rust
#[cfg(test)]
mod three_way_split_tests {
    use super::*;

    #[test]
    fn it_should_separate_user_inputs_from_context() {
        let env = EnvironmentTestBuilder::new()
            .with_name("test-split")
            .build();

        // Can access user inputs directly
        assert_eq!(env.context.user_inputs.name.as_str(), "test-split");
        assert_eq!(env.context.user_inputs.ssh_port, 22);
    }

    #[test]
    fn it_should_derive_internal_config_automatically() {
        let env = EnvironmentTestBuilder::new()
            .with_name("test-derived")
            .build();

        // Internal config is derived from name
        let data_dir = &env.context.internal_config.data_dir;
        let build_dir = &env.context.internal_config.build_dir;

        assert!(data_dir.to_string_lossy().contains("test-derived"));
        assert!(build_dir.to_string_lossy().contains("test-derived"));
    }

    #[test]
    fn it_should_initialize_runtime_outputs_as_empty() {
        let env = EnvironmentTestBuilder::new()
            .with_name("test-runtime")
            .build();

        // Runtime outputs start empty
        assert_eq!(env.context.runtime_outputs.instance_ip, None);
    }

    #[test]
    fn it_should_populate_runtime_outputs_during_operations() {
        let mut env = EnvironmentTestBuilder::new()
            .with_name("test-populate")
            .build();

        // Simulate provisioning operation setting the IP
        let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        env.context.runtime_outputs.instance_ip = Some(ip);

        assert_eq!(env.context.runtime_outputs.instance_ip, Some(ip));
    }

    #[test]
    fn it_should_serialize_with_semantic_structure() {
        let env = EnvironmentTestBuilder::new()
            .with_name("test-serialize")
            .build();

        let json = serde_json::to_value(&env.context).unwrap();

        // Verify JSON has three top-level keys
        assert!(json.get("user_inputs").is_some());
        assert!(json.get("internal_config").is_some());
        assert!(json.get("runtime_outputs").is_some());
    }
}
```

##### Checklist

- [ ] Add test for direct user inputs access
- [ ] Add test for internal config derivation
- [ ] Add test for runtime outputs initialization
- [ ] Add test for runtime outputs population
- [ ] Add test for serialization structure

##### Benefits

- ✅ Tests document the intended use
- ✅ Verify the three-way split works correctly
- ✅ Catch regressions if structure changes

---

## 📅 Timeline

### Estimated Duration

- **Phase 1**: 2-4 hours
- **Phase 2**: 1-2 hours
- **Total**: 3-6 hours

### Dependencies

✅ **Prerequisite Complete**: The Environment Context Extraction refactoring has been completed. The `EnvironmentContext` struct exists with all 8 state-independent fields.

**Current State:**

1. ✅ `EnvironmentContext` struct created and implemented
2. ✅ `Environment<S>` refactored to use `context: EnvironmentContext`
3. ✅ All tests passing (758 unit + 107 doc tests)
4. ✅ Ready to proceed with three-way semantic split

### Sprint Planning

This refactoring can be completed in a single sprint (approximately half sprint):

- **Phase 1**: Create three semantic types (2-4 hours)
- **Phase 2**: Update documentation and tests (1-2 hours)
- **Total**: 3-6 hours

## 🎯 Review Process

### Approval Criteria

1. ✅ All proposals implemented and tested
2. ✅ All existing tests pass
3. ✅ Code review by at least one maintainer
4. ✅ Documentation is clear and comprehensive
5. ✅ Backward compatibility maintained

### Validation

- Run full test suite: `cargo test`
- Run E2E tests: `cargo run --bin e2e-tests-full`
- Check documentation builds: `cargo doc --no-deps`
- Run linters: `cargo run --bin linter all`

### Completion Checklist

- [ ] All 4 proposals implemented
- [ ] Tests pass
- [ ] Documentation updated
- [ ] Code reviewed and approved
- [ ] Merged to main branch
- [ ] This document moved to completed refactorings

## 📚 Related Documentation

- ~~[Environment Context Extraction](./environment-context-extraction.md)~~ - ✅ **Completed** prerequisite refactoring
- [Development Principles](../development-principles.md) - Observability and maintainability
- [Module Organization](../contributing/module-organization.md) - Code structure conventions

## 🔍 Future Considerations

### Expected Runtime Outputs

As deployment operations evolve, `RuntimeOutputs` is expected to grow with:

- `container_id: Option<String>` - LXD/Docker container identifier
- `deployment_timestamp: Option<DateTime<Utc>>` - When environment was deployed
- `resource_metrics: Option<ResourceMetrics>` - CPU, memory, disk usage
- `service_endpoints: Vec<ServiceEndpoint>` - HTTP/TCP service URLs
- `health_check_results: Option<HealthStatus>` - Service health information

### Alternative: Make RuntimeOutputs Extensible

If runtime outputs grow too large, consider making it extensible:

```rust
pub struct RuntimeOutputs {
    pub instance_ip: Option<IpAddr>,
    pub extensions: HashMap<String, serde_json::Value>,
}
```

This allows adding fields without breaking changes, but loses type safety.

## 💡 Key Takeaways

### Why This Refactoring?

1. **Semantic Clarity**: Types document the purpose of each field
2. **Future-Proofing**: Easy to add more runtime outputs
3. **Developer Guidance**: Clear where to add new fields
4. **Separation of Concerns**: Each type has one responsibility

### Impact Summary

| Aspect            | Before                       | After                                  |
| ----------------- | ---------------------------- | -------------------------------------- |
| Structure         | Flat 8 fields in Context     | 3 semantic types with clear boundaries |
| Adding Fields     | "Where does this go?"        | Clear category for each field type     |
| Documentation     | Generic field descriptions   | Type-level semantic documentation      |
| Runtime Data      | Mixed with inputs and config | Isolated in RuntimeOutputs             |
| Future Extensions | Unclear where to add         | Add to appropriate semantic type       |
