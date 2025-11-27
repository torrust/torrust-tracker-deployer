# Register Existing Instances Specification

## 📋 Overview

This feature adds a new `torrust-tracker-deployer register` command that serves as an **alternative to the `provision` command** for environments with existing infrastructure. Instead of provisioning new infrastructure, `register` allows users to provide the IP address of an existing instance, transitioning the environment from `Created` to `Provisioned` state.

This solves two critical needs: allowing end users to deploy on their own servers, and providing a proper solution for container-based E2E testing.

### Context

Currently, the Torrust Tracker Deployer only supports the LXD provider for local development and testing. The deployment workflow follows a strict state machine:

```text
Created → Provisioning → Provisioned → Configuring → Configured → Releasing → Released → Running
```

Users must go through the `provision` phase, which creates new infrastructure using OpenTofu. This creates two problems:

1. **For End Users**: No way to register existing infrastructure (spare servers, unsupported cloud providers, custom setups)
2. **For E2E Testing**: Container-based tests require a hacky workaround (`run_provision_simulation.rs`) that manually sets up state to bypass provisioning

### Key Insight

The `create environment` command creates the environment _concept_ (SSH credentials, name, configuration) - not the actual infrastructure. The infrastructure is either:

- **Provisioned** via the `provision` command (creates new VMs)
- **Registered** via the `register` command (uses existing infrastructure)

Both paths lead to the same `Provisioned` state, and the only runtime output from provisioning stored is the instance IP (see `src/domain/environment/runtime_outputs.rs`). Therefore, `register` only needs the instance IP as input - all other configuration (SSH credentials, etc.) comes from the existing environment.

### Problem Statement

**Primary Problem**: Users cannot leverage the deployer's configuration and deployment capabilities on already-provisioned infrastructure.

**Secondary Problem**: E2E testing requires containers (faster than VMs, GitHub Actions compatible), but containers aren't "provisioned" in the traditional sense, requiring hacky simulation code.

## 🎯 Goals

### Primary Goals

- **Enable infrastructure reuse**: Allow users to register existing VMs, physical servers, or containers as an alternative to provisioning
- **Maintain state machine integrity**: Transition environments from `Created` to `Provisioned` state using the same state machine
- **Replace test simulation**: Provide proper solution for container-based E2E tests, removing `run_provision_simulation` hack
- **Validate connectivity**: Ensure registered instances are reachable using environment's SSH credentials

### Secondary Goals (Nice-to-Have)

- **Metadata tracking**: Mark registered environments differently for safety (prevent accidental destroy of user infrastructure) - **CONFIRMED REQUIRED** ✅
- **User-friendly errors**: Clear, actionable error messages for connectivity and validation issues - **CONFIRMED REQUIRED** ✅

### Non-Goals (Out of Scope for v1)

What this feature explicitly does NOT aim to do:

- **Cluster registration**: Registering multiple instances at once - **CONFIRMED OUT OF SCOPE** ✅
- **Cloud provider integration**: Registering by Hetzner instance ID - **FUTURE ENHANCEMENT** (future work after Hetzner provider)
- **Auto-discovery**: Automatically detecting instance configuration - **CONFIRMED OUT OF SCOPE** ✅
- **Docker Compose migration**: Converting existing Docker deployments - **CONFIRMED OUT OF SCOPE** ✅
- **Advanced instance validation**: Checking OS version, architecture, disk space, memory beyond SSH connectivity - **DEFERRED TO v2** ✅
- **Flexible SSH setup**: Automated key installation or key generation - **CONFIRMED OUT OF SCOPE** (users must provide keys like create command) ✅
- **Automated dependency detection**: Detecting and resolving missing dependencies - **CONFIRMED OUT OF SCOPE** ✅

## 💡 Proposed Solution

### Approach

Add a new `register` command that transitions environments from `Created` to `Provisioned` state by providing the instance IP address. This approach:

- **Maintains state machine integrity**: Uses existing states and transitions, `register` is simply an alternative path to `Provisioned`
- **Avoids duplication**: SSH credentials and other configuration are provided once via `create environment`
- **Simple input**: Only the instance IP is needed (the only runtime output from provisioning)
- **Solves both problems**: Works for end users AND testing scenarios
- **Clear semantics**: "Register" clearly communicates bringing external infrastructure under management
- **Industry precedent**: Parallels GitHub/GitLab runner registration workflows
- **Future-proof**: If `RuntimeOutputs` grows, `register` parameters grow accordingly

