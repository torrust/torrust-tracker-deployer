# Environment State Management Feature

> **🎯 Feature Overview**  
> Persistent environment state tracking to improve observability and user experience during deployment operations.

## 📋 Problem Statement

Currently, the Torrust Tracker Deploy application assumes that all deployment operations succeed. When operations fail during any deployment phase, the application halts and shows error logs, but the deployment state is not persisted.

### Current Issues

- **No State Persistence**: Environment states only exist in memory during command execution
- **Poor Error Recovery**: Users must manually inspect `./build` and `./data` folders to understand deployment state
- **Limited Observability**: No way to know the current state of a deployment environment without reading logs
- **User Experience**: Difficult to understand what went wrong and where to continue

### Current Architecture Context

- **No Production CLI**: Currently using E2E tests to validate functionality
- **Memory-Only Environments**: Environments are not persisted between command executions
- **File-Based Artifacts**: Configuration files are stored in `./data/{ENV_NAME}` folders
- **Three-Layer Architecture**: Commands → Steps → Remote Operations

## 🎯 Solution Overview

Implement persistent environment state management with a **type-state pattern** to track deployment progress and enable better error recovery guidance. This approach ensures that invalid state transitions are caught at compile-time rather than runtime.

### Happy Path State Flow

```text
created → provisioning → provisioned → configuring → configured → releasing → released → running → destroyed
```

### Error States

- `provision_failed`
- `configure_failed`
- `release_failed`
- `run_failed`

## 📊 Feature Specification

### 1. State Granularity & Transitions

#### State Types

- **Intermediate States**: Track command execution progress (`provisioning`, `configuring`, etc.)
- **Final States**: Track command completion (`provisioned`, `configured`, etc.)
- **Error States**: Track failures with step context (`provision_failed`, etc.)

#### State Transitions

- **Command Start**: Transition to intermediate state (e.g., `created` → `provisioning`)
- **Command Success**: Transition to final state (e.g., `provisioning` → `provisioned`)
- **Command Failure**: Transition to error state with step information

#### Error Context

- Store the name of the step that failed within the command
- Example: `provision_failed` with step name "cloud_init_execution"

### 2. State Persistence Strategy

#### Storage Location

- File path: `./data/{ENV_NAME}/state.json`
- Contains the complete Environment object including current state

#### Storage Content

- **Complete Environment Object**: All environment data, not just state
- **Current State**: Latest state enum value
- **No History**: State transitions logged via tracing at info level with timestamps for audit trail
- **No Versioning**: State schema versioning deferred - deployer is used once for short-lived deployments (minutes)
- **Versioning Rationale**: Application helps users set up initial environment; long-lived environment management can add versioning later if needed
- **Future Enhancement**: Consider event sourcing model if transition history becomes critical domain requirement

#### Validation Strategy

- **No Infrastructure Validation**: Don't validate actual infrastructure matches stored state in initial iteration
- **Future Enhancement**: Add validation in `status` or `test` commands

### 3. Command Integration Points

#### State Transition Timing

- **Command Start**: Update state to intermediate state (e.g., `provisioning`)
- **Command Completion**: Update state to final state (e.g., `provisioned`)
- **Both Events**: Track both start and completion for full visibility

#### Interrupted Commands

- Interrupted commands remain in intermediate state (e.g., `provisioning`)
- Provides clear indication that operation was in progress
- **Future Enhancement**: Track step-level progress or sub-states

### 4. Status Query Implementation

#### Status Command Scope

- **Current State Only**: Display current environment state
- **Single Environment**: Require environment name parameter
- **Simple Output**: Just the state, additional info via logging

#### Infrastructure Validation

- **Not Implemented**: No validation of actual infrastructure state
- **Future Enhancement**: Add validation capabilities

### 5. Repository Pattern & Storage

#### Repository Design

- **Generic StateRepository Trait**: Support multiple storage backends
- **Initial Implementation**: JSON file storage
- **Future Backends**: Database, remote storage, etc.

#### Atomic Operations

- **Atomic Writes**: Use temp file + rename pattern
- **Corruption Prevention**: Avoid partial writes during failures
- **Consistency**: Ensure state file integrity
- **File Locking**: Implement lock mechanism to prevent concurrent access and race conditions
- **Lock Ownership**: Use process ID in lock files to identify which process holds the lock

### 6. Error Recovery Considerations

#### Error Information

- **Basic Error Context**: Store step name that failed
- **Future Enhancement**: Detailed error information and recovery suggestions

#### Manual Recovery

