# Command Code Quality Improvements

## 📋 Overview

This refactoring plan addresses code quality, maintainability, readability, and testability issues in the command layer, specifically in `ProvisionCommand` and `ConfigureCommand`. Both commands share significant structural patterns and have opportunities for reducing duplication, improving test clarity, and enhancing overall code quality.

**Target Files:**

- `src/application/commands/provision.rs`
- `src/application/commands/configure.rs`
- Test modules within both files

**Scope:**

- Extract common patterns into reusable abstractions
- Improve test code quality and reduce boilerplate
- Fix architectural inconsistencies
- Enhance maintainability and readability

## 📊 Progress Tracking

**Total Active Proposals**: 5 (API Simplification + Quick Wins + Test Improvements)
**Total Postponed**: 4 (Will revisit after implementing 1-2 more commands)
**Total Discarded**: 1
**Completed**: 4
**In Progress**: 0
**Not Started**: 1

### Phase Summary

- **Phase 0 - API Simplification (High Impact, Low Effort)**: ✅ 1/1 completed
- **Phase 1 - Quick Wins (High Impact, Low Effort)**: ✅ 3/3 completed
- **Phase 2 - Test Improvements (Medium Impact, Medium Effort)**: 0/1 completed
- **Phase 3 - Advanced Patterns**: Postponed until 1-2 more commands implemented

### Discarded Proposals

- **Command Builder Pattern**: All dependencies are required, builder pattern would be misleading and doesn't provide sufficient value

### Postponed Proposals

These 4 proposals are deferred until 1-2 more commands exist to better understand common patterns:

- **Proposal #5 - Command Trait**: Will reconsider after implementing a third command to validate the abstraction
- **Proposal #6 - Typed Step Tracking**: Will reevaluate after implementing other refactorings to see if still needed
- **Proposal #7 - Template Method Pattern**: Needs more command examples to identify the right abstraction
- **Proposal #8 - Failure Context Helper**: Should wait until patterns stabilize across more commands

## 🎯 Key Problems Identified

### 1. Code Duplication

- **State Persistence**: Identical `persist_*_state()` methods in both commands with copy-pasted error handling
- **Failure Context Building**: Similar `build_failure_context()` methods following identical patterns
- **Trace Writing**: Duplicate trace file generation logic with identical error handling

### 2. Architectural Inconsistencies

- **Hard-coded Clock**: Trace writers instantiate `Arc::new(SystemClock)` instead of using the injected `clock` dependency
- **Clock Injection Ignored**: Commands receive a clock but don't use it consistently for all time-sensitive operations

### 3. Testability Issues

- **Complex Mock Setup**: `create_mock_dependencies()` functions are verbose (8-tuple returns) and hard to maintain
- **Test Boilerplate**: Similar test structures repeated across both files
- **Large Constructors**: Commands have 5-6 Arc-wrapped parameters making tests cumbersome

### 4. Readability Concerns

- **Manual Step Tracking**: Tuple return `(error, current_step)` pattern is error-prone and not self-documenting
- **Large Implementation Blocks**: Methods mix concerns (execution, persistence, error handling, trace generation)

## 🚀 Refactoring Phases

---

## Phase 0: API Simplification (Highest Priority)

Foundation changes that simplify the command API and should be done first.

### Proposal #0: Add instance_ip to Environment Context

**Status**: ✅ Completed  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P0 (Must do first - simplifies subsequent refactorings)  
**Completed**: October 7, 2025  
**Commit**: `d8673f5`

#### Problem

Currently, `ProvisionCommand` returns a tuple `(Environment<Provisioned>, IpAddr)` because the instance IP is needed by subsequent commands, but there's nowhere to store it in the environment.

```rust
pub async fn execute(
    &self,
    environment: Environment<Created>,
) -> Result<(Environment<Provisioned>, IpAddr), ProvisionCommandError> {
    // Returns tuple with IP separate from environment
}
```

Problems with current approach:

- **Complex return types**: Tuple `(Environment<T>, IpAddr)` is less ergonomic than just `Environment<T>`
- **IP separation**: The IP is logically part of the environment's execution context but has to be passed separately
- **Awkward API**: Callers must destructure tuples: `let (env, ip) = provision.execute(env)?;`
- **Inconsistency**: ConfigureCommand needs the IP but has to receive it separately from the environment

#### Two Alternatives Considered

**Alternative 1: Store IP in Provisioned state** ❌ Not Recommended

- Would violate separation of concerns (states represent lifecycle, not execution context)
- Requires passing IP through all subsequent state transitions
- Makes state transitions complex
- Not extensible (what about future outputs like ports, URLs?)

**Alternative 2: Add optional instance_ip to Environment** ✅ Recommended

- IP is execution context, belongs in Environment alongside data_dir, build_dir
- No impact on state transitions
- Extensible for future outputs
- Consistent with existing patterns

#### Proposed Solution

Add `instance_ip: Option<IpAddr>` to the `Environment<S>` struct as execution context:

```rust
// In src/domain/environment/mod.rs

pub struct Environment<S> {
    name: EnvironmentName,
    ssh_credentials: SshCredentials,
    data_dir: PathBuf,
    build_dir: PathBuf,
    state: S,
    instance_ip: Option<IpAddr>,  // NEW: Runtime context populated after provisioning
}

impl<S> Environment<S> {
    // Add getter for instance_ip
    pub fn instance_ip(&self) -> Option<IpAddr> {
        self.instance_ip
    }

    // Add setter for commands to populate the IP
    pub fn with_instance_ip(mut self, ip: IpAddr) -> Self {
        self.instance_ip = Some(ip);
        self
    }
}
```

**Updated ProvisionCommand:**

```rust
// Before
pub async fn execute(
    &self,
    environment: Environment<Created>,
) -> Result<(Environment<Provisioned>, IpAddr), ProvisionCommandError> {
    // ... provisioning logic ...
    let ip = get_instance_ip()?;
    let provisioned = provisioning_env.provisioned();
    Ok((provisioned, ip))
}

// After - much cleaner!
pub async fn execute(
    &self,
    environment: Environment<Created>,
) -> Result<Environment<Provisioned>, ProvisionCommandError> {
    // ... provisioning logic ...
    let ip = get_instance_ip()?;
    let provisioned = provisioning_env
        .provisioned()
        .with_instance_ip(ip);  // Store IP in environment context
    Ok(provisioned)
}
```

**Benefits for ConfigureCommand:**