### Design Overview

```text
Standard Workflow:
  create environment → provision → configure → release → run
       |                  |
    [Created]        [Provisioned]
                          ↓
                     (continues...)

Register Workflow (Alternative to provision):
  create environment → register → configure → release → run
       |                  |
    [Created]        [Provisioned]
                          ↓
                     (continues...)
```

Both `provision` and `register` result in the same `Provisioned` state with `runtime_outputs.instance_ip` set.

### Key Design Decisions

1. **Register as Alternative to Provision**

   - **Rationale**: The `create environment` command creates the environment concept, not infrastructure. `register` and `provision` are two ways to materialize infrastructure.
   - **Benefit**: No duplication of SSH credentials, environment name, etc.
   - **Benefit**: Simple command - only needs instance IP
   - **Benefit**: Consistent with existing domain model (`RuntimeOutputs` only stores instance IP)

2. **Command Name: `register`**

   - **Rationale**: Strong industry precedent (GitHub/GitLab runners, Consul services, Vault backends)
   - **Parallel**: Just like CI runners "register" with GitHub/GitLab, instances "register" with deployer
   - **Alternatives considered**: `import`, `adopt`, `add`, `attach`, `connect` (see Naming Analysis section)
   - **Decision**: Best balance of familiarity, clarity, and professional tone

3. **State Transition `Created` → `Provisioned`**

   - **Rationale**: Uses existing state machine, same target state as `provision`
   - **Implementation**: Load environment in `Created` state, set `runtime_outputs.instance_ip`, transition to `Provisioned`
   - **Decision**: Reuses existing code, simpler to understand, works with all existing commands

4. **Minimal SSH Validation (v1 Scope)** ✅

   - **Rationale**: Start simple - only validate SSH connectivity for v1, defer advanced validation to v2
   - **Decision from Q&A**: "Only basic connectivity validation (minimal)" - defer OS, architecture, disk space checks
   - **Error Handling**: Transition to `Provisioned` even if validation fails, inform user, use `ProvisionFailed` state for future validations
   - **SSH Credentials**: Uses credentials already stored in environment from `create environment`

5. **Metadata Tracking for Safety** ✅

   - **Rationale**: Prevent accidental destruction of user-owned infrastructure
   - **Decision from Q&A**: "They should be marked differently. And we should prevent destroying registered instances unless explicitly confirmed"
   - **Implementation**: Add `"registered": "true"` metadata flag to environment
   - **Destroy Command**: Will require confirmation or explicit flag to destroy registered instances (prevents data loss)

6. **Environment Must Exist in Created State** ✅
   - **Rationale**: Enforces proper workflow - user must first create environment with SSH credentials
   - **Error Handling**: Return clear error message if environment doesn't exist or is in wrong state

### Alternatives Considered

#### Option 1: Extend `create` command with `--host` flag

- **Pros**: Fewer commands, simpler CLI
- **Cons**: Conflates environment creation with infrastructure registration, confusing semantics
- **Decision**: Rejected - violates Single Responsibility Principle

#### Option 2: OpenTofu Container Provider

- **Pros**: Consistent with OpenTofu approach
- **Cons**: Only solves testing problem, not for end users, unnecessary complexity
- **Decision**: Rejected - doesn't solve the user need

#### Option 3: Keep simulation hack, add separate user command

- **Pros**: Separation of concerns
- **Cons**: Two different solutions for same problem, more code to maintain
- **Decision**: Rejected - one solution should solve both needs

#### Option 4: Register creates new environment with all parameters (original approach)

- **Pros**: Single command for everything
- **Cons**: Duplicates SSH credential handling from `create environment`, different config model
- **Decision**: Rejected - better to reuse existing environment creation and only add instance IP

### Command Naming Analysis

The command name underwent careful analysis to find the best fit for the feature.

#### Selected: `register` ⭐

**Industry Precedents**:

- **GitHub Actions**: `./config.sh --url ... --token ...` registers runner with GitHub
- **GitLab Runners**: `gitlab-runner register` registers runner with GitLab
- **Consul**: `consul services register` registers services with Consul
- **Vault**: Register auth backends, secrets engines
- **Kubernetes**: `kubectl register` (in some contexts)

**Why `register` wins**:

- ✅ **Established pattern**: Developers understand from CI/CD workflows (GitHub/GitLab runners)
- ✅ **Bidirectional relationship**: Server becomes "registered" with the deployer (parallel to runners)
- ✅ **Clear ownership**: Implies ongoing management relationship
- ✅ **Professional tone**: Business-appropriate for infrastructure tools
- ✅ **Discoverable**: Intuitive for users familiar with CI/CD

**Example**:

```bash
# Step 1: Create environment with SSH credentials
torrust-tracker-deployer create environment --env-file prod-config.json

# Step 2: Register existing instance (only needs IP)
torrust-tracker-deployer register production-tracker --instance-ip 192.168.1.100

# Step 3: Continue with normal workflow
torrust-tracker-deployer configure production-tracker
```

#### Alternatives Considered

| Command   | Pros                    | Cons                                 | Verdict  |
| --------- | ----------------------- | ------------------------------------ | -------- |
| `import`  | Common in data contexts | Overloaded (modules, packages, data) | Rejected |
| `adopt`   | Unique semantic meaning | Less familiar in DevOps contexts     | Rejected |
| `add`     | Extremely simple        | Too generic, vague                   | Rejected |
| `attach`  | Physical metaphor       | Implies temporary vs permanent       | Rejected |
| `connect` | Network-oriented        | Sounds like connectivity test        | Rejected |
| `onboard` | Process metaphor        | Too corporate/HR-like                | Rejected |
| `claim`   | Strong ownership        | Too aggressive                       | Rejected |
| `enroll`  | Formal process          | Too academic, longer to type         | Rejected |

## 🔧 Implementation Details

### Architecture Changes

#### Domain Layer

Add state transition method to `Environment<Created>`:

```rust
impl Environment<Created> {
    /// Register an existing instance, transitioning to Provisioned state
    ///
    /// # Parameters
    /// - `instance_ip`: IP address of the existing instance
    ///
    /// # Returns
    /// Environment in Provisioned state with instance_ip set and "registered" metadata
    pub fn register(self, instance_ip: IpAddr) -> Environment<Provisioned> {
        let mut context = self.context;

        // Set the runtime output (same as provision would do)
        context.runtime_outputs.instance_ip = Some(instance_ip);

        // Mark as registered for safety (destroy protection)
        context.metadata.insert("registered".to_string(), "true".to_string());

        Environment {
            context,
            state: PhantomData,
        }
    }
}
```

**File**: `src/domain/environment/mod.rs`

#### Application Layer

Create new command handler that loads existing environment and transitions it:

```rust
pub struct RegisterCommandHandler {
    ssh_client: Arc<SshClient>,
    environment_repository: Arc<dyn EnvironmentRepository>,
}

impl RegisterCommandHandler {
    pub async fn execute(
        &self,
        environment_name: EnvironmentName,
        instance_ip: IpAddr,
    ) -> Result<Environment<Provisioned>, RegisterError> {
        // 1. Load existing environment
        let environment = self
            .environment_repository
            .load(&environment_name)?
            .ok_or_else(|| RegisterError::EnvironmentNotFound {
                name: environment_name.clone(),
            })?;

        // 2. Verify environment is in Created state
        let created_env = environment
            .try_into_created()
            .map_err(|current_state| RegisterError::InvalidState {
                name: environment_name.clone(),
                current_state: current_state.to_string(),
            })?;

        // 3. Get SSH credentials from environment
        let ssh_credentials = created_env.context().ssh_credentials.clone();

        // 4. Validate SSH connectivity (minimal validation for v1)
        // Note: Even if validation fails, we transition the environment but inform the user
        let connectivity_result = self
            .validate_connectivity(instance_ip, &ssh_credentials)
            .await;

        if let Err(e) = &connectivity_result {
            // Log warning but continue (as per Q&A decision)
            tracing::warn!(
                "SSH connectivity validation failed, registering environment anyway: {:?}",
                e
            );
        }

        // 5. Transition to Provisioned state with instance IP and "registered" metadata
        let provisioned_env = created_env.register(instance_ip);

        // 6. Save to repository
        self.environment_repository.save(&provisioned_env)?;

        // 7. Return environment with connectivity result as warning
        if connectivity_result.is_err() {
            tracing::warn!("Environment registered but SSH validation failed - subsequent commands may fail");
        }

        Ok(provisioned_env)
    }

    async fn validate_connectivity(
        &self,
        ip: IpAddr,
        credentials: &SshCredentials,
    ) -> Result<(), RegisterError> {
        let address = SocketAddr::new(ip, credentials.port);
        self.ssh_client
            .test_connection(address, credentials)
            .await
            .map_err(|e| RegisterError::ConnectivityFailed {
                address: ip,
                source: e
            })
    }
}
```

