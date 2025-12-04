# Phase 5: Command Integration - Implementation Plan ✅ COMPLETED

> **📋 Detailed Plan**  
> Breaking down Phase 5 into manageable, testable subtasks for integrating type-safe state management into commands.

## 🎯 Phase 5 Overview

**Status**: ✅ **COMPLETED** - All 4 subtasks complete (October 6, 2025)

**Goal**: Update commands to use type-safe state transitions, persist state during execution, and enable orchestration with compile-time state guarantees.

**Why We Need This**: Commands currently operate on stateless `Environment` objects. By integrating the type-state pattern and persistence layer, we enable:

- **State Visibility**: Track command progress and detect interrupted operations
- **Error Recovery Guidance**: Know exactly which step failed and provide actionable advice
- **Orchestration Safety**: Chain commands with compile-time guarantees of valid state flows
- **Observability**: Automatic state persistence creates audit trail of every transition

**Final Test Count**: 704 tests passing ✅

---

## 🎯 Phase 5 Summary

### Deliverables Checklist

- ✅ `ProvisionCommand` accepts `Environment<Created>`, returns `Environment<Provisioned>`
- ✅ `ConfigureCommand` accepts `Environment<Provisioned>`, returns `Environment<Configured>`
- ✅ Commands transition through intermediate states (`Provisioning`, `Configuring`)
- ✅ Commands transition to error states on failure with step context
- ✅ State persisted after each transition via repository
- ✅ Persistence failures handled gracefully (don't break commands)
- ✅ Compile-time prevention of invalid state transitions
- ✅ Type-safe state transitions demonstrated in E2E tests
- ✅ State transitions visible in logs during E2E tests and persistent audit trail

**Approach**: Incrementally update commands to accept and return typed states, integrate persistence at command boundaries, and build an orchestration layer that leverages compile-time state validation.

## 📋 Implementation Subtasks

### Subtask 1: Preparatory Refactoring ✅ COMPLETED

**Status**: ✅ Completed (Commit: ff80d7d)

**Purpose**: Clean up and restructure commands to make Phase 5 implementation easier and safer.

**Why This First**: Before integrating state management (which requires significant changes to command signatures), we should:

- Extract logical phases from monolithic `execute()` methods
- Remove redundant dependencies that will be replaced by `Environment<S>`
- Add helper methods for error context extraction
- Create clear structure with obvious injection points

**See**: [Refactoring Plan](../../../refactors/command-preparation-for-state-management.md)

**Tasks**:

1. **Refactor ProvisionCommand Structure** - Extract 5 logical phases from execute() method
2. **Remove Redundant SshCredentials Field** - Pass as parameter instead of storing in struct
3. **Add Error Context Extraction Helpers** - Prepare for error state tracking
4. **Add Persistence Injection Point Markers** - Document where state transitions will go
5. **Setup Repository Infrastructure in Container** - Instantiate repository, prepare for injection (non-breaking since commands keep old signatures for now)

**Benefits**:

- Smaller, focused commits before major changes
- Reduced complexity of Phase 5 implementation
- Clear separation between refactoring and feature work
- Lower risk of bugs during state integration

**Tests**:

- All existing tests continue to pass (no behavior changes)
- 2 new tests for error context extraction helpers
- 5 new integration tests for repository infrastructure:
  1. **Repository Instantiation**: Verify repository is correctly instantiated with 30-second timeout
  2. **State File Paths**: Verify state files follow `build/{env_name}/state.json` convention
  3. **File Locking**: Verify locks prevent concurrent modifications to same environment
  4. **Concurrent Execution**: Verify different environments can be modified concurrently
  5. **Lock Cleanup**: Verify orphaned locks (dead process) are automatically cleaned up

**Estimated Impact**:

- ProvisionCommand: ~180 LOC → ~220 LOC (+40 for extracted methods)
- ConfigureCommand: ~100 LOC → ~110 LOC (+10 for helpers)
- Test count: 663 → 670 tests (+2 unit + 5 integration)

**Commit Messages**:

```text
refactor: extract logical phases in ProvisionCommand for better clarity
refactor: pass SshCredentials as parameter instead of storing in ProvisionCommand
refactor: add error context extraction helpers for future state tracking
docs: add Phase 5 injection point markers in command handlers
refactor: setup repository infrastructure in Container
test: add integration tests for repository infrastructure and file locking
```

**Container Setup Details** (Task 5):

Add repository to container WITHOUT changing command signatures yet:

```rust
pub struct Container {
    // ... existing fields ...
    environment_repository: Arc<dyn EnvironmentRepository>,
}

impl Container {
    pub fn new() -> Result<Self, ContainerError> {
        // ... existing setup ...

        // Create repository with 30-second lock timeout
        let repository = Arc::new(JsonFileRepository::new(
            Duration::from_secs(30),
        )) as Arc<dyn EnvironmentRepository>;

        Ok(Self {
            // ... existing fields ...
            environment_repository: repository,
        })
    }

    // Keep existing factory methods unchanged for now
    // They will be updated in Subtasks 3-4 when command signatures change
}
```

**Success Criteria**:

- ✅ All refactoring tasks completed
- ✅ Container instantiates repository (not yet injected into commands)
- ✅ Repository integration tests pass (5 tests covering locking, paths, cleanup)
- ✅ All linters pass (no new warnings)
- ✅ All tests pass (no behavior changes)
- ✅ Code complexity reduced (main execute() methods < 30 lines)
- ✅ Clear structure for Phase 5 integration

---

### Subtask 2: Document Architectural Decision (ADR) ✅ COMPLETED

**Status**: ✅ Completed (Commit: 7f02019)

**Purpose**: Document the architectural decision to use typed state returns in commands instead of the pure command handler pattern.

**Changes**:

Create an Architectural Decision Record (ADR) at `docs/decisions/command-state-return-pattern.md` documenting:

1. **Context**: Why this decision is needed

   - Commands need to integrate with type-state pattern
   - Two main approaches: pure command handler vs typed returns
   - Need to decide which pattern to use

2. **Decision**: Use typed state returns (commands accept and return `Environment<S>`)

3. **Rationale**: Why typed returns are better for this use case

   - Leverages compile-time type safety from type-state pattern (4 phases of work)
   - Avoids repeated loading/pattern matching on `AnyEnvironmentState`
   - Enables fluent command chaining in the future
   - Commands are state transformations, not just side effects
   - Repository is for persistence, not primary data flow
   - Type safety benefits outweigh CQS purism

4. **Alternatives Considered**:

   - **Pure Command Handler**: Commands load/save via repository, no returns
     - **Pros**: Follows traditional Command/Query Separation (CQS), well-known CQRS pattern
     - **Cons**: Pattern matching overhead, loses compile-time safety, repository-centric
   - **Hybrid Approach**: Store environment in command struct with interior mutability
     - **Cons**: Borrowing complexity, unclear ownership

5. **Consequences**:

   - **Positive**:
     - Compile-time prevention of invalid state transitions
     - Clear data flow: input → transform → output
     - No repeated parsing of `AnyEnvironmentState`
     - Future orchestration layer possible
     - Type-state pattern reaches its full potential
   - **Negative**:
     - Deviates from pure command handler pattern
     - Commands return values (not traditional CQS)
   - **Neutral**:
     - Repository is secondary concern (persistence only)

6. **References**:
   - Type-State Pattern: https://cliffle.com/blog/rust-typestate/
   - CQRS flexibility: Commands can return acknowledgments
   - Phase 1-4 implementation of type-state pattern

**Success Criteria**:

- ✅ ADR document created at `docs/decisions/command-state-return-pattern.md`
- ✅ ADR follows project template structure
- ✅ Decision clearly explained with concrete examples
- ✅ Alternatives documented with fair assessment
- ✅ Consequences (positive, negative, neutral) listed
- ✅ ADR added to decisions index in `docs/decisions/README.md`

**Commit**: `docs: add ADR for command state return pattern decision`

**Estimated Test Count**: 663 tests (no new tests, documentation only)

---

### Subtask 3: Update ProvisionCommand for Type-Safe States

**Purpose**: Modify `ProvisionCommand` to accept `Environment<Created>` and return `Environment<Provisioned>`, with intermediate state transitions and persistence. **Immediately update all code that uses this command** (Container factory, E2E binaries) since this is a breaking change.

**Changes**:

1. **Update Command Signature**:

   - Change `execute()` to accept `Environment<Created>` instead of raw parameters
   - Return `Environment<Provisioned>` instead of just `IpAddr`
   - Store `Environment` as a field in the command struct

2. **Integrate State Transitions**:

   - Transition to `Provisioning` state at command start
   - Persist state after each major step (OpenTofu init, apply, etc.)
   - Transition to `Provisioned` state on success
   - Transition to `ProvisionFailed` state on error with failed step name

3. **Add Persistence Integration**:

   - Inject `Arc<dyn EnvironmentRepository>` dependency
   - Save state after transitioning to `Provisioning`
   - Save state after transitioning to `Provisioned` or `ProvisionFailed`
   - Handle persistence errors gracefully (log but don't fail command)

4. **Preserve Backward Compatibility**:
   - Keep existing E2E tests passing
   - Maintain current logging and error handling behavior
   - Ensure IP address is still returned (via `Environment<Provisioned>` getter)

**Implementation Details**:

```rust
use crate::domain::environment::{Environment, Created, Provisioning, Provisioned, ProvisionFailed};
use crate::domain::environment::repository::EnvironmentRepository;

pub struct ProvisionCommand {
    tofu_template_renderer: Arc<TofuTemplateRenderer>,
    ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
    ansible_client: Arc<AnsibleClient>,
    opentofu_client: Arc<OpenTofuClient>,
    repository: Arc<dyn EnvironmentRepository>,
}

impl ProvisionCommand {
    pub fn new(
        tofu_template_renderer: Arc<TofuTemplateRenderer>,
        ansible_template_renderer: Arc<AnsibleTemplateRenderer>,
        ansible_client: Arc<AnsibleClient>,
        opentofu_client: Arc<OpenTofuClient>,
        repository: Arc<dyn EnvironmentRepository>,
    ) -> Self {
        Self {
            tofu_template_renderer,
            ansible_template_renderer,
            ansible_client,
            opentofu_client,
            repository,
        }
    }

    #[instrument(
        name = "provision_command",
        skip_all,
        fields(
            command_type = "provision",
            environment = %environment.name()
        )
    )]
    pub async fn execute(
        &self,
        environment: Environment<Created>,
    ) -> Result<Environment<Provisioned>, ProvisionCommandError> {
        info!(
            command = "provision",
            environment = %environment.name(),
            "Starting infrastructure provisioning workflow"
        );

        // Transition to Provisioning state
        let environment = environment.start_provisioning();

        // Persist intermediate state
        if let Err(e) = self.persist_state(&environment) {
            warn!(
                "Failed to persist provisioning state: {}. Continuing with command execution.",
                e
            );
        }

        // Execute provisioning steps...
        match self.execute_provisioning_steps(&environment).await {
            Ok(provisioned_env) => {
                // Persist final state
                if let Err(e) = self.persist_state(&provisioned_env) {
                    warn!(
                        "Failed to persist provisioned state: {}. Instance is provisioned but state not saved.",
                        e
                    );
                }
                Ok(provisioned_env)
            }
            Err(e) => {
                // Transition to error state with step context
                let failed_env = environment.fail_provisioning(
                    self.extract_failed_step(&e)
                );

                // Persist error state
                if let Err(persist_err) = self.persist_state(&failed_env) {
                    warn!(
                        "Failed to persist provision failure state: {}",
                        persist_err
                    );
                }

                Err(e)
            }
        }
    }

    async fn execute_provisioning_steps(
        &self,
        environment: &Environment<Provisioning>,
    ) -> Result<Environment<Provisioned>, ProvisionCommandError> {
        // Render OpenTofu templates
        RenderOpenTofuTemplatesStep::new(Arc::clone(&self.tofu_template_renderer))
            .execute()
            .await?;

        // Create instance via OpenTofu
        self.create_instance()?;

        // Get instance info
        let instance_info = GetInstanceInfoStep::new(Arc::clone(&self.opentofu_client))
            .execute()?;
        let instance_ip = instance_info.ip_address;

        // Render Ansible templates with runtime IP
        RenderAnsibleTemplatesStep::new(
            Arc::clone(&self.ansible_template_renderer),
            instance_ip,
        )
        .execute()?;

        // Wait for connectivity
        WaitForSSHConnectivityStep::new(
            instance_ip,
            &environment.ssh_credentials(),
        )
        .execute()
        .await?;

        WaitForCloudInitStep::new(
            Arc::clone(&self.ansible_client),
        )
        .execute()?;

        info!(
            command = "provision",
            environment = %environment.name(),
            ip = %instance_ip,
            "Infrastructure provisioning completed successfully"
        );

        // Transition to Provisioned state
        Ok(environment.clone().complete_provisioning(instance_ip))
    }

    fn persist_state<S: Serialize>(
        &self,
        environment: &Environment<S>,
    ) -> Result<(), RepositoryError> {
        let any_state = environment.clone().into_any();
        self.repository.save(&any_state)
    }

    fn extract_failed_step(&self, error: &ProvisionCommandError) -> String {
        // Extract step information from error for context
        match error {
            ProvisionCommandError::OpenTofuTemplateRendering(_) => {
                "render_opentofu_templates".to_string()
            }
            ProvisionCommandError::OpenTofu(e) => {
                format!("opentofu_{}", self.extract_opentofu_step(e))
            }
            ProvisionCommandError::AnsibleTemplateRendering(_) => {
                "render_ansible_templates".to_string()
            }
            ProvisionCommandError::SshConnectivity(_) => {
                "wait_ssh_connectivity".to_string()
            }
            ProvisionCommandError::Command(_) => {
                "cloud_init_wait".to_string()
            }
        }
    }

    fn extract_opentofu_step(&self, _error: &OpenTofuError) -> String {
        // Could be more sophisticated based on error variant
        "operation".to_string()
    }
}
```

**Tests to Add**:

- Test command accepts `Environment<Created>` and returns `Environment<Provisioned>`
- Test state transition to `Provisioning` at command start
- Test state transition to `Provisioned` on success
- Test state transition to `ProvisionFailed` on error with step name
- Test persistence is called after each state transition
- Test command continues execution if persistence fails (with warning log)
- Test IP address is accessible from returned `Environment<Provisioned>`
- Mock repository to verify state persistence calls

**Integration Updates (Must be done together with command changes)**:

1. **Update Container factory method**:

   ```rust
   pub fn provision_command(&self) -> ProvisionCommand {
       ProvisionCommand::new(
           Arc::clone(&self.tofu_template_renderer),
           Arc::clone(&self.ansible_template_renderer),
           Arc::clone(&self.ansible_client),
           Arc::clone(&self.opentofu_client),
           Arc::clone(&self.environment_repository),  // Inject repository
       )
   }
   ```

2. **Update E2E Provision and Destroy Test binary** (`src/bin/e2e-provision-and-destroy-tests.rs`):

   ```rust
   async fn run_provision_test() -> Result<(), Box<dyn std::error::Error>> {
       let container = Container::new()?;

       // Create environment in Created state
       let env_name = EnvironmentName::new("e2e-provision".to_string())?;
       let ssh_credentials = load_test_ssh_credentials()?;
       let environment = Environment::new(env_name, ssh_credentials);

       // Execute with typed state
       let provision_cmd = container.provision_command();
       let provisioned_env = provision_cmd.execute(environment).await?;

       // Extract IP from typed environment
       let ip_address = provisioned_env.ip_address()
           .expect("Provisioned environment must have IP");

       info!(ip = %ip_address, "Provisioning completed");
       Ok(())
   }
   ```

3. **Update E2E Full Test binary** (`src/bin/e2e-tests-full.rs`) - Provision part:

   ```rust
   async fn run_full_deployment() -> Result<(), Box<dyn std::error::Error>> {
       let container = Container::new()?;

       // Create initial environment
       let env_name = EnvironmentName::new("e2e-full".to_string())?;
       let ssh_credentials = load_test_ssh_credentials()?;
       let environment = Environment::new(env_name, ssh_credentials);

       // Provision with typed state
       let provision_cmd = container.provision_command();
       let provisioned_env = provision_cmd.execute(environment).await?;

       // TODO: Configure step will be updated in Subtask 4
       // For now, just verify provisioning worked
       info!(ip = %provisioned_env.ip_address().unwrap(), "Provisioned");

       Ok(())
   }
   ```

**Tests to Update**:

- Update E2E provision test binary to create `Environment<Created>`
- Update E2E full test binary (provision part) to use typed states
- Verify existing tests still pass with new signature

**Success Criteria**:

- ✅ `ProvisionCommand` signature uses typed states
- ✅ State transitions at command boundaries
- ✅ State persisted after transitions
- ✅ Error states include failed step information
- ✅ Persistence failures don't break command execution
- ✅ All existing E2E tests pass
- ✅ All unit tests pass
- ✅ All linters pass

**Commit Messages**:

```text
feat: integrate type-safe state management into ProvisionCommand
refactor: update Container factory method to inject repository into ProvisionCommand
refactor: update E2E provision test to use typed Environment states
refactor: update E2E full test (provision part) to use typed states
```

**Estimated Test Count**: 670+ tests (7 new unit tests)

---

### Subtask 4: Update ConfigureCommand for Type-Safe States ✅ COMPLETED

**Status**: ✅ Completed (Commit: 8997275)

**Purpose**: Modify `ConfigureCommand` to accept `Environment<Provisioned>` and return `Environment<Configured>`, with state transitions and persistence. **Immediately update all code that uses this command** (Container factory, E2E binaries) since this is a breaking change.

**Changes**:

1. **Update Command Signature**:

   - Change `execute()` to accept `Environment<Provisioned>` instead of depending on external state
   - Return `Environment<Configured>` on success
   - Store `Environment` as a field in the command struct

2. **Integrate State Transitions**:

   - Transition to `Configuring` state at command start
   - Persist state after transition
   - Transition to `Configured` state on success
   - Transition to `ConfigureFailed` state on error with failed step name

3. **Add Persistence Integration**:

   - Inject `Arc<dyn EnvironmentRepository>` dependency
   - Save state after each transition
   - Handle persistence errors gracefully

4. **Extract IP Address**:
   - Access IP address from `Environment<Provisioned>` state
   - Use IP for Ansible operations

**Implementation Details**:

```rust
use crate::domain::environment::{Environment, Provisioned, Configuring, Configured, ConfigureFailed};
use crate::domain::environment::repository::EnvironmentRepository;

pub struct ConfigureCommand {
    ansible_client: Arc<AnsibleClient>,
    repository: Arc<dyn EnvironmentRepository>,
}

impl ConfigureCommand {
    pub fn new(
        ansible_client: Arc<AnsibleClient>,
        repository: Arc<dyn EnvironmentRepository>,
    ) -> Self {
        Self {
            ansible_client,
            repository,
        }
    }

    #[instrument(
        name = "configure_command",
        skip_all,
        fields(
            command_type = "configure",
            environment = %environment.name()
        )
    )]
    pub fn execute(
        &self,
        environment: Environment<Provisioned>,
    ) -> Result<Environment<Configured>, ConfigureCommandError> {
        info!(
            command = "configure",
            environment = %environment.name(),
            "Starting infrastructure configuration workflow"
        );

        // Transition to Configuring state
        let environment = environment.start_configuring();

        // Persist intermediate state
        if let Err(e) = self.persist_state(&environment) {
            warn!(
                "Failed to persist configuring state: {}. Continuing with command execution.",
                e
            );
        }

        // Execute configuration steps
        match self.execute_configuration_steps(&environment) {
            Ok(configured_env) => {
                // Persist final state
                if let Err(e) = self.persist_state(&configured_env) {
                    warn!(
                        "Failed to persist configured state: {}. Instance is configured but state not saved.",
                        e
                    );
                }
                Ok(configured_env)
            }
            Err(e) => {
                // Transition to error state
                let failed_env = environment.fail_configuring(
                    self.extract_failed_step(&e)
                );

                // Persist error state
                if let Err(persist_err) = self.persist_state(&failed_env) {
                    warn!(
                        "Failed to persist configure failure state: {}",
                        persist_err
                    );
                }

                Err(e)
            }
        }
    }

    fn execute_configuration_steps(
        &self,
        environment: &Environment<Configuring>,
    ) -> Result<Environment<Configured>, ConfigureCommandError> {
        // Install Docker
        InstallDockerStep::new(Arc::clone(&self.ansible_client))
            .execute()?;

        // Install Docker Compose
        InstallDockerComposeStep::new(Arc::clone(&self.ansible_client))
            .execute()?;

        info!(
            command = "configure",
            environment = %environment.name(),
            "Infrastructure configuration completed successfully"
        );

        // Transition to Configured state
        Ok(environment.clone().complete_configuring())
    }

    fn persist_state<S: Serialize>(
        &self,
        environment: &Environment<S>,
    ) -> Result<(), RepositoryError> {
        let any_state = environment.clone().into_any();
        self.repository.save(&any_state)
    }

    fn extract_failed_step(&self, error: &ConfigureCommandError) -> String {
        match error {
            ConfigureCommandError::Command(e) => {
                // Try to extract step from command error
                format!("configuration_step: {}", e)
            }
        }
    }
}
```

**Tests to Add**:

- Test command accepts `Environment<Provisioned>` and returns `Environment<Configured>`
- Test compile-time error when passing wrong state (documentation test)
- Test state transition to `Configuring` at command start
- Test state transition to `Configured` on success
- Test state transition to `ConfigureFailed` on error with step name
- Test persistence is called after each state transition
- Test command continues if persistence fails
- Mock repository to verify persistence calls

**Integration Updates (Must be done together with command changes)**:

1. **Update Container factory method**:

   ```rust
   pub fn configure_command(&self) -> ConfigureCommand {
       ConfigureCommand::new(
           Arc::clone(&self.ansible_client),
           Arc::clone(&self.environment_repository),  // Inject repository
       )
   }
   ```

2. **Update E2E Config and Release Test binary** (`src/bin/e2e-config-and-release-tests.rs`):

   ```rust
   fn run_configure_test() -> Result<(), Box<dyn std::error::Error>> {
       let container = Container::new()?;

       // Load provisioned environment (from previous provision)
       // Or mock a provisioned environment for testing
       let provisioned_env = load_provisioned_environment()?;

       // Execute with typed state
       let configure_cmd = container.configure_command();
       let configured_env = configure_cmd.execute(provisioned_env)?;

       info!(environment = %configured_env.name(), "Configuration completed");
       Ok(())
   }
   ```

3. **Update E2E Full Test binary** (`src/bin/e2e-tests-full.rs`) - Complete the chain:

   ```rust
   async fn run_full_deployment() -> Result<(), Box<dyn std::error::Error>> {
       let container = Container::new()?;

       // Create initial environment
       let env_name = EnvironmentName::new("e2e-full".to_string())?;
       let ssh_credentials = load_test_ssh_credentials()?;
       let environment = Environment::new(env_name, ssh_credentials);

       // Provision
       let provision_cmd = container.provision_command();
       let provisioned_env = provision_cmd.execute(environment).await?;

       // Configure (compile-time enforced: requires Provisioned state!)
       let configure_cmd = container.configure_command();
       let configured_env = configure_cmd.execute(provisioned_env)?;

       info!(
           environment = %configured_env.name(),
           "Full deployment completed successfully"
       );

       Ok(())
   }
   ```

**Tests to Update**:

- Update E2E config test binary to use typed `Environment<Provisioned>`
- Update E2E full test binary (configure part) to chain from provision
- Verify compile-time type safety: cannot pass `Environment<Created>` to configure
- Verify tests pass with new typed interface

**Success Criteria**: ✅ All achieved

- ✅ `ConfigureCommand` signature uses typed states
- ✅ Compile-time enforcement: only accepts `Environment<Provisioned>`
- ✅ State transitions at command boundaries
- ✅ State persisted after transitions
- ✅ Error states include failed step information
- ✅ All existing E2E tests pass
- ✅ All unit tests pass (704 total)
- ✅ All linters pass

**Commits**:

- `8997275` - feat: integrate state management in ConfigureCommand and fix environment persistence
- `c0aff04` - fix: use temp directory in repository_factory test to avoid data/ artifacts

**Final Test Count**: 704 tests passing ✅

---

## 🎯 Phase 5 Summary ✅ COMPLETED

### Deliverables Checklist - All Achieved ✅

- ✅ **ADR documenting command state return pattern** vs pure command handler approach (Commit: 7f02019)
- ✅ `ProvisionCommand` accepts `Environment<Created>`, returns `Environment<Provisioned>` (Commit: 698d85a)
- ✅ `ConfigureCommand` accepts `Environment<Provisioned>`, returns `Environment<Configured>` (Commit: 8997275)
- ✅ Commands transition through intermediate states (`Provisioning`, `Configuring`)
- ✅ Commands transition to error states on failure with step context
- ✅ State persisted after each transition via repository
- ✅ Persistence failures handled gracefully (don't break commands)
- ✅ Compile-time prevention of invalid state transitions
- ✅ Type-safe state transitions demonstrated in E2E tests
- ✅ State transitions visible in logs during E2E tests
- ✅ Environment files correctly persisted at `data/{env}/environment.json`

### Critical Integration Considerations

#### 1. State File Management

**Location and Naming Convention**:

- State files: `build/{environment_name}/state.json`
- Lock files: `build/{environment_name}/state.json.lock`
- Created by repository during first save operation
- Persisted across command executions for same environment

**Handling Missing or Corrupted State**:

- If state file missing: Command proceeds (environment is source of truth)
- If state file corrupted: Log error, proceed with command (or fail-fast, to be decided)
- If lock file orphaned (process died): File locking library handles with PID validation
- Manual state manipulation: Commands can detect mismatch and warn user

**Cleanup Strategy**:

- E2E tests can optionally clean up state files after test completion
- Or leave them for debugging (current approach in E2E tests)
- Future: Add `destroy` command to clean up both infrastructure and state

#### 2. Concurrent Execution and File Locking

**Locking Behavior**:

- File locks are **per-environment** (each environment has its own state file)
- Multiple commands on **different environments** can run concurrently ✅
- Multiple commands on **same environment** are serialized by file lock ✅
- Lock timeout: 30 seconds (configurable)

**User Experience with Locks**:

- If lock acquisition fails: Error message includes PID of lock holder
- User can check if process is running: `ps -p <pid>` (Unix) or `tasklist /FI "PID eq <pid>"` (Windows)
- If process is dead: Lock file is automatically cleaned up on next attempt
- Clear actionable error messages (following error handling guide)

#### 3. Error State Recovery

**Current Behavior (Phase 5)**:

- Commands **do not** validate input state against persisted state
- Type system ensures correct state at compile time
- If provision fails → `Environment<ProvisionFailed>` persisted
- User must handle error state (future: add `resume` or `retry` command)

**Invalid State Scenarios**:

❌ **Cannot call configure on `Environment<Created>`** → Compile error (type safety)  
❌ **Cannot call provision on `Environment<Provisioned>`** → Would create new typed env  
✅ **Can call provision on `Environment<ProvisionFailed>`** → Would replace failed state  
⚠️ **State file says "Provisioned" but user has `Environment<Created>`** → Mismatch warning (optional validation)

**Future Enhancement** (Phase 6+):

- Add state validation: Check stored state matches input state type
- Add `resume` command: Load state, determine next action, continue workflow
- Add `retry` command: Reset error state, retry failed operation

#### 4. Repository Configuration

**Constructor Parameters**:

- Lock timeout: `Duration` (30 seconds default)
- State file base path: Derived from environment name (`build/{env_name}/`)
- Lock file strategy: Same directory as state file, `.lock` extension

**Test vs Production**:

- **Unit tests**: Mock repository (`MockEnvironmentRepository`)
- **E2E tests**: Real file repository (`JsonFileRepository`) with test environment names
- **Production** (future): Same file repository, configured paths

**Example Configuration**:

```rust
// In container or main setup
let repository = Arc::new(JsonFileRepository::new(
    Duration::from_secs(30),  // Lock timeout
)) as Arc<dyn EnvironmentRepository>;
```

#### 5. Logging and Observability

**State Transition Logging** (from Phase 2):

- Every state transition automatically logged with structured fields
- `#[instrument]` spans track command execution
- State name, environment name, timestamp included

**Example Log Output**:

```text
2025-10-03T10:30:15.123Z INFO provision_command{environment="e2e-full"}:
  Starting infrastructure provisioning workflow
2025-10-03T10:30:15.124Z INFO state_transition{environment="e2e-full"}:
  Transitioning from Created to Provisioning
2025-10-03T10:30:15.125Z DEBUG persistence{environment="e2e-full"}:
  Saving state to build/e2e-full/state.json
... provision steps ...
2025-10-03T10:32:45.678Z INFO state_transition{environment="e2e-full"}:
  Transitioning from Provisioning to Provisioned with IP 10.140.190.14
2025-10-03T10:32:45.679Z DEBUG persistence{environment="e2e-full"}:
  Saving state to build/e2e-full/state.json
2025-10-03T10:32:45.680Z INFO provision_command{environment="e2e-full"}:
  Provisioning completed successfully
```

#### 6. Compile-Time Type Safety Demonstration

**Documentation Tests** (to be added in implementation):

````rust
/// # Compile-Time Type Safety
///
/// The type-state pattern prevents invalid state transitions at compile time:
///
/// ```compile_fail
/// # use torrust_tracker_deploy::domain::environment::{Environment, Created, Provisioned};
/// # use torrust_tracker_deploy::application::commands::ConfigureCommand;
/// let environment: Environment<Created> = Environment::new(/* ... */);
/// let configure_cmd = ConfigureCommand::new(/* ... */);
///
/// // This will NOT compile: ConfigureCommand requires Environment<Provisioned>
/// configure_cmd.execute(environment)?;
/// //                    ^^^^^^^^^^^ expected Environment<Provisioned>, found Environment<Created>
/// ```
///
/// Correct usage requires provisioning first:
///
/// ```rust
/// # use torrust_tracker_deploy::domain::environment::{Environment, Created};
/// let environment: Environment<Created> = Environment::new(/* ... */);
/// let provision_cmd = ProvisionCommand::new(/* ... */);
/// let configure_cmd = ConfigureCommand::new(/* ... */);
///
/// // Type-safe command chaining
/// let provisioned = provision_cmd.execute(environment).await?;
/// let configured = configure_cmd.execute(provisioned)?;  // ✅ Compiles!
/// ```
pub struct ConfigureCommand { /* ... */ }
````

### Testing Strategy

1. **Unit Tests** (Subtasks 1-2):

   - Test individual command state transitions
   - Test error state handling with failed steps
   - Test persistence integration (with mocked repository)
   - Verify compile-time type safety (documentation tests)
   - Test graceful handling of persistence failures

2. **Integration Tests**:

   - Test commands with mocked repository and infrastructure
   - Verify state persistence across command executions
   - Test error propagation and state tracking

3. **E2E Tests**:
   - Update existing E2E tests to use new command signatures
   - Verify full workflow with real infrastructure
   - Verify state transitions are logged
   - Verify persisted state (for file-based E2E tests)

### Estimated Impact

- **New Tests**: ~21 tests (7 Subtask 1 + 7 Subtask 3 + 7 Subtask 4)
  - Subtask 1: 2 unit tests (error helpers) + 5 integration tests (repository/locking)
  - Subtask 3: 7 unit tests (ProvisionCommand state management)
  - Subtask 4: 7 unit tests (ConfigureCommand state management)
- **Total Tests**: ~684 (starting from 663)
- **Test Execution Time**: Similar to current (mocked repository for unit tests)
- **Documentation**:
  - New ADR documenting command state return pattern decision
  - Updated command documentation with state management examples
  - Comprehensive integration considerations documented
- **Code Changes**: ~600 lines (refactoring + commands + integration + tests)
- **Breaking Changes**: Command signatures, container, E2E test binaries (internal only, no public API yet)
- **Subtasks**: 4 (1 refactoring + 2 ADR + 3-4 commands with integration)

### Dependencies

- **Phase 1**: State marker types ✅
- **Phase 2**: State transition logging ✅
- **Phase 3**: Type erasure and serialization ✅
- **Phase 4**: Persistence layer with file locking ✅

### Next Steps After Phase 5

With Phase 5 complete, the environment state management feature will be fully functional:

- ✅ Type-safe state machine (Phase 1)
- ✅ Observable transitions (Phase 2)
- ✅ Serialization support (Phase 3)
- ✅ Persistent storage (Phase 4)
- ✅ Command integration (Phase 5)

**Future Enhancements** (Phase 6+):

- **Command Orchestration Layer**: Build a `DeploymentOrchestrator` that chains commands with compile-time state validation, enabling fluent API like `orchestrator.provision(env).await?.configure()?`
- Add `status` command to query environment state
- Add `destroy` command with state cleanup
- Add `resume` command for recovering from interrupted operations
- Implement validation of infrastructure state vs stored state
- Add event sourcing for full transition history
- Create CLI for user-facing operations

### Risks and Mitigations

| Risk                                  | Impact | Mitigation                                                       |
| ------------------------------------- | ------ | ---------------------------------------------------------------- |
| Breaking E2E tests during refactoring | High   | Incremental changes, maintain backward compatibility temporarily |
| Persistence errors breaking commands  | Medium | Graceful degradation: log warnings, continue execution           |
| Type-state pattern too complex        | Low    | Excellent documentation, examples, clear error messages          |

## 📚 Related Documentation

- [Feature Description](../feature-description.md) - Overall feature goals and requirements
- [Requirements Analysis](../requirements-analysis.md) - Q&A that shaped the feature
- [Phase 1: Foundation](./phase-1-foundation.md) - Type-state pattern implementation
- [Phase 4: Persistence](./phase-4-persistence.md) - Repository and file locking
- [Error Handling Guide](../../../contributing/error-handling.md) - Error handling principles
- [Testing Conventions](../../../contributing/testing/) - Testing best practices

## 🚀 Getting Started

To begin implementing Phase 5:

1. Review this plan thoroughly, especially the **Critical Integration Considerations** section
2. Read related documentation to understand context
3. **Start with Subtask 1 (Preparatory Refactoring)** - Clean up commands AND setup repository infrastructure first
4. Continue with Subtask 2 (Create ADR) - Document the architectural decision
5. Implement Subtask 3 (ProvisionCommand) - **Immediately update Container and E2E binaries** (breaking change!)
6. Implement Subtask 4 (ConfigureCommand) - **Immediately update Container and E2E binaries** (breaking change!)
7. Follow the test-driven development approach for each subtask
8. Commit after each major change within a subtask
9. Update this document with actual test counts and any deviations

**Remember**: The goal is incremental, testable progress with immediate integration:

- **Refactoring + Setup** (Subtask 1) reduces complexity AND prepares infrastructure (non-breaking)
- **Document decision** (Subtask 2) explains the "why" before the "how"
- **Command + Integration** (Subtasks 3-4) implement functionality AND update all callers atomically

**Critical**: Each command update MUST include updating its callers (Container factory + E2E binaries) in the same subtask. Otherwise code won't compile!