```rust
// Before - IP passed separately
pub async fn execute(
    &self,
    environment: Environment<Provisioned>,
    instance_ip: IpAddr,  // Awkward separate parameter
) -> Result<Environment<Configured>, ConfigureCommandError> {
    // ...
}

// After - IP is part of environment
pub async fn execute(
    &self,
    environment: Environment<Provisioned>,
) -> Result<Environment<Configured>, ConfigureCommandError> {
    let ip = environment.instance_ip()
        .expect("Environment must have instance_ip after provisioning");
    // ...
}
```

#### Benefits

- ✅ **Simplified Return Types**: `Result<Environment<T>, Error>` instead of `Result<(Environment<T>, IpAddr), Error>`
- ✅ **Better API Ergonomics**: No tuple destructuring needed
- ✅ **Logical Cohesion**: IP is execution context, belongs with the environment
- ✅ **Consistent Pattern**: Follows existing pattern of data_dir, build_dir in Environment
- ✅ **Extensible**: Easy to add more optional outputs (ports, URLs) without changing state transitions
- ✅ **Type Safe Enough**: Option<IpAddr> makes it explicit when IP might not be present
- ✅ **Simpler State Transitions**: State transitions don't need to extract/carry execution outputs
- ✅ **Enables Other Refactorings**: Simplifies the API that other proposals will work with

#### Implementation Checklist