**File**: `src/application/commands/register/handler.rs`

#### Presentation Layer (CLI)

Add new subcommand:

```rust
#[derive(Parser)]
pub enum Command {
    // ... existing commands ...

    /// Register an existing instance for a created environment
    ///
    /// This command is an alternative to 'provision' that uses existing infrastructure
    /// instead of creating new VMs. The environment must already exist (created via
    /// 'create environment') and be in the Created state.
    ///
    /// PREREQUISITES:
    /// - Environment must exist (run 'create environment' first)
    /// - Environment must be in Created state
    ///
    /// INSTANCE REQUIREMENTS:
    /// - Ubuntu 24.04 LTS
    /// - SSH connectivity with credentials from environment
    /// - Public SSH key installed for access
    /// - Public IP address reachable from deployer
    /// - Username with sudo access
    /// - Cloud-init completion mark (see templates/tofu/lxd/cloud-init.yml.tera)
    ///
    /// MUST NOT HAVE:
    /// - Incompatible dependencies (old Docker, old systemd, etc.)
    /// - Custom configurations preventing deployer operation
    ///
    /// After registering, continue with: configure → release → run
    ///
    /// NOTE: Registered environments are marked to prevent accidental destruction.
    Register {
        /// Name of the existing environment to register instance for
        environment: String,

        /// IP address of the existing instance
        #[arg(long, value_name = "IP", required = true)]
        instance_ip: IpAddr,
    },
}
```

**File**: `src/presentation/input/cli/commands.rs`

### Data Model

No new data structures required. Reuses existing:

- `Environment<Created>` - input environment
- `Environment<Provisioned>` - output environment
- `RuntimeOutputs` - stores instance_ip
- `SshCredentials` - already in environment context
- `IpAddr` for instance address

### API Changes

#### New Command Handler

```rust
pub trait RegisterCommandHandler {
    async fn execute(
        &self,
        environment_name: EnvironmentName,
        instance_ip: IpAddr,
    ) -> Result<Environment<Provisioned>, RegisterError>;
}
```

#### New Error Type

```rust
#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("Environment '{name}' not found")]
    EnvironmentNotFound {
        name: EnvironmentName,
    },

    #[error("Environment '{name}' is not in Created state (current: {current_state})")]
    InvalidState {
        name: EnvironmentName,
        current_state: String,
    },

    #[error("Failed to connect to instance at {address}")]
    ConnectivityFailed {
        address: IpAddr,
        #[source]
        source: SshError,
    },

    #[error("Invalid IP address: {value}")]
    InvalidIpAddress {
        value: String,
    },

    #[error("Failed to save environment")]
    RepositorySave {
        #[source]
        source: RepositoryError,
    },
}
```

### Configuration

#### Command-Line Arguments

```bash
# Step 1: Create environment with SSH credentials (already exists)
torrust-tracker-deployer create environment --env-file my-config.json

# Step 2: Register existing instance (only needs IP)
torrust-tracker-deployer register my-tracker --instance-ip 192.168.1.100
```

## 📊 Impact Analysis

### Files to Create