- **Not Implemented**: No manual state transitions in initial iteration
- **Current Approach**: Users informed via internal logging and must destroy environment manually using OpenTofu commands
- **No Destroy Command Yet**: Manual cleanup requires running underlying OpenTofu commands directly
- **Future Enhancement**: Implement destroy command and manual state reset capabilities

## 🧪 Acceptance Criteria

### Core Functionality

- [ ] Environment states persist across command executions
- [ ] State transitions follow defined state machine rules with compile-time validation
- [ ] Failed commands transition to appropriate error states
- [ ] Type-safe serialization and deserialization of all state types

### Error Handling

- [ ] Failed commands store step context in error states
- [ ] Interrupted commands remain in intermediate states
- [ ] Storage corruption is prevented through atomic operations

### Integration

- [ ] All existing commands update state appropriately
- [ ] State management doesn't break existing E2E tests
- [ ] Commands can only be called on valid state types (compile-time enforced)

### Quality

- [ ] Comprehensive unit test coverage
- [ ] E2E tests validate state persistence
- [ ] Error scenarios are properly tested
- [ ] Documentation is complete and accurate
- [ ] All state transitions logged at info level with timestamps
- [ ] File locking mechanism prevents concurrent access issues

## 🔄 Future Enhancements

### Iteration 2: Status Query & CLI Integration

- Implement production CLI framework with subcommand structure
- Add `status` command for environment state visibility
- Provide type-safe state extraction for command execution
- Add user-friendly state display with recovery suggestions

### Iteration 3: Enhanced Error Recovery

- Store detailed error information and recovery suggestions
- Add infrastructure state validation
- Implement manual state reset capabilities
- Add automated destroy command for environment cleanup
- Document manual OpenTofu cleanup procedures

### Iteration 4: Step-Level Progress Tracking

- Track progress within commands at step level
- Enable graceful continuation of interrupted operations
- Add sub-states for complex operations

### Iteration 5: Event Sourcing

- Implement full state transition history
- Add rollback capabilities
- Enable audit trail for deployment operations

## 🔗 Related Documentation

- [Development Principles](../../development-principles.md) - Observability and user experience principles
- [Deployment Overview](../../deployment-overview.md) - Current deployment states and commands
- [Codebase Architecture](../../codebase-architecture.md) - Three-layer architecture context
- [Error Handling Guide](../../contributing/error-handling.md) - Error handling best practices
- [Implementation Plan](./implementation-plan.md) - Detailed implementation roadmap
- [Requirements Analysis](./requirements-analysis.md) - Questions and answers that defined this specification

## 📝 Implementation Notes

### State Machine Design

Use type-state pattern with distinct types for each state to enforce valid transitions at compile-time:

```rust
// Each state is a distinct type
pub struct Created;
pub struct Provisioning;
pub struct Provisioned;
// ... etc

// Environment is parameterized by state type
pub struct Environment<S> {
    name: EnvironmentName,
    ssh_credentials: SshCredentials,
    state: S,
}

// Only valid transitions are available
impl Environment<Created> {
    pub fn start_provisioning(self) -> Environment<Provisioning> { /* ... */ }
}

impl Environment<Provisioning> {
    pub fn provisioned(self) -> Environment<Provisioned> { /* ... */ }
    pub fn provision_failed(self, step: String) -> Environment<ProvisionFailed> { /* ... */ }
}
```

### Repository Pattern

Abstract storage to enable future enhancements, with type erasure for serialization:

```rust
// Type erasure enum for storage
pub enum AnyEnvironmentState {
    Created(Environment<Created>),
    Provisioning(Environment<Provisioning>),
    // ... etc
}

pub trait StateRepository {
    fn save(&self, env: &AnyEnvironmentState) -> Result<(), StateError>;
    fn load(&self, env_name: &EnvironmentName) -> Result<Option<AnyEnvironmentState>, StateError>;
    fn exists(&self, env_name: &EnvironmentName) -> Result<bool, StateError>;
}
```

### Command Integration

Update commands to use type-safe state transitions:

```rust
// Commands accept and return specific state types
impl ProvisionCommand {
    pub async fn execute(
        &self,
        environment: Environment<Created>
    ) -> Result<Environment<Provisioned>, ProvisionError> {
        let provisioning_env = environment.start_provisioning();
        // ... execute steps ...
        Ok(provisioning_env.provisioned())
    }
}

// Compile-time enforcement - this won't compile:
// let created_env = Environment::new(...);
// let configure_cmd = ConfigureCommand::new();
// configure_cmd.execute(created_env); // ERROR: expects Environment<Provisioned>
```

This feature will significantly improve the observability and user experience of the Torrust Tracker Deploy application by providing clear visibility into deployment state and better guidance during error scenarios.