- [x] Add `instance_ip: Option<IpAddr>` field to `Environment<S>` struct
- [x] Add `instance_ip(&self) -> Option<IpAddr>` getter method
- [x] Add `with_instance_ip(self, ip: IpAddr) -> Self` builder method
- [x] Update `Environment::new()` to initialize `instance_ip: None`
- [x] Update `ProvisionCommand::execute()` return type to `Result<Environment<Provisioned>, Error>`
- [x] Update `ProvisionCommand` to call `.with_instance_ip(ip)` before returning
- [x] Update `ConfigureCommand::execute()` signature (already didn't have separate parameter)
- [x] Update all test code that calls these commands
- [x] Update state serialization/deserialization (handled automatically by serde)
- [x] Update `AnyState` enum (preserves `instance_ip` through type erasure automatically)
- [x] Verify all tests pass (758 tests passed)
- [x] Run linter and fix any issues (all linters passed)
- [x] Update documentation and API examples

#### Testing Strategy

- Verify provision command stores IP correctly
- Test that configure command can access IP from environment
- Test serialization/deserialization preserves IP
- Test state transitions maintain IP across states
- Verify IP is None for environments that haven't been provisioned
- Test error handling when IP is expected but missing

#### Migration Path

This is a breaking API change, but we're still in POC phase:

1. Update Environment struct and constructors
2. Update ProvisionCommand return type and implementation
3. Update ConfigureCommand signature and implementation
4. Update all call sites (main.rs, tests)
5. Update state persistence to include instance_ip

---

## Phase 1: Quick Wins (High Impact, Low Effort)

Quick improvements that significantly enhance code quality with minimal effort.

### Proposal #1: Remove Duplicate State Persistence Methods

**Status**: ✅ Completed  
**Completed**: October 7, 2025  
**Commit**: d84f380  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P1  
**Actual Time**: 1 hour

#### Problem

Both commands have identical `persist_*_state()` methods that are just wrappers around `repository.save()`:

```rust
// In provision.rs - duplicated 3 times
fn persist_provisioning_state(&self, environment: &Environment<Provisioning>) {
    let any_state = environment.clone().into_any();
    if let Err(e) = self.repository.save(&any_state) {
        warn!(
            environment = %environment.name(),
            error = %e,
            "Failed to persist provisioning state. Command execution continues."
        );
    }
}

// In configure.rs - duplicated 3 times with only state names changed
fn persist_configuring_state(&self, environment: &Environment<Configuring>) {
    let any_state = environment.clone().into_any();
    if let Err(e) = self.repository.save(&any_state) {
        warn!(
            environment = %environment.name(),
            error = %e,
            "Failed to persist configuring state. Command execution continues."
        );
    }
}
```

These methods:

- Violate DRY principles (6 nearly identical methods)
- Silently ignore persistence errors (wrong behavior - state must be saved)
- Mix concerns (repository logging should be in repository, not command)

#### Proposed Solution

**Remove these helper methods entirely** and use the repository directly:

1. **Update Repository to log errors** at the infrastructure layer where they occur
2. **Use `?` operator in commands** to propagate persistence errors (fail-fast)
3. **No service or trait needed** - just call `repository.save()` directly

**Repository logs errors:**

```rust
// In infrastructure/environment_repository.rs
impl EnvironmentRepository for JsonFileRepository {
    fn save(&self, environment: &Environment<AnyState>) -> Result<(), RepositoryError> {
        // ... write to file ...
        if let Err(e) = write_file() {
            error!(
                path = %file_path,
                error = %e,
                "Failed to save environment state to file"
            );
            return Err(RepositoryError::WriteFailed { path, source: e });
        }
        Ok(())
    }
}
```

**Commands propagate errors:**

```rust
// In provision.rs - Before (3 identical methods)
fn persist_provisioning_state(&self, environment: &Environment<Provisioning>) {
    let any_state = environment.clone().into_any();
    if let Err(e) = self.repository.save(&any_state) {
        warn!(/* ... */);
    }
}

// After - just one line, repeated inline where needed
let environment = environment.with_state(Provisioning);
self.repository.save(&environment.clone().into_any())?;  // Fail if save fails
```

#### Rationale

**Why no service/trait abstraction?**

- **Not really duplication**: It's just calling a dependency method with `?` operator
- **Single Responsibility**: Repository handles persistence, command handles orchestration
- **Proper error handling**: Persistence failures should stop command execution
- **Separation of concerns**: Repository logs at ERROR level, commands just propagate
- **YAGNI**: Don't create abstractions for simple pass-through calls

**Error Handling Philosophy:**

- **Before**: Silently ignore errors, continue execution (dangerous - wrong state persisted)
- **After**: Fail fast on persistence errors (correct - state must be saved or command fails)

This aligns with the observability principle: if we can't save state, the deployment is in an unknown state and should fail.

#### Benefits

- ✅ Eliminates 6 duplicate methods (3 per command)
- ✅ **Proper error handling**: Commands fail if state can't be saved
- ✅ **Clearer code**: Direct repository calls are more explicit
- ✅ **Single Responsibility**: Repository logs errors, commands propagate them
- ✅ **Simpler architecture**: No unnecessary service/trait abstraction
- ✅ **Better observability**: Repository logs at ERROR level when saves fail

#### Implementation Checklist

- [x] Remove `persist_provisioning_state()` from `ProvisionCommand`
- [x] Remove `persist_provisioned_state()` from `ProvisionCommand`
- [x] Remove `persist_provision_failed_state()` from `ProvisionCommand`
- [x] Remove `persist_configuring_state()` from `ConfigureCommand`
- [x] Remove `persist_configured_state()` from `ConfigureCommand`
- [x] Remove `persist_configure_failed_state()` from `ConfigureCommand`
- [x] Replace all method calls with direct `self.repository.save(&env.clone().into_any())?`
- [x] Add `StatePersistence` variant to `ProvisionCommandError` enum
- [x] Add `StatePersistence` variant to `ConfigureCommandError` enum
- [x] Add `StatePersistence` to `ErrorKind` enum for error classification
- [x] Update `Traceable` implementations for new error variants
- [x] Repository already has proper error context via `with_context()` calls
- [x] Verify all 100 tests pass
- [x] Run linter and ensure all checks pass

#### Results

- **Lines Removed**: 42 (6 duplicate methods eliminated)
- **Lines Added**: 6 (error variants)
- **Net Change**: -36 lines
- **Tests**: All 100 tests passing
- **Linters**: All linters passing
- **Error Handling**: Improved - persistence failures now stop command execution

---

### Proposal #2: Use Injected Clock for Trace Writers

**Status**: ✅ Completed  
**Completed**: October 7, 2025  
**Commit**: 4c116a0  
**Impact**: 🟢🟢🟢 High  
**Effort**: 🔵 Low  
**Priority**: P1  
**Actual Time**: 30 minutes

#### Problem

Both commands instantiate a new `SystemClock` when creating trace writers, despite having a `clock` dependency injected:

```rust
// In both provision.rs and configure.rs build_failure_context()
let traces_dir = environment.traces_dir();
let clock = Arc::new(SystemClock);  // ❌ Creates new clock instead of using self.clock
let trace_writer = ProvisionTraceWriter::new(traces_dir, clock);
```

This violates dependency injection principles and makes testing harder (can't inject a mock clock for trace timestamps).

#### Proposed Solution

Use the injected clock dependency:

```rust
// Before
let clock = Arc::new(SystemClock);
let trace_writer = ProvisionTraceWriter::new(traces_dir, clock);

// After
let trace_writer = ProvisionTraceWriter::new(traces_dir, Arc::clone(&self.clock));
```

#### Benefits

- ✅ Respects dependency injection pattern
- ✅ Enables clock mocking in tests
- ✅ Consistent time source across command execution
- ✅ Better testability for trace file timestamps
- ✅ Removes unnecessary `SystemClock` instantiation

#### Implementation Checklist

- [x] Update `build_failure_context()` in `provision.rs` to use `Arc::clone(&self.clock)`
- [x] Update `build_failure_context()` in `configure.rs` to use `Arc::clone(&self.clock)`
- [x] Remove unused `SystemClock` imports from both files
- [x] Verify trace files are generated correctly (tested with E2E tests)
- [x] Verify all tests pass (100 tests passed)
- [x] All linters passing

#### Testing Strategy

- Existing tests should work without changes
- Consider adding a test with a mock clock to verify trace timestamps use injected clock

---

### Proposal #3: Extract Common Trace Writing Pattern (Move Logging Inside write_trace)

**Status**: ✅ Completed  
**Completed**: October 7, 2025  
**Commit**: b033843  
**Impact**: 🟢🟢 Medium-High  
**Effort**: 🔵 Low  
**Priority**: P2  
**Actual Time**: 1 hour

#### Problem

Both commands have identical trace writing logic with duplicate error handling and logging:

```rust
// In provision.rs
match writer.write_trace(&context, error) {
    Ok(trace_file) => {
        info!(
            trace_id = %context.base.trace_id,
            trace_file = ?trace_file,
            "Generated trace file for provision failure"
        );
        context.base.trace_file_path = Some(trace_file);
    }
    Err(e) => {
        warn!(
            trace_id = %context.base.trace_id,
            error = %e,
            "Failed to generate trace file for provision failure"
        );
    }
}

// In configure.rs - nearly identical except for log messages
match trace_writer.write_trace(&context, error) {
    Ok(trace_file_path) => {
        info!(
            command = "configure",
            trace_id = %context.base.trace_id,
            trace_file = ?trace_file_path,
            "Trace file generated successfully"
        );
        context.base.trace_file_path = Some(trace_file_path);
    }
    Err(e) => {
        warn!(
            command = "configure",
            trace_id = %context.base.trace_id,
            error = %e,
            "Failed to generate trace file"
        );
    }
}
```

#### Proposed Solution

Move the logging logic directly into the `write_trace` method implementation. The trace writer should be responsible for logging its own operations.

**In `ProvisionTraceWriter` and `ConfigureTraceWriter`:**

```rust
// In src/infrastructure/trace/provision_trace_writer.rs (and configure_trace_writer.rs)

use tracing::{info, warn};

impl ProvisionTraceWriter {
    pub fn write_trace(
        &self,
        context: &ProvisionFailureContext,
        error: &impl Traceable,
    ) -> Result<PathBuf, std::io::Error> {
        // Generate trace content
        let trace_content = self.format_trace(context, error);

        // Write trace file
        match self.write_trace_file(&trace_content, &context.base.trace_id) {
            Ok(trace_file_path) => {
                info!(
                    command = "provision",
                    trace_id = %context.base.trace_id,
                    trace_file = ?trace_file_path,
                    "Trace file generated successfully"
                );
                Ok(trace_file_path)
            }
            Err(e) => {
                warn!(
                    command = "provision",
                    trace_id = %context.base.trace_id,
                    error = %e,
                    "Failed to generate trace file"
                );
                Err(e)
            }
        }
    }
}
```

**In commands (simplified usage):**

```rust
// Before - commands handle logging
let traces_dir = environment.traces_dir();
let trace_writer = ProvisionTraceWriter::new(traces_dir, Arc::clone(&self.clock));

match trace_writer.write_trace(&context, error) {
    Ok(trace_file) => {
        info!(/* ... */);  // Duplicate logging
        context.base.trace_file_path = Some(trace_file);
    }
    Err(e) => {
        warn!(/* ... */);  // Duplicate logging
    }
}

// After - trace writer handles logging internally
let traces_dir = environment.traces_dir();
let trace_writer = ProvisionTraceWriter::new(traces_dir, Arc::clone(&self.clock));

if let Ok(trace_file_path) = trace_writer.write_trace(&context, error) {
    context.base.trace_file_path = Some(trace_file_path);
}
// No else needed - trace writer already logged the error
```

#### Benefits

- ✅ Eliminates duplicate logging logic in commands
- ✅ Trace writers are responsible for their own logging (better encapsulation)
- ✅ Consistent logging format across all trace writers
- ✅ Commands have cleaner, simpler code
- ✅ Single place to modify trace logging behavior

#### Implementation Checklist

- [x] Move logging into `ProvisionTraceWriter::write_trace()` implementation
- [x] Move logging into `ConfigureTraceWriter::write_trace()` implementation
- [x] Update `ProvisionCommand::build_failure_context()` to remove logging
- [x] Update `ConfigureCommand::build_failure_context()` to remove logging
- [x] Remove unused `warn` import from provision.rs
- [x] Verify all tests pass (100 tests passed)
- [x] All linters passing

#### Testing Strategy

- Existing tests continued to work without changes
- Logging output is still generated correctly (verified in E2E tests)
- No new tests needed (logging moved, not changed)

---

## Phase 2: Test Improvements (Medium Impact, Medium Effort)

Improvements to test code quality, maintainability, and reducing boilerplate.

### Proposal #4: Test Builder Helpers

**Status**: ⏳ Not Started  
**Impact**: 🟢🟢 Medium-High  
**Effort**: 🔵🔵 Medium  
**Priority**: P2  
**Estimated Time**: 3 hours

#### Problem

Both command test files have verbose `create_mock_dependencies()` functions that return large tuples:

```rust
// In provision.rs tests (returns 8-tuple!)
fn create_mock_dependencies() -> (
    Arc<TofuTemplateRenderer>,
    Arc<AnsibleTemplateRenderer>,
    Arc<AnsibleClient>,
    Arc<OpenTofuClient>,
    Arc<dyn Clock>,
    Arc<dyn EnvironmentRepository>,
    SshCredentials,
    TempDir,
) {
    // 50+ lines of setup code
}
```

Problems:

- Hard to remember what each tuple element represents
- Adding a new dependency breaks all test call sites
- Can't selectively override individual dependencies

#### Proposed Solution

Create a dedicated test builder that works with the command builder:

```rust
// In src/application/commands/provision.rs (in #[cfg(test)] module)

#[cfg(test)]
pub struct ProvisionCommandTestBuilder {
    temp_dir: TempDir,
    ssh_credentials: Option<SshCredentials>,
    // Store components that we need to keep alive
    template_manager: Option<Arc<TemplateManager>>,
}

#[cfg(test)]
impl ProvisionCommandTestBuilder {
    pub fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        Self {
            temp_dir,
            ssh_credentials: None,
            template_manager: None,
        }
    }

    pub fn with_ssh_credentials(mut self, credentials: SshCredentials) -> Self {
        self.ssh_credentials = Some(credentials);
        self
    }

    pub fn build(self) -> (ProvisionCommand, TempDir, SshCredentials) {
        // Create default SSH credentials if not provided
        let ssh_credentials = self.ssh_credentials.unwrap_or_else(|| {
            let ssh_key_path = self.temp_dir.path().join("test_key");
            let ssh_pub_key_path = self.temp_dir.path().join("test_key.pub");
            SshCredentials::new(
                ssh_key_path,
                ssh_pub_key_path,
                Username::new("test_user").unwrap(),
            )
        });

        let template_manager = Arc::new(TemplateManager::new(self.temp_dir.path()));

        let tofu_renderer = Arc::new(TofuTemplateRenderer::new(
            template_manager.clone(),
            self.temp_dir.path(),
            ssh_credentials.clone(),
            InstanceName::new("torrust-tracker-vm".to_string()).unwrap(),
            ProfileName::new("default-profile".to_string()).unwrap(),
        ));

        let ansible_renderer = Arc::new(AnsibleTemplateRenderer::new(
            self.temp_dir.path(),
            template_manager.clone(),
        ));

        let ansible_client = Arc::new(AnsibleClient::new(self.temp_dir.path()));

        let opentofu_client = Arc::new(OpenTofuClient::new(self.temp_dir.path()));

        let clock: Arc<dyn Clock> = Arc::new(SystemClock);

        let repository_factory = RepositoryFactory::new(Duration::from_secs(30));
        let repository = repository_factory.create(self.temp_dir.path().to_path_buf());

        let command = ProvisionCommand::builder()
            .with_tofu_renderer(tofu_renderer)
            .with_ansible_renderer(ansible_renderer)
            .with_ansible_client(ansible_client)
            .with_opentofu_client(opentofu_client)
            .with_clock(clock)
            .with_repository(repository)
            .build()
            .expect("Failed to build ProvisionCommand for test");

        (command, self.temp_dir, ssh_credentials)
    }
}
```

**Usage in tests:**

```rust
// Before
let (tofu_renderer, ansible_renderer, ansible_client, opentofu_client, clock,
     repository, ssh_credentials, temp_dir) = create_mock_dependencies();
let command = ProvisionCommand::new(
    tofu_renderer, ansible_renderer, ansible_client,
    opentofu_client, clock, repository
);

// After
let (command, temp_dir, ssh_credentials) = ProvisionCommandTestBuilder::new().build();
```

#### Benefits

- ✅ Dramatically reduces test boilerplate
- ✅ Self-contained: manages TempDir lifetime properly
- ✅ Extensible: easy to add customization methods
- ✅ Type-safe: returns properly typed values
- ✅ Maintainable: changes to command dependencies don't break all tests

#### Implementation Checklist

- [ ] Create `ProvisionCommandTestBuilder` in test module
- [ ] Implement builder with sensible defaults
- [ ] Add customization methods (e.g., `with_ssh_credentials()`)
- [ ] Update all tests in `provision.rs` to use test builder
- [ ] Remove old `create_mock_dependencies()` function
- [ ] Create `ConfigureCommandTestBuilder` in test module
- [ ] Update all tests in `configure.rs` to use test builder
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

- Verify all existing tests work with new builder
- Test builder with default values
- Test builder with custom values
- Ensure TempDir cleanup works correctly

---

### Proposal #5: ~~Extract Common Command Trait~~ (DEFERRED)

**Status**: ⏸️ Deferred  
**Reason**: Will reconsider after implementing a third command. With only two commands, it's premature to abstract a common interface. The pattern will be clearer once we have more examples.

~~#### Problem

Both commands share similar lifecycle patterns but have no common interface:

- State persistence
- Error handling
- Failure context building
- Execution flow

This makes it harder to:

- Add new commands with consistent behavior
- Create generic command utilities
- Test commands uniformly

#### Proposed Solution

Define a `Command` trait that captures common command behavior:

```rust
// In src/application/commands/common/mod.rs

use std::sync::Arc;
use crate::domain::environment::repository::EnvironmentRepository;

/// Base trait for all commands in the application
pub trait Command {
    /// The environment state this command accepts as input
    type Input;

    /// The environment state this command produces on success
    type Output;

    /// The error type this command can return
    type Error: std::error::Error;

    /// Execute the command
    fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

/// Extension trait providing common command utilities
pub trait CommandExt: Command {
    /// Get the repository used by this command for state persistence
    fn repository(&self) -> &Arc<dyn EnvironmentRepository>;

    /// Get the clock used by this command for timing
    fn clock(&self) -> &Arc<dyn crate::shared::Clock>;
}
```

**Implementation for existing commands:**

```rust
impl Command for ProvisionCommand {
    type Input = Environment<Created>;
    type Output = (Environment<Provisioned>, IpAddr);
    type Error = ProvisionCommandError;

    fn execute(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        // Existing execute implementation
        self.execute(input)
    }
}

impl CommandExt for ProvisionCommand {
    fn repository(&self) -> &Arc<dyn EnvironmentRepository> {
        &self.repository
    }

    fn clock(&self) -> &Arc<dyn crate::shared::Clock> {
        &self.clock
    }
}
```

#### Benefits

- ✅ Establishes common interface for all commands
- ✅ Enables generic utilities that work with any command
- ✅ Makes command responsibilities explicit
- ✅ Easier to add new commands with consistent behavior
- ✅ Better documentation of command contracts

#### Implementation Checklist

- [ ] Create `src/application/commands/common/mod.rs`
- [ ] Define `Command` trait
- [ ] Define `CommandExt` trait
- [ ] Implement `Command` for `ProvisionCommand`
- [ ] Implement `CommandExt` for `ProvisionCommand`
- [ ] Implement `Command` for `ConfigureCommand`
- [ ] Implement `CommandExt` for `ConfigureCommand`
- [ ] Update command documentation to reference traits
- [ ] Verify tests still pass
- [ ] Run linter and fix any issues

#### Testing Strategy

- Verify trait implementations compile
- Test that trait methods work correctly
- Ensure existing command tests still pass

---

### Proposal #6: ~~Typed Step Tracking Result~~ (DEFERRED)

**Status**: ⏸️ Deferred  
**Reason**: Will reevaluate after implementing other refactorings. The current tuple pattern works, and we should see if other refactorings make this unnecessary or clarify the best approach.

~~#### Problem

Both commands use a tuple pattern `(Error, Step)` to track which step failed:

```rust
// Current pattern - not self-documenting
fn execute_provisioning_with_tracking(
    &self,
    environment: &Environment<Provisioning>,
) -> Result<(Environment<Provisioned>, IpAddr), (ProvisionCommandError, ProvisionStep)> {
    // ...
}
```

Problems:

- Tuples don't convey intent
- Easy to mix up error and step
- No compile-time guarantee about which step maps to which error

#### Proposed Solution

Create a dedicated result type for step tracking:

```rust
// In src/application/commands/common/step_result.rs

/// Result type that tracks which step was executing when an error occurred
pub struct StepResult<T, E, S> {
    inner: Result<T, StepError<E, S>>,
}

/// Error that includes step information
pub struct StepError<E, S> {
    pub error: E,
    pub failed_step: S,
}

impl<T, E, S> StepResult<T, E, S> {
    /// Create a successful result
    pub fn ok(value: T) -> Self {
        Self {
            inner: Ok(value),
        }
    }

    /// Create an error result with step information
    pub fn err(error: E, step: S) -> Self {
        Self {
            inner: Err(StepError { error, failed_step: step }),
        }
    }

    /// Unwrap the result, separating success and error cases
    pub fn into_result(self) -> Result<T, (E, S)> {
        self.inner.map_err(|e| (e.error, e.failed_step))
    }
}

// Helper trait for converting Result<T, E> to StepResult
pub trait IntoStepResult<T, E, S> {
    fn with_step(self, step: S) -> StepResult<T, E, S>;
}

impl<T, E, S> IntoStepResult<T, E, S> for Result<T, E> {
    fn with_step(self, step: S) -> StepResult<T, E, S> {
        match self {
            Ok(value) => StepResult::ok(value),
            Err(error) => StepResult::err(error, step),
        }
    }
}
```

**Usage example:**

```rust
// Before
fn execute_provisioning_with_tracking(
    &self,
    environment: &Environment<Provisioning>,
) -> Result<(Environment<Provisioned>, IpAddr), (ProvisionCommandError, ProvisionStep)> {
    let current_step = ProvisionStep::RenderOpenTofuTemplates;
    self.render_opentofu_templates()
        .await
        .map_err(|e| (e, current_step))?;
    // ...
}

// After
fn execute_provisioning_with_tracking(
    &self,
    environment: &Environment<Provisioning>,
) -> StepResult<(Environment<Provisioned>, IpAddr), ProvisionCommandError, ProvisionStep> {
    self.render_opentofu_templates()
        .await
        .with_step(ProvisionStep::RenderOpenTofuTemplates)?;
    // ...
}
```

#### Benefits

- ✅ Self-documenting code
- ✅ Type-safe step tracking
- ✅ Cleaner error propagation
- ✅ Less boilerplate in step execution
- ✅ Easier to add new steps

#### Implementation Checklist

- [ ] Create `src/application/commands/common/step_result.rs`
- [ ] Define `StepResult` and `StepError` types
- [ ] Implement `IntoStepResult` trait
- [ ] Update `ProvisionCommand` to use `StepResult`
- [ ] Update `ConfigureCommand` to use `StepResult`
- [ ] Update error handling to unwrap `StepResult`
- [ ] Verify tests still pass
- [ ] Run linter and fix any issues
- [ ] Update documentation

#### Testing Strategy

- Unit tests for `StepResult` type
- Verify error + step information is preserved
- Ensure existing command tests work with new type

---

## Phase 3: Postponed Proposals (Advanced Patterns)

**Status**: ⏸️ Postponed until 1-2 more commands are implemented

These proposals require more command examples to validate the right abstractions. They will be reconsidered after implementing additional commands.

### Proposal #5: Command Trait

**Status**: ⏸️ Postponed  
**Impact**: 🟡🟡 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: Deferred  
**Estimated Time**: 4 hours  
**Reason**: Will reconsider after implementing a third command to validate the abstraction

This proposal would extract a common `Command` trait to establish a consistent interface for all commands. With only two commands currently implemented, it's premature to abstract a common interface. The pattern will be clearer once we have 3+ command examples.

See the full original proposal in the git history for implementation details when ready to revisit.

---

### Proposal #6: Typed Step Tracking

**Status**: ⏸️ Postponed  
**Impact**: 🟡 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: Deferred  
**Estimated Time**: 3 hours  
**Reason**: Will reevaluate after implementing other refactorings to see if still needed

This proposal would introduce a `StepResult<T, E, S>` type to replace the current tuple pattern `(Error, Step)` for tracking which step failed. After implementing Proposals #1-3, we should reassess whether this added complexity is still necessary or if the simpler patterns address the concerns.

See the full original proposal in the git history for implementation details when ready to revisit.

---

### Proposal #7: Template Method Pattern for Command Execution

**Status**: ⏸️ Postponed  
**Impact**: 🟡🟡 Medium  
**Effort**: 🔵🔵🔵 Medium-High  
**Priority**: Deferred  
**Estimated Time**: 6 hours  
**Depends On**: Proposal #1  
**Reason**: Needs 1-2 more command examples to identify the right abstraction

#### Problem

Both commands follow the same execution pattern:

1. Capture start time
2. Transition to "in-progress" state
3. Persist intermediate state
4. Execute steps with tracking
5. On success: transition to success state, persist, return
6. On failure: build context, transition to failed state, persist, return error

This pattern is duplicated across commands with only the specific steps differing.

#### Proposed Solution

Implement the Template Method pattern using a trait with default implementations:

```rust
// In src/application/commands/common/execution_template.rs

use std::sync::Arc;
use crate::domain::environment::{Environment, repository::EnvironmentRepository};
use crate::shared::Clock;

/// Template for command execution with common lifecycle handling
pub trait CommandExecutionTemplate {
    type InputState;
    type InProgressState;
    type OutputState;
    type FailedState;
    type Error;
    type FailureContext;
    type Output;

    /// Get command dependencies
    fn clock(&self) -> &Arc<dyn Clock>;
    fn repository(&self) -> &Arc<dyn EnvironmentRepository>;

    /// Lifecycle transitions - implemented by specific commands
    fn transition_to_in_progress(&self, input: Environment<Self::InputState>)
        -> Environment<Self::InProgressState>;
    fn transition_to_success(&self, in_progress: Environment<Self::InProgressState>)
        -> Environment<Self::OutputState>;
    fn transition_to_failed(
        &self,
        in_progress: Environment<Self::InProgressState>,
        context: Self::FailureContext,
    ) -> Environment<Self::FailedState>;

    /// Core execution logic - implemented by specific commands
    fn execute_steps(&self, environment: &Environment<Self::InProgressState>)
        -> Result<Self::Output, Self::Error>;

    /// Failure context building - implemented by specific commands
    fn build_failure_context(
        &self,
        environment: &Environment<Self::InProgressState>,
        error: &Self::Error,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> Self::FailureContext;

    /// Template method with default implementation
    fn execute_with_lifecycle(
        &self,
        input: Environment<Self::InputState>,
    ) -> Result<(Environment<Self::OutputState>, Self::Output), Self::Error>
    where
        Environment<Self::InProgressState>: Clone + Into<AnyState>,
        Environment<Self::OutputState>: Clone + Into<AnyState>,
        Environment<Self::FailedState>: Clone + Into<AnyState>,
    {
        let started_at = self.clock().now();

        // Transition to in-progress state
        let in_progress = self.transition_to_in_progress(input);

        // Persist intermediate state
        self.persist_state(&in_progress, "in-progress");

        // Execute command-specific steps
        match self.execute_steps(&in_progress) {
            Ok(output) => {
                // Transition to success state
                let success = self.transition_to_success(in_progress);

                // Persist final state
                self.persist_state(&success, "success");

                Ok((success, output))
            }
            Err(error) => {
                // Build failure context
                let context = self.build_failure_context(&in_progress, &error, started_at);

                // Transition to failed state
                let failed = self.transition_to_failed(in_progress, context);

                // Persist error state
                self.persist_state(&failed, "failed");

                Err(error)
            }
        }
    }

    /// Helper for state persistence (uses Proposal #1)
    fn persist_state<S>(&self, environment: &Environment<S>, state_name: &str)
    where
        Environment<S>: Clone + Into<AnyState>
    {
        // Use StatePersistence trait from Proposal #1
        self.repository().persist_state(environment, state_name);
    }
}
```

**Usage in commands:**

```rust
impl CommandExecutionTemplate for ProvisionCommand {
    type InputState = Created;
    type InProgressState = Provisioning;
    type OutputState = Provisioned;
    type FailedState = ProvisionFailed;
    type Error = ProvisionCommandError;
    type FailureContext = ProvisionFailureContext;
    type Output = IpAddr;

    fn clock(&self) -> &Arc<dyn Clock> { &self.clock }
    fn repository(&self) -> &Arc<dyn EnvironmentRepository> { &self.repository }

    fn transition_to_in_progress(&self, input: Environment<Created>)
        -> Environment<Provisioning>
    {
        input.start_provisioning()
    }

    fn transition_to_success(&self, in_progress: Environment<Provisioning>)
        -> Environment<Provisioned>
    {
        in_progress.provisioned()
    }

    fn transition_to_failed(
        &self,
        in_progress: Environment<Provisioning>,
        context: ProvisionFailureContext,
    ) -> Environment<ProvisionFailed> {
        in_progress.provision_failed(context)
    }

    fn execute_steps(&self, environment: &Environment<Provisioning>)
        -> Result<IpAddr, ProvisionCommandError>
    {
        // Existing step execution logic
        self.execute_provisioning_with_tracking(environment)
            .map(|(_, ip)| ip)
            .map_err(|(e, _)| e)
    }

    fn build_failure_context(
        &self,
        environment: &Environment<Provisioning>,
        error: &ProvisionCommandError,
        started_at: chrono::DateTime<chrono::Utc>,
    ) -> ProvisionFailureContext {
        // Existing failure context building logic
        // (simplified from current implementation)
    }
}

// Then the public execute method becomes:
pub async fn execute(
    &self,
    environment: Environment<Created>,
) -> Result<(Environment<Provisioned>, IpAddr), ProvisionCommandError> {
    self.execute_with_lifecycle(environment)
}
```

#### Benefits

- ✅ Eliminates duplicate execution lifecycle code
- ✅ Enforces consistent state management across commands
- ✅ Makes command structure explicit and self-documenting
- ✅ Easier to add new commands with correct lifecycle
- ✅ Single place to modify execution flow

#### Implementation Checklist

- [ ] Create `src/application/commands/common/execution_template.rs`
- [ ] Define `CommandExecutionTemplate` trait
- [ ] Implement template for `ProvisionCommand`
- [ ] Update `ProvisionCommand::execute()` to use template
- [ ] Implement template for `ConfigureCommand`
- [ ] Update `ConfigureCommand::execute()` to use template
- [ ] Verify tests still pass
- [ ] Run linter and fix any issues
- [ ] Update documentation

#### Testing Strategy

- Verify template method works correctly for both commands
- Test state transitions
- Test error handling
- Ensure existing integration tests pass

---

### Proposal #8: Extract Build Failure Context Helper

**Status**: ⏸️ Postponed  
**Impact**: 🟡 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: Deferred  
**Estimated Time**: 3 hours  
**Depends On**: Proposals #2, #3  
**Reason**: Should wait until patterns stabilize across more commands

#### Problem

Both commands have similar `build_failure_context()` methods with common logic:

- Calculate execution duration
- Generate trace ID
- Build base context
- Write trace file
- Update context with trace file path

Only the specific context types and step enums differ.

#### Proposed Solution

Extract common logic into a helper function:

```rust
// In src/application/commands/common/failure_context_builder.rs

use std::sync::Arc;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use crate::domain::environment::{Environment, TraceId};
use crate::domain::environment::state::BaseFailureContext;
use crate::shared::{Clock, ErrorKind, Traceable};

/// Builder for failure contexts with common logic
pub struct FailureContextBuilder<'a, S, E: Traceable> {
    environment: &'a Environment<S>,
    error: &'a E,
    started_at: DateTime<Utc>,
    clock: &'a Arc<dyn Clock>,
}

impl<'a, S, E: Traceable> FailureContextBuilder<'a, S, E>
where
    Environment<S>: HasTracesDir,
{
    pub fn new(
        environment: &'a Environment<S>,
        error: &'a E,
        started_at: DateTime<Utc>,
        clock: &'a Arc<dyn Clock>,
    ) -> Self {
        Self {
            environment,
            error,
            started_at,
            clock,
        }
    }

    /// Build the base failure context with timing and error information
    pub fn build_base_context(&self) -> BaseFailureContext {
        let now = self.clock.now();
        let trace_id = TraceId::new();

        let execution_duration = now
            .signed_duration_since(self.started_at)
            .to_std()
            .unwrap_or_default();

        BaseFailureContext {
            error_summary: self.error.to_string(),
            failed_at: now,
            execution_started_at: self.started_at,
            execution_duration,
            trace_id,
            trace_file_path: None,
        }
    }

    /// Get error kind from the traceable error
    pub fn error_kind(&self) -> ErrorKind {
        self.error.error_kind()
    }

    /// Get traces directory for writing trace files
    pub fn traces_dir(&self) -> PathBuf {
        self.environment.traces_dir()
    }
}

/// Trait for environments that have a traces directory
pub trait HasTracesDir {
    fn traces_dir(&self) -> PathBuf;
}

// Implement for all environment states (they all have this method)
impl<S> HasTracesDir for Environment<S> {
    fn traces_dir(&self) -> PathBuf {
        // Delegate to the actual environment method
        self.traces_dir()
    }
}
```

**Usage in commands:**

```rust
// Before
fn build_failure_context(
    &self,
    environment: &Environment<Provisioning>,
    error: &ProvisionCommandError,
    current_step: ProvisionStep,
    started_at: chrono::DateTime<chrono::Utc>,
) -> ProvisionFailureContext {
    let failed_step = current_step;
    let error_kind = error.error_kind();
    let now = self.clock.now();
    let trace_id = TraceId::new();
    let execution_duration = now
        .signed_duration_since(started_at)
        .to_std()
        .unwrap_or_default();

    let mut context = ProvisionFailureContext {
        failed_step,
        error_kind,
        base: BaseFailureContext {
            error_summary: error.to_string(),
            failed_at: now,
            execution_started_at: started_at,
            execution_duration,
            trace_id,
            trace_file_path: None,
        },
    };

    // ... trace writing logic ...
    context
}

// After
fn build_failure_context(
    &self,
    environment: &Environment<Provisioning>,
    error: &ProvisionCommandError,
    current_step: ProvisionStep,
    started_at: chrono::DateTime<chrono::Utc>,
) -> ProvisionFailureContext {
    let builder = FailureContextBuilder::new(environment, error, started_at, &self.clock);

    let mut context = ProvisionFailureContext {
        failed_step: current_step,
        error_kind: builder.error_kind(),
        base: builder.build_base_context(),
    };

    // Write trace using helper from Proposal #3
    let trace_writer = ProvisionTraceWriter::new(
        builder.traces_dir(),
        Arc::clone(&self.clock),
    );
    write_trace_with_logging(&trace_writer, &mut context, error, "provision");

    context
}
```

#### Benefits

- ✅ Reduces duplication in failure context building
- ✅ Consistent timing and trace ID generation
- ✅ Easier to add new failure context fields
- ✅ Better separation of concerns

#### Implementation Checklist

- [ ] Create `src/application/commands/common/failure_context_builder.rs`
- [ ] Define `FailureContextBuilder` struct
- [ ] Implement base context building logic
- [ ] Define `HasTracesDir` trait
- [ ] Update `ProvisionCommand::build_failure_context()` to use builder
- [ ] Update `ConfigureCommand::build_failure_context()` to use builder
- [ ] Verify tests still pass
- [ ] Run linter and fix any issues

#### Testing Strategy

- Unit tests for failure context builder
- Verify timing calculations are correct
- Test trace ID generation
- Ensure existing command tests pass

---

## Phase 4: Additional Test Improvements (Optional)

### Proposal #9: Standardize Test Environment Creation

**Status**: ⏳ Not Started  
**Impact**: 🟡 Medium  
**Effort**: 🔵🔵 Medium  
**Priority**: P3 (Optional - can be done alongside Phase 1-2)  
**Estimated Time**: 2 hours

#### Problem

Both command test files have similar `create_test_environment()` helper functions with slight variations:

```rust
// In provision.rs and configure.rs tests
fn create_test_environment(_temp_dir: &TempDir) -> (Environment<SomeState>, TempDir) {
    use crate::domain::environment::testing::EnvironmentTestBuilder;

    let (env, _data_dir, _build_dir, env_temp_dir) = EnvironmentTestBuilder::new()
        .with_name("test-env")
        .build_with_custom_paths();

    // Transition to desired state...
    (env.start_provisioning(), env_temp_dir)
}
```

This creates confusion:

- Takes a `_temp_dir` parameter that's never used
- Returns a `TempDir` that must be kept alive
- State transitions are duplicated

#### Proposed Solution

Create a standardized test environment builder specifically for command tests:

```rust
// In src/domain/environment/testing.rs (add to existing module)

impl EnvironmentTestBuilder {
    /// Create an environment in Provisioning state for command tests
    pub fn build_provisioning() -> (Environment<Provisioning>, TempDir) {
        let (env, _, _, temp_dir) = Self::new()
            .with_name("test-env")
            .build_with_custom_paths();

        (env.start_provisioning(), temp_dir)
    }

    /// Create an environment in Configuring state for command tests
    pub fn build_configuring() -> (Environment<Configuring>, TempDir) {
        let (env, _, _, temp_dir) = Self::new()
            .with_name("test-env")
            .build_with_custom_paths();

        (
            env.start_provisioning().provisioned().start_configuring(),
            temp_dir,
        )
    }

    /// Create an environment in Provisioned state for command tests
    pub fn build_provisioned() -> (Environment<Provisioned>, TempDir) {
        let (env, _, _, temp_dir) = Self::new()
            .with_name("test-env")
            .build_with_custom_paths();

        (env.start_provisioning().provisioned(), temp_dir)
    }
}
```

**Usage in tests:**

```rust
// Before
fn create_test_environment(_temp_dir: &TempDir) -> (Environment<Provisioning>, TempDir) {
    use crate::domain::environment::testing::EnvironmentTestBuilder;

    let (env, _data_dir, _build_dir, env_temp_dir) = EnvironmentTestBuilder::new()
        .with_name("test-env")
        .build_with_custom_paths();

    (env.start_provisioning(), env_temp_dir)
}

#[test]
fn it_should_build_failure_context_from_opentofu_template_error() {
    let (_, _, _, _, _, _, _, temp_dir) = create_mock_dependencies();
    let (environment, _env_temp_dir) = create_test_environment(&temp_dir);
    // ...
}

// After
#[test]
fn it_should_build_failure_context_from_opentofu_template_error() {
    let (environment, _env_temp_dir) = EnvironmentTestBuilder::build_provisioning();
    // ...
}
```

#### Benefits

- ✅ Removes confusing unused parameters
- ✅ Centralizes test environment creation logic
- ✅ Consistent environment creation across all tests
- ✅ Clearer test code with less boilerplate
- ✅ Easier to add new environment states for testing

#### Implementation Checklist

- [ ] Add `build_provisioning()` to `EnvironmentTestBuilder`
- [ ] Add `build_configuring()` to `EnvironmentTestBuilder`
- [ ] Add `build_provisioned()` to `EnvironmentTestBuilder`
- [ ] Update all `provision.rs` tests to use new methods
- [ ] Update all `configure.rs` tests to use new methods
- [ ] Remove old `create_test_environment()` helper functions
- [ ] Verify all tests pass
- [ ] Run linter and fix any issues

#### Testing Strategy

- Verify all command tests work with new builders
- Ensure TempDir cleanup works correctly
- Test that environments are in correct states

---

## 📅 Implementation Timeline

### Sprint 0 (Day 1): API Simplification - MUST DO FIRST

- **Day 1 Morning**: Proposal #0 - Add instance_ip to Environment Context
- **Day 1 Afternoon**: Update all call sites, tests, verify everything works

**Deliverables**: Simplified command API, all tests passing, foundation ready for other refactorings

**Critical**: This must be completed first as it simplifies the API that all other proposals will work with.

---

### Sprint 1 (Week 1): Quick Wins

- **Day 1-2**: Proposal #1 - State Persistence Helper
- **Day 2**: Proposal #2 - Use Injected Clock
- **Day 3-4**: Proposal #3 - Trace Writing Pattern
- **Day 5**: Testing, documentation, code review

**Deliverables**: 3 quick wins implemented, tests passing, documentation updated

### Sprint 2 (Week 2): Test Improvements

- **Day 1-3**: Proposal #4 - Test Builder Helpers
- **Day 4**: Proposal #9 - Standardize Test Environments (Optional)
- **Day 5**: Testing, code review, final documentation

**Deliverables**: All active proposals complete, test code simplified and standardized

### Future Sprints: Postponed Proposals

After implementing 1-2 more commands, revisit:

- Proposal #5 - Command Trait (validate abstraction with 3+ commands)
- Proposal #6 - Typed Step Tracking (reassess if needed)
- Proposal #7 - Template Method Pattern (identify right abstraction)
- Proposal #8 - Failure Context Helper (extract common patterns)

## ✅ Definition of Done

For each proposal:

- [ ] Code changes implemented and committed
- [ ] All existing tests pass
- [ ] New tests added where appropriate
- [ ] Linter runs without errors (`cargo run --bin linter all`)
- [ ] No unused dependencies (`cargo machete`)
- [ ] Documentation updated (code comments, module docs)
- [ ] Code reviewed and approved
- [ ] Changes merged to main branch

For the entire refactoring (Phase 0-2):

- [ ] All 5 active proposals completed (1 discarded, 4 postponed based on feedback)
- [ ] API simplification (#0) completed FIRST - prerequisite for all others
- [ ] Quick wins (#1, #2, #3) implemented and tested
- [ ] Test improvements (#4, optionally #9) implemented
- [ ] Test coverage maintained or improved
- [ ] No performance regressions
- [ ] Documentation complete and accurate
- [ ] Team retrospective conducted
- [ ] Postponed proposals documented for future consideration
- [ ] Refactoring plan marked as ✅ Completed (Phase 1-2)

## 🔍 Review Checklist

Before starting implementation:

- [ ] Proposals prioritized correctly?
- [ ] Dependencies between proposals clear?
- [ ] Timeline realistic for team capacity?
- [ ] Alignment with development principles?
- [ ] Test strategy adequate?
- [ ] Risk mitigation considered?

During implementation:

- [ ] Tests pass after each proposal?
- [ ] Code quality maintained?
- [ ] Documentation kept up to date?
- [ ] Team alignment maintained?
- [ ] Issues documented and resolved?

After completion:

- [ ] All acceptance criteria met?
- [ ] Code quality improved measurably?
- [ ] Team satisfied with changes?
- [ ] Lessons learned documented?
- [ ] Plan archived properly?

## 📝 Notes

### Risk Mitigation

- **Breaking Changes**: Use deprecation warnings for old APIs before removal
- **Test Coverage**: Run full test suite after each proposal
- **Performance**: Profile commands before/after for any regressions
- **Team Alignment**: Daily check-ins during implementation sprints

### Future Considerations

After this refactoring, consider:

- Dependency injection container for simpler command construction
- Command pipeline pattern for chaining multiple commands
- Event sourcing for command history and replay
- Command middleware for cross-cutting concerns (logging, metrics)

### Related Documentation

- [Development Principles](../development-principles.md)
- [Testing Conventions](../contributing/testing.md)
- [Error Handling Guide](../contributing/error-handling.md)
- [Codebase Architecture](../codebase-architecture.md)

---

**Created**: October 7, 2025
**Status**: 📋 Planning
**Last Updated**: October 7, 2025