| File Path                                       | Purpose                        | Effort |
| ----------------------------------------------- | ------------------------------ | ------ |
| `src/application/commands/register/mod.rs`      | Register command module root   | Low    |
| `src/application/commands/register/handler.rs`  | Register command handler logic | Medium |
| `src/application/commands/register/error.rs`    | Register-specific errors       | Low    |
| `docs/user-guide/commands/register.md`          | User documentation             | Medium |
| `docs/decisions/register-existing-instances.md` | ADR for this feature           | Low    |
| `tests/integration/register_command.rs`         | Integration tests              | Medium |

### Files to Modify

| File Path                                | Changes Required                          | Effort |
| ---------------------------------------- | ----------------------------------------- | ------ |
| `src/domain/environment/mod.rs`          | Add `register()` method to Created state  | Low    |
| `src/presentation/input/cli/commands.rs` | Add `Register` variant                    | Low    |
| `src/presentation/input/cli/mod.rs`      | Handle `Register` command                 | Medium |
| `src/testing/e2e/tasks/container/mod.rs` | Replace simulation with register          | Medium |
| `src/bin/e2e_config_tests.rs`            | Refactor to black-box test using register | Medium |
| `docs/console-commands.md`               | Document register command                 | Medium |
| `docs/features/README.md`                | Add feature to active list                | Low    |

**Note**: `tests/e2e_create_command.rs` and `tests/e2e_destroy_command.rs` are black-box tests that only test environment creation/destruction without provisioning - they do not need updates.

### Breaking Changes

None. This is a purely additive feature.

### Performance Impact

Neutral. The register command is faster than provisioning (no OpenTofu/LXD setup), but individual command performance is unchanged.

### Security Considerations

1. **SSH Key Handling**: Keys already stored in environment from `create environment`
2. **Credential Validation**: SSH connectivity checked using environment's credentials (minimal validation for v1)
3. **Instance Trust**: Users responsible for security and compatibility of their own infrastructure
4. **Destroy Protection**: Registered instances marked with metadata, destroy command will require confirmation (future enhancement)

### Instance Requirements

Per Q&A decision, registered instances must meet these requirements:

**REQUIRED**:

- Ubuntu 24.04 LTS (exact version)
- SSH connectivity with provided credentials
- Public SSH key installed for deployer access
- Public IP address reachable from deployer machine
- User account with sudo privileges
- Cloud-init completion mark (same as provisioned instances)
- All dependencies from cloud-init template (see `templates/tofu/lxd/cloud-init.yml.tera`)

**MUST NOT HAVE**:

- Incompatible dependencies (outdated Docker, systemd, etc.)
- Custom configurations preventing deployer operation
- Security restrictions blocking required operations

**VALIDATION STRATEGY (v1)**:

- Only SSH connectivity validated
- Subsequent commands (configure, release, run) will fail with clear errors if requirements not met
- Future versions may add advanced validation (OS version, architecture, disk space, memory)

## 🗓️ Implementation Plan

### Priority and Timeline

**Priority**: **HIGH** ✅
**Rationale**: Simplifies E2E tests before adding Hetzner provider, making test changes easier
**Timeline**: No fixed deadline
**Dependencies**: Must be implemented BEFORE Hetzner provider

### Phase 1: Foundation (0.5 day)

- [ ] Create ADR documenting architectural decision
- [ ] Define `RegisterError` enum with clear, actionable messages
- [ ] Add `Environment<Created>::register(instance_ip)` method for state transition
- [ ] Add unit tests for domain layer state transition

### Phase 2: Core Implementation (1 day)

- [ ] Implement `RegisterCommandHandler` with minimal SSH validation
- [ ] Load existing environment and verify Created state
- [ ] Add metadata tracking for registered instances
- [ ] Add repository integration
- [ ] Handle validation failures gracefully (transition anyway, warn user)
- [ ] Add unit tests for command handler

### Phase 3: CLI Integration (0.5 day)

- [ ] Add `Register` command variant with environment name and `--instance-ip` parameter
- [ ] Add argument parsing and validation
- [ ] Add user output formatting
- [ ] Handle errors with actionable messages
- [ ] Add CLI integration tests

### Phase 4: Testing & E2E Migration (2 days)

- [ ] Write integration tests with Docker containers
- [ ] Replace `run_provision_simulation` with register in existing E2E tests
- [ ] Verify GitHub Actions compatibility
- [ ] Remove old simulation code (`run_provision_simulation.rs`)
- [ ] Manual testing: Register LXD VM successfully
- [ ] Manual testing: Register Docker container successfully

