# Multi-Environment Support Implementation Plan

## 📋 Overview

This document outlines the implementation plan for adding multi-environment support to the Torrust Tracker Deploy application. Currently, the application only supports one hardcoded environment. This enhancement will enable users to deploy and manage multiple isolated environments (dev, staging, production, e2e-config, e2e-provision, e2e-full) concurrently.

## 🎯 Goals

- **Environment Isolation**: Each environment has its own configuration, build artifacts, and infrastructure
- **Concurrent Testing**: Support running multiple e2e test environments simultaneously without conflicts
- **Traceability**: Maintain all generated artifacts per environment for debugging and investigation
- **Clean Architecture**: Introduce proper domain entities following DDD principles
- **Future-Ready**: Design for eventual CLI argument/environment variable support

## 📐 Architecture Overview

### Current State

```text
data/
  templates/           # Shared templates
build/                 # Shared build output
TestContext            # Hardcoded paths and instance names
```

### Target State

```text
data/
  {env_name}/
    templates/         # Environment-specific templates
build/
  {env_name}/          # Environment-specific build output
Environment            # New domain entity encapsulating environment data
TestContext            # Updated to work with Environment entity
```

## 🏗️ Implementation Steps

### Step 1: Create EnvironmentName Domain Entity ✅ COMPLETED

**Location**: `src/domain/environment_name.rs`

**Requirements**:

- ✅ Restricted string format: lowercase letters, numbers, and dashes only (updated for InstanceName compatibility)
- ✅ Support common environment names: `dev`, `staging`, `production`, `e2e-config`, `e2e-provision`, `e2e-full`
- ✅ Validation with clear error messages
- ✅ Serializable/deserializable for future JSON storage
- ✅ Flexible constructor accepting anything convertible to String

**Implementation Details**:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EnvironmentName(String);

impl EnvironmentName {
    pub fn new<S: Into<String>>(name: S) -> Result<Self, EnvironmentNameError> {
        // Validates format: lowercase letters, numbers, dashes only
        // Compatible with InstanceName constraints for auto-generated instance names
    }
}
```

**Error Handling**:

- ✅ Clear validation errors explaining the allowed format
- ✅ Suggest correct format when validation fails
- ✅ Include examples of valid environment names

### Step 2: Create Environment Domain Entity ✅ COMPLETED

**Location**: `src/domain/environment.rs`

**Requirements**:

- ✅ Encapsulate all environment-specific configuration
- ✅ JSON serializable/deserializable for future state persistence
- ✅ Environment-specific directory path calculation
- ✅ Instance name generation with conflict avoidance

**Structure**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub name: EnvironmentName,
    pub instance_name: InstanceName,
    pub ssh_private_key_path: PathBuf,
    pub ssh_public_key_path: PathBuf,
    pub build_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Environment {
    pub fn new(
        name: EnvironmentName,
        ssh_private_key_path: PathBuf,
        ssh_public_key_path: PathBuf,
    ) -> Self {
        // ✅ Auto-generates instance_name as: torrust-tracker-vm-{env_name}
        // ✅ Auto-generates build_dir as: build/{env_name}
        // ✅ Auto-generates data_dir as: data/{env_name}
    }

    pub fn templates_dir(&self) -> PathBuf {
        self.data_dir.join("templates")
    }
}
```

### Step 3: Update Domain Module Structure ✅ COMPLETED

**Location**: `src/domain/mod.rs`

**Requirements**:

- ✅ Add new modules to domain exports
- ✅ Ensure proper module organization
- ✅ Update any existing code that might conflict

**Changes**:

```rust
pub mod environment;
pub mod environment_name;
// ... existing modules
```

### Step 4: Update E2E Binaries

**Files to Modify**:

- `src/bin/e2e_provision_tests.rs`
- `src/bin/e2e_config_tests.rs`
- `src/bin/e2e_tests_full.rs`

**Requirements**:

- Create Environment instances in each binary
- Use environment-specific configurations
- Maintain existing CLI argument compatibility where possible

**Example for e2e_config_tests.rs**:

```rust
let environment = Environment::new(
    EnvironmentName::new("e2e-config".to_string())?,
    ssh_private_key_path,
    ssh_public_key_path,
);

let test_context = TestContext::initialized(
    false,
    environment,
    &ssh_user,
    TestContextType::Container,
)?;
```

### Step 5: Update TestContext

**Location**: `src/e2e/context.rs`

**Requirements**:

- Replace individual parameters with Environment entity
- Maintain backward compatibility during transition
- Update all dependent code

**Key Changes**:

```rust
impl TestContext {
    pub fn initialized(
        skip_teardown: bool,
        environment: Environment,
        ssh_user: &Username,
        context_type: TestContextType,
    ) -> Result<Self, TestContextError> {
        // Use environment.templates_dir() instead of hardcoded path
        // Use environment.build_dir for build artifacts
        // Use environment.instance_name for instance creation
    }
}
```