**Note**: Per Q&A decision - "we do not need E2E tests since the feature will be indirectly tested when we replace the `run_provision_simulation.rs` with the register command in existing E2E tests"

### Phase 5: Documentation (1 day)

- [ ] Create `docs/user-guide/commands/register.md`
- [ ] Update `docs/console-commands.md` with register command
- [ ] Add examples for common scenarios
- [ ] Document prerequisites (create environment first)
- [ ] Update all command lists throughout documentation

**Note**: Per Q&A decision - "New command docs in docs/user-guide/commands. Update other parts of the documentation where there is a list of commands."

**Total Estimated Duration**: 5 days

## ✅ Definition of Done

### Functional Requirements

- [ ] Command requires environment to exist in `Created` state
- [ ] Command only requires `--instance-ip` parameter (SSH credentials come from environment)
- [ ] Environment transitions to `Provisioned` state with `runtime_outputs.instance_ip` set
- [ ] Registered environments marked with `"registered": "true"` metadata
- [ ] Registered environments work identically to provisioned ones for all subsequent commands (`configure`, `release`, `run`)
- [ ] SSH connectivity validated using environment's SSH credentials (minimal validation only)
- [ ] Environment transitions even if validation fails (with warning to user)
- [ ] Clear error messages for environment not found
- [ ] Clear error messages for wrong environment state
- [ ] Clear error messages for connectivity failures
- [ ] Destroy command prevents destruction of registered instances (requires confirmation)
- [ ] Manual test: Successfully register LXD VM
- [ ] Manual test: Successfully register Docker container

### Technical Requirements

- [ ] Code follows DDD layer placement guidelines
- [ ] All linters pass (clippy, rustfmt, markdownlint, yamllint)
- [ ] No compiler warnings
- [ ] No unused dependencies (cargo machete)
- [ ] Error handling follows project conventions
- [ ] Logging provides sufficient traceability

### Testing Requirements

- [ ] Unit tests cover `RegisterCommandHandler` logic
- [ ] Unit tests verify state transition from `Created` to `Provisioned`
- [ ] Unit tests verify metadata tracking
- [ ] Integration tests verify SSH connectivity validation
- [ ] Integration tests verify environment repository integration
- [ ] Integration tests verify graceful handling of validation failures
- [ ] `src/bin/e2e_config_tests.rs` refactored to black-box test using register command
- [ ] All E2E tests pass on GitHub Actions

**Note**: No dedicated E2E tests for register command - indirectly tested through `e2e_config_tests.rs` after replacing `run_provision_simulation.rs`. The black-box tests (`e2e_create_command.rs`, `e2e_destroy_command.rs`) only test create/destroy without provisioning and don't need updates.

### Documentation Requirements

- [ ] User-facing documentation in `docs/user-guide/commands/register.md`
- [ ] ADR in `docs/decisions/register-existing-instances.md`
- [ ] Examples in README showing register workflow
- [ ] Console commands documentation updated (`docs/console-commands.md`)
- [ ] All command lists throughout documentation updated
- [ ] Code comments for complex logic
- [ ] Instance requirements documented (Ubuntu 24.04, SSH, cloud-init, etc.)
- [ ] Feature marked as complete in `docs/features/README.md`

### Review and Approval

- [ ] Code review completed by maintainers
- [ ] Technical architecture review
- [ ] All feedback addressed
- [ ] Pre-commit checks pass
- [ ] Ready to merge

## 🧪 Testing Strategy

### Unit Tests

Test command handler logic in isolation:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::mocks::{MockSshClient, MockRepository};

    #[tokio::test]
    async fn it_should_transition_environment_to_provisioned_state() {
        let ssh_client = Arc::new(MockSshClient::new());
        let repo = Arc::new(MockRepository::with_created_environment("test-env"));
        let handler = RegisterCommandHandler::new(ssh_client, repo.clone());

        let result = handler.execute(
            EnvironmentName::new("test-env".to_string())?,
            "192.168.1.100".parse()?,
        ).await;

        assert!(result.is_ok());
        let env = result.unwrap();
        assert_eq!(env.context().runtime_outputs.instance_ip, Some("192.168.1.100".parse().unwrap()));
        assert_eq!(env.context().metadata.get("registered"), Some(&"true".to_string()));
    }

    #[tokio::test]
    async fn it_should_fail_if_environment_not_found() {
        let ssh_client = Arc::new(MockSshClient::new());
        let repo = Arc::new(MockRepository::empty());
        let handler = RegisterCommandHandler::new(ssh_client, repo);

        let result = handler.execute(
            EnvironmentName::new("nonexistent".to_string())?,
            "192.168.1.100".parse()?,
        ).await;

        assert!(matches!(result, Err(RegisterError::EnvironmentNotFound { .. })));
    }

    #[tokio::test]
    async fn it_should_fail_if_environment_not_in_created_state() {
        let ssh_client = Arc::new(MockSshClient::new());
        let repo = Arc::new(MockRepository::with_provisioned_environment("test-env"));
        let handler = RegisterCommandHandler::new(ssh_client, repo);

        let result = handler.execute(
            EnvironmentName::new("test-env".to_string())?,
            "192.168.1.100".parse()?,
        ).await;

        assert!(matches!(result, Err(RegisterError::InvalidState { .. })));
    }
}
```

**File**: `src/application/commands/register/handler.rs` (in tests module)

### Integration Tests

Test with real Docker containers:

```rust
#[tokio::test]
async fn it_should_register_docker_container() {
    // Create environment first
    let test_context = TestContext::new()?;
    create_environment(&test_context, "test-register").await?;

    // Start container with SSH
    let container = TestContainer::start().await?;

    // Execute register command with container IP
    let env = test_context
        .register_handler
        .execute(
            EnvironmentName::new("test-register".to_string())?,
            container.ip_address(),
        )
        .await?;

    // Verify environment state
    assert!(matches!(env.state_type(), StateType::Provisioned));
    assert_eq!(env.context().runtime_outputs.instance_ip, Some(container.ip_address()));

    // Cleanup
    container.stop().await?;
}
```

**File**: `tests/integration/register_command.rs`

### End-to-End Tests

Test full create → register → configure → release workflow:

```rust
#[tokio::test]
async fn it_should_register_and_deploy_on_container() {
    let test_context = TestContext::new_container()?;

    // Create environment first
    create_environment(&test_context).await?;

    // Register container (instead of provision)
    let env = register_container(&test_context).await?;
    assert_eq!(env.state_type(), StateType::Provisioned);

    // Configure
    let env = configure_environment(&test_context, env).await?;
    assert_eq!(env.state_type(), StateType::Configured);

    // Release
    let env = release_tracker(&test_context, env).await?;
    assert_eq!(env.state_type(), StateType::Released);

    // Verify deployment
    verify_tracker_running(&test_context).await?;
}
```

**File**: Existing E2E tests will be updated to use register instead of simulation

### Manual Testing

Steps for manual verification:

1. Create environment: `torrust-tracker-deployer create environment --env-file config.json`
2. Start a Docker container with SSH enabled
3. Run register command: `torrust-tracker-deployer register test-env --instance-ip 127.0.0.1`
4. Verify environment updated: Check `data/test-env/environment.json` shows `Provisioned` state with instance_ip set
5. Run configure command: `torrust-tracker-deployer configure test-env`
6. Verify configuration succeeded
7. Check that all subsequent commands work normally

## 📚 Related Documentation

- [Development Principles](../../development-principles.md) - Observability, testability, user friendliness
- [Error Handling Guide](../../contributing/error-handling.md) - Clear, actionable errors
- [DDD Layer Placement](../../contributing/ddd-layer-placement.md) - Correct layer for register handler
- [Runtime Outputs](../../../src/domain/environment/runtime_outputs.rs) - Shows instance_ip is the only provisioning output
- [State Machine](../../../src/domain/environment/state/mod.rs) - Environment states and transitions
- [Roadmap](../../roadmap.md) - Feature fits with Hetzner provider future work
- [VM Providers](../../vm-providers.md) - Context on LXD vs containers

## � Future Enhancements

### Destroy Command Protection

**Current**: Registered instances marked with metadata but not protected from destruction  
**Future**: Destroy command checks metadata and requires explicit confirmation for registered instances  
**Rationale**: Prevent accidental data loss from destroying user-owned infrastructure  
**Decision from Q&A**: "We should prevent destroying registered instances unless explicitly confirmed by the user"

### Advanced Instance Validation (v2)

**Current**: Only SSH connectivity validated  
**Future**: Validate OS version, architecture, disk space, memory, required dependencies  
**Rationale**: Deferred to v2 to keep initial implementation simple  
**Decision from Q&A**: "Only basic connectivity validation (minimal)" - advanced validation out of scope for v1

### Cloud Provider Registration

**Current**: Register by IP address only  
**Future**: Register Hetzner instances by instance ID, support multiple cloud providers  
**Rationale**: Wait until Hetzner provider implemented to see patterns emerge  
**Timeline**: After Hetzner provider development  
**Decision from Q&A**: "The users will be able to register any existing virtual machine that meets the requirements"

### Re-register Command

**Future Enhancement**: Add command to destroy environment metadata without destroying instance  
**Use Case**: Allow users to "unregister" instances without deleting them  
**Current Workaround**: Users can manually delete deployer data directory  
**Decision from Q&A**: "In the future we could add a reregister command to destroy the metadata without destroying the instance. But we do not need it for now."

### ProvisionFailed State Usage

**Future Enhancement**: Use `ProvisionFailed` state for validation failures instead of just warnings  
**Rationale**: Better state machine representation of registration issues  
**Decision from Q&A**: "Use ProvisionFailed state for future validations"

## 🔗 References

### E2E Testing Architecture

Per Q&A answer about testing infrastructure replacement:

**Current Approach** (`run_provision_simulation.rs`):

1. Creates Docker container to act as instance
2. Creates internal state to simulate provisioned instance (data + build contents)
3. Environment state created directly in E2E bootstrap code

**Future Approach** (with register command):

1. Create environment first (provides SSH credentials)
2. Create container (same as before)
3. Use register command to transition environment to Provisioned state (replaces manual state setup)
4. Eliminates `run_provision_simulation.rs` entirely

**Code Reference**:

```rust
// OLD: Direct state manipulation
let created_env = test_context
    .environment
    .clone()
    .try_into_created()
    .context("Environment must be in Created state for config tests")?;