### Step 6: Implement Environment-Specific Directory Structure

**Requirements**:

- Ensure directories are created automatically when needed
- Update all template generation to use environment-specific paths
- Update all build artifact generation to use environment-specific paths

**Directory Structure**:

```text
data/
  e2e-config/
    templates/
      ansible/
      tofu/
  e2e-provision/
    templates/
      ansible/
      tofu/
  production/
    templates/
      ansible/
      tofu/
build/
  e2e-config/
    ansible/
    tofu/
  e2e-provision/
    ansible/
    tofu/
  production/
    ansible/
    tofu/
```

### Step 7: Update Instance Naming for Isolation

**Requirements**:

- Change instance naming pattern to: `torrust-tracker-vm-{env_name}`
- Ensure no conflicts when running concurrent environments
- Update all references to instance names throughout the codebase

**Example Names**:

- `torrust-tracker-vm-e2e-config`
- `torrust-tracker-vm-e2e-provision`
- `torrust-tracker-vm-production`

### Step 8: Comprehensive Testing

**Requirements**:

- All linters must pass (`cargo run --bin linter all`)
- All unit tests must pass (`cargo test`)
- All e2e tests must pass:
  - `cargo run --bin e2e-provision-tests`
  - `cargo run --bin e2e-config-tests`
  - `cargo run --bin e2e-tests-full`

## 🚦 Quality Gates

Each step must pass these quality gates before proceeding:

1. **Linting**: `cargo run --bin linter all` ✅
2. **Unit Tests**: `cargo test` ✅
3. **E2E Tests**: All three e2e test suites pass ✅
4. **Documentation**: Code is properly documented ✅
5. **Error Handling**: Follows project error handling principles ✅

## 🧪 Testing Strategy

### Unit Tests

- Test EnvironmentName validation with various inputs
- Test Environment entity creation and path generation
- Test error scenarios with clear, actionable error messages

### Integration Tests

- Test environment isolation (creating multiple environments)
- Test template generation with environment-specific paths
- Test build artifact separation

### E2E Tests

- Verify each e2e test suite works with new environment system
- Test concurrent execution (if safe on the current system)
- Validate that artifacts are properly isolated

## 📝 Error Handling Requirements

Following the project's [error handling principles](../contributing/error-handling.md):

### Clarity

- Environment name validation errors must clearly explain the allowed format
- Path-related errors must include the attempted path and suggested fixes

### Context and Traceability

- Include environment name in all error messages
- Preserve source errors when wrapping filesystem or template errors

### Actionability

- Provide specific instructions for fixing validation errors
- Include examples of valid environment names
- Guide users on how to resolve path conflicts

### Example Error Messages

```rust
// Good: Clear, contextual, actionable
EnvironmentNameError::InvalidFormat {
    attempted_name: "Invalid-Name",
    reason: "contains uppercase letters and hyphens",
    valid_examples: vec!["dev", "staging", "e2e-config"],
}

// Error message should be:
// "Environment name 'Invalid-Name' is invalid: contains uppercase letters and hyphens.
//
//  Valid format: lowercase letters, underscores, and slashes only
//  Examples: dev, staging, production, e2e-config, e2e-provision"
```

## 🔮 Future Considerations

While not implemented in this phase, the design should support:

### CLI Integration

- `--environment dev` command line argument
- `TORRUST_ENV=production` environment variable support

### State Persistence

- `data/{env_name}/state.json` for environment state
- Environment configuration management commands

### Template Inheritance

- Shared base templates with environment-specific overrides
- Template composition and inheritance system

## 📚 References

- [Development Principles](../development-principles.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Testing Conventions](../contributing/testing.md)
- [Branching Conventions](../contributing/branching.md)
- [Commit Process](../contributing/commit-process.md)

## ✅ Acceptance Criteria

This feature is complete when:

1. ✅ All domain entities (EnvironmentName, Environment) are implemented with proper validation
2. ✅ All e2e binaries use the new Environment entity
3. ✅ Directory structure is environment-specific (data/{env_name}/, build/{env_name}/)
4. ✅ Instance naming prevents conflicts (torrust-tracker-vm-{env_name})
5. ✅ All linters pass without warnings
6. ✅ All unit tests pass
7. ✅ All e2e tests pass with the new environment system
8. ✅ Error handling follows project principles with clear, actionable messages
9. ✅ Code is properly documented with examples and usage guidelines
10. ✅ Multiple environments can be used simultaneously without conflicts

## 🎉 Success Metrics

- **Development**: Clean domain-driven design with proper separation of concerns
- **Testing**: All existing functionality continues to work with improved isolation
- **Maintainability**: Clear, self-documenting code with comprehensive error handling
- **Future-Ready**: Architecture supports planned CLI and state management features