let provisioned_env = created_env.start_provisioning().provisioned();

// NEW: Use register command (environment already created)
let provisioned_env = register_handler.execute(
    environment_name,
    container.ip_address(),
).await?;
```

### Instance Requirements

Per Q&A decision, registered instances must meet these requirements:

**REQUIRED**:

- Ubuntu 24.04 LTS (exact version)
- SSH connectivity with credentials from environment
- Public SSH key installed for deployer access
- Public IP address reachable from deployer machine
- User account with sudo privileges
- Cloud-init completion mark (same as provisioned instances)
- All dependencies from cloud-init template (see `templates/tofu/lxd/cloud-init.yml.tera`)

**MUST NOT HAVE**:

- Incompatible dependencies (outdated Docker, systemd, etc.)
- Custom configurations preventing deployer operation
- Security restrictions blocking required operations

### Risk Mitigation

Per Q&A answers on risk assessment:

**Users importing incompatible instances**:  
→ Subsequent commands will fail and inform the user of the issues

**SSH connectivity issues**:  
→ No special handling needed, just inform the user of connectivity issues

**State management complexity**:  
→ Flow should be the same as normal environments after initial registration (same `Provisioned` state)

**Destroy command accidentally destroying user infrastructure**:  
→ Prevent destroying registered instances (require confirmation or explicit flag)

### Additional References

- Runtime Outputs: `src/domain/environment/runtime_outputs.rs` - Shows instance_ip is the only provisioning output
- Current simulation hack: `src/testing/e2e/tasks/container/run_provision_simulation.rs`
- GitHub Actions E2E issues: `docs/github-actions-issues/`
- State management ADR: `docs/decisions/command-state-return-pattern.md`
- Actionable errors ADR: `docs/decisions/actionable-error-messages.md`

---

**Created**: November 19, 2025  
**Last Updated**: November 27, 2025  
**Status**: ✅ Questions Answered - Ready for Implementation
