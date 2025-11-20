# Register Existing Instances Specification

## 📋 Overview

This feature adds a new `torrust-tracker-deployer register` command that enables users to bring their own pre-existing infrastructure into the deployer workflow. This solves two critical needs: allowing end users to deploy on their own servers, and providing a proper solution for container-based E2E testing.

### Context

Currently, the Torrust Tracker Deployer only supports the LXD provider for local development and testing. The deployment workflow follows a strict state machine:

```text
Created → Provisioning → Provisioned → Configuring → Configured → Releasing → Released → Running
```

Users must go through the `provision` phase, which creates new infrastructure using OpenTofu. This creates two problems:

1. **For End Users**: No way to register existing infrastructure (spare servers, unsupported cloud providers, custom setups)
2. **For E2E Testing**: Container-based tests require a hacky workaround (`run_provision_simulation.rs`) that manually sets up state to bypass provisioning

### Problem Statement

**Primary Problem**: Users cannot leverage the deployer's configuration and deployment capabilities on already-provisioned infrastructure.

**Secondary Problem**: E2E testing requires containers (faster than VMs, GitHub Actions compatible), but containers aren't "provisioned" in the traditional sense, requiring hacky simulation code.

## 🎯 Goals

### Primary Goals

- **Enable infrastructure reuse**: Allow users to register existing VMs, physical servers, or containers
- **Maintain state machine integrity**: Create environments directly in `Provisioned` state without breaking existing workflows
- **Replace test simulation**: Provide proper solution for container-based E2E tests, removing `run_provision_simulation` hack
- **Validate connectivity**: Ensure registered instances are reachable and meet minimum requirements

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

Add a new `register` command that creates environments directly in the `Provisioned` state, bypassing the `provision` phase. This approach:

- **Maintains state machine integrity**: Uses existing states, just enters at a different point
- **Solves both problems**: Works for end users AND testing scenarios
- **Clear semantics**: "Register" clearly communicates bringing external infrastructure under management
- **Industry precedent**: Parallels GitHub/GitLab runner registration workflows
- **Future-proof**: Can be extended for cloud provider registration, cluster registration, etc.

### Design Overview

```text
Standard Workflow:
  create → provision → configure → release → run
  |         |          |
  Created   Provisioning → Provisioned
                          ↓
                        (continues...)

Register Workflow:
  register → configure → release → run
    |          |
  (No state)   Provisioned
    ↑          ↓
  Directly creates      (continues...)
  environment in
  Provisioned state
```

### Key Design Decisions

1. **Command Name: `register`**

   - **Rationale**: Strong industry precedent (GitHub/GitLab runners, Consul services, Vault backends)
   - **Parallel**: Just like CI runners "register" with GitHub/GitLab, instances "register" with deployer
   - **Alternatives considered**: `import`, `adopt`, `add`, `attach`, `connect` (see Naming Analysis section)
   - **Decision**: Best balance of familiarity, clarity, and professional tone

2. **Direct `Provisioned` State Creation**

   - **Rationale**: Simplest approach, reuses existing state machine
   - **Alternatives considered**: New `Registered` state, metadata flag
   - **Decision**: Less code, simpler to understand, works with all existing commands

3. **Minimal SSH Validation (v1 Scope)** ✅

   - **Rationale**: Start simple - only validate SSH connectivity for v1, defer advanced validation to v2
   - **Decision from Q&A**: "Only basic connectivity validation (minimal)" - defer OS, architecture, disk space checks
   - **Error Handling**: Create environment even if validation fails, inform user, use `ProvisionFailed` state for future validations
   - **User Experience**: Non-interactive command - all parameters via CLI flags or config file

4. **Metadata Tracking for Safety** ✅

   - **Rationale**: Prevent accidental destruction of user-owned infrastructure
   - **Decision from Q&A**: "They should be marked differently. And we should prevent destroying registered instances unless explicitly confirmed"
   - **Implementation**: Add optional "registered" metadata flag to environment
   - **Destroy Command**: Will require confirmation or explicit flag to destroy registered instances (prevents data loss)

5. **No Duplicate Environment Names** ✅
   - **Rationale**: Maintain environment name uniqueness
   - **Decision from Q&A**: "Fail if environment name exists"
   - **Error Handling**: Return clear error message if environment name already taken

### Alternatives Considered

#### Option 1: Extend `create` command with `--host` flag

- **Pros**: Fewer commands, simpler CLI
- **Cons**: Conflates two different workflows, confusing semantics
- **Decision**: Rejected - violates Single Responsibility Principle

#### Option 2: OpenTofu Container Provider

- **Pros**: Consistent with OpenTofu approach
- **Cons**: Only solves testing problem, not for end users, unnecessary complexity
- **Decision**: Rejected - doesn't solve the user need

#### Option 3: Keep simulation hack, add separate user command

- **Pros**: Separation of concerns
- **Cons**: Two different solutions for same problem, more code to maintain
- **Decision**: Rejected - one solution should solve both needs

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
# Parallel to GitHub Actions runner registration
torrust-tracker-deployer register production-tracker \
  --host 192.168.1.100 \
  --ssh-user torrust \
  --ssh-key ~/.ssh/prod-key
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

Add constructor to create `Environment<Provisioned>` directly with metadata support:

```rust
impl Environment<Provisioned> {
    /// Create environment from existing instance (bypass provision phase)
    ///
    /// # Parameters
    /// - `context`: Environment context with instance details
    /// - `registered`: Optional flag to mark environment as registered (for safety)
    ///
    /// # Returns
    /// Environment in Provisioned state, ready for configuration
    pub fn from_existing_instance(
        context: EnvironmentContext,
        registered: bool,
    ) -> Result<Self, DomainError> {
        let mut env = Self {
            context,
            state: PhantomData,
        };

        if registered {
            env.context.metadata.insert("registered".to_string(), "true".to_string());
        }

        Ok(env)
    }
}
```

**File**: `src/domain/environment/mod.rs`

#### Application Layer

Create new command handler with minimal validation:

```rust
pub struct RegisterCommandHandler {
    ssh_client: Arc<SshClient>,
    environment_repository: Arc<dyn EnvironmentRepository>,
}

impl RegisterCommandHandler {
    pub async fn execute(
        &self,
        environment_name: EnvironmentName,
        instance_address: SocketAddr,
        ssh_credentials: SshCredentials,
    ) -> Result<Environment<Provisioned>, RegisterError> {
        // 1. Check for duplicate environment name
        if self.environment_repository.exists(&environment_name)? {
            return Err(RegisterError::EnvironmentExists {
                name: environment_name,
            });
        }

        // 2. Validate SSH connectivity (minimal validation for v1)
        // Note: Even if validation fails, we create the environment but inform the user
        let connectivity_result = self
            .validate_connectivity(instance_address, &ssh_credentials)
            .await;

        if let Err(e) = &connectivity_result {
            // Log warning but continue (as per Q&A decision)
            tracing::warn!(
                "SSH connectivity validation failed, creating environment anyway: {:?}",
                e
            );
        }

        // 3. Create environment context
        let context = EnvironmentContext::new(
            environment_name,
            ssh_credentials,
            instance_address,
        );

        // 4. Create environment in Provisioned state with "registered" metadata
        let environment = Environment::from_existing_instance(context, true)?;

        // 5. Save to repository
        self.environment_repository.save(&environment)?;

        // 6. Return environment with connectivity result as warning
        if connectivity_result.is_err() {
            // In future, we could use ProvisionFailed state here
            tracing::warn!("Environment created but SSH validation failed - subsequent commands may fail");
        }

        Ok(environment)
    }

    async fn validate_connectivity(
        &self,
        address: SocketAddr,
        credentials: &SshCredentials,
    ) -> Result<(), RegisterError> {
        self.ssh_client
            .test_connection(address, credentials)
            .await
            .map_err(|e| RegisterError::ConnectivityFailed {
                address,
                source: e
            })
    }
}
```

**File**: `src/application/commands/register/handler.rs`

#### Presentation Layer (CLI)

Add new subcommand with non-interactive parameters:

```rust
#[derive(Parser)]
pub enum Command {
    // ... existing commands ...

    /// Register an existing instance into a new environment
    ///
    /// This command allows you to use pre-existing infrastructure (VMs, physical
    /// servers, or containers) with the Torrust Tracker Deployer. The registered
    /// instance must meet these requirements:
    ///
    /// REQUIRED:
    /// - Ubuntu 24.04 LTS
    /// - SSH connectivity with provided credentials
    /// - Public SSH key installed for access
    /// - Public IP address reachable from deployer
    /// - Username with sudo access
    /// - Cloud-init completion mark (see templates/tofu/lxd/cloud-init.yml.tera)
    ///
    /// MUST NOT HAVE:
    /// - Incompatible dependencies (old Docker, old systemd, etc.)
    /// - Custom configurations preventing deployer operation
    ///
    /// After registering, you can use the normal deployment workflow:
    /// configure → release → run
    ///
    /// NOTE: Registered environments are marked to prevent accidental destruction.
    Register {
        /// Name for the new environment
        environment: String,

        /// IP address or hostname of the existing instance
        #[arg(long, value_name = "IP", required = true)]
        host: String,

        /// SSH username for accessing the instance
        #[arg(long, value_name = "USERNAME", required = true)]
        ssh_user: String,

        /// Path to SSH private key
        #[arg(long, value_name = "PATH", required = true)]
        ssh_key: PathBuf,

        /// SSH port (default: 22)
        #[arg(long, value_name = "PORT", default_value = "22")]
        ssh_port: u16,
    },
}
```

**File**: `src/presentation/input/cli/commands.rs`

### Data Model

No new data structures required. Reuses existing:

- `Environment<Provisioned>`
- `EnvironmentContext`
- `SshCredentials`
- `SocketAddr` for instance address

### API Changes

#### New Command Handler

```rust
pub trait RegisterCommandHandler {
    async fn execute(
        &self,
        environment_name: EnvironmentName,
        instance_address: SocketAddr,
        ssh_credentials: SshCredentials,
    ) -> Result<Environment<Provisioned>, RegisterError>;
}
```

#### New Error Type

```rust
#[derive(Debug, Error)]
pub enum RegisterError {
    #[error("Failed to connect to instance at {address}")]
    ConnectivityFailed {
        address: SocketAddr,
        #[source]
        source: SshError,
    },

    #[error("Instance validation failed: {reason}")]
    ValidationFailed {
        reason: String,
    },

    #[error("Environment '{name}' already exists")]
    EnvironmentExists {
        name: EnvironmentName,
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
torrust-tracker-deployer register my-tracker \
  --host 192.168.1.100 \
  --ssh-user torrust \
  --ssh-key ~/.ssh/my-server-key \
  --ssh-port 22
```

#### Environment Variables (Optional Enhancement)

```bash
export TORRUST_TD_REGISTER_HOST=192.168.1.100
export TORRUST_TD_REGISTER_SSH_USER=torrust
export TORRUST_TD_REGISTER_SSH_KEY=~/.ssh/my-server-key
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
| `tests/e2e_register_command.rs`                 | E2E tests                      | Medium |

### Files to Modify

| File Path                                | Changes Required                      | Effort |
| ---------------------------------------- | ------------------------------------- | ------ |
| `src/domain/environment/mod.rs`          | Add `from_existing_instance()`        | Low    |
| `src/presentation/input/cli/commands.rs` | Add `Register` variant                | Low    |
| `src/presentation/input/cli/mod.rs`      | Handle `Register` command             | Medium |
| `src/testing/e2e/tasks/container/mod.rs` | Replace simulation with register      | Medium |
| `tests/e2e_create_command.rs`            | Update to use register for containers | Low    |
| `tests/e2e_destroy_command.rs`           | Update to use register for containers | Low    |
| `docs/console-commands.md`               | Document register command             | Medium |
| `docs/features/README.md`                | Add feature to active list            | Low    |

### Breaking Changes

None. This is a purely additive feature.

### Performance Impact

Neutral. The import command is faster than provisioning (no OpenTofu/LXD setup), but individual command performance is unchanged.

### Security Considerations

1. **SSH Key Handling**: Keys must be stored securely and permissions validated
2. **Credential Validation**: SSH connectivity checked (minimal validation for v1)
3. **Instance Trust**: Users responsible for security and compatibility of their own infrastructure
4. **Destroy Protection**: Registered instances marked with metadata, destroy command will require confirmation (future enhancement)
5. **No Auto-Installation**: Users must provide valid SSH keys - no automated key generation or installation

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

### Phase 1: Foundation

- [ ] Create ADR documenting architectural decision
- [ ] Define `RegisterError` enum with clear, actionable messages
- [ ] Add `Environment::from_existing_instance()` constructor with metadata support
- [ ] Add integration tests for domain layer

**Estimated Duration**: 1 day

### Phase 2: Core Implementation

- [ ] Implement `RegisterCommandHandler` with minimal SSH validation
- [ ] Add duplicate environment name check
- [ ] Add metadata tracking for registered instances
- [ ] Add repository integration
- [ ] Handle validation failures gracefully (create env, warn user)
- [ ] Add unit tests for command handler

**Estimated Duration**: 2 days

### Phase 3: CLI Integration

- [ ] Add `Register` command variant with required parameters
- [ ] Add argument parsing and validation
- [ ] Add user output formatting
- [ ] Handle errors with actionable messages
- [ ] Add CLI integration tests

**Estimated Duration**: 1 day

### Phase 4: Testing & E2E Migration (No E2E Tests for Register)

- [ ] Write integration tests with Docker containers
- [ ] Replace `run_provision_simulation` with register in existing E2E tests
- [ ] Verify GitHub Actions compatibility
- [ ] Remove old simulation code (`run_provision_simulation.rs`)
- [ ] Manual testing: Register LXD VM successfully
- [ ] Manual testing: Register Docker container successfully

**Note**: Per Q&A decision - "we do not need E2E tests since the feature will be indirectly tested when we replace the `run_provision_simulation.rs` with the register command in existing E2E tests"

**Estimated Duration**: 2 days

### Phase 5: Documentation

- [ ] Create `docs/user-guide/commands/register.md`
- [ ] Update `docs/console-commands.md` with register command
- [ ] Add examples for common scenarios
- [ ] Update README with register workflow
- [ ] Update all command lists throughout documentation

**Note**: Per Q&A decision - "New command docs in docs/user-guide/commands. Update other parts of the documentation where there is a list of commands."

**Estimated Duration**: 1 day

**Total Estimated Duration**: 7 days

## ✅ Definition of Done

### Functional Requirements

- [ ] Command successfully registers instances with valid SSH credentials
- [ ] Environment created in `Provisioned` state with "registered" metadata
- [ ] Registered environments work identically to provisioned ones for all subsequent commands (`configure`, `release`, `run`)
- [ ] SSH connectivity validated (minimal validation only - defer advanced checks)
- [ ] Environment created even if validation fails (with warning to user)
- [ ] Clear error messages for connectivity failures
- [ ] Clear error messages for invalid credentials
- [ ] Duplicate environment names rejected with helpful error
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
- [ ] Unit tests verify duplicate name detection
- [ ] Unit tests verify metadata tracking
- [ ] Integration tests verify SSH connectivity validation
- [ ] Integration tests verify environment repository integration
- [ ] Integration tests verify graceful handling of validation failures
- [ ] E2E container tests use register instead of simulation
- [ ] All E2E tests pass on GitHub Actions

**Note**: No dedicated E2E tests for register command - indirectly tested through existing E2E tests after replacing `run_provision_simulation.rs`

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
    async fn it_should_create_environment_in_provisioned_state() {
        let ssh_client = Arc::new(MockSshClient::new());
        let repo = Arc::new(MockRepository::new());
        let handler = RegisterCommandHandler::new(ssh_client, repo.clone());

        let result = handler.execute(
            EnvironmentName::new("test-env".to_string())?,
            "192.168.1.100:22".parse()?,
            SshCredentials::new(/* ... */),
        ).await;

        assert!(result.is_ok());
        assert_eq!(repo.saved_environments().len(), 1);
    }

    #[tokio::test]
    async fn it_should_fail_with_connectivity_error() {
        let ssh_client = Arc::new(MockSshClient::failing());
        let handler = RegisterCommandHandler::new(ssh_client, /* ... */);

        let result = handler.execute(/* ... */).await;

        assert!(matches!(result, Err(RegisterError::ConnectivityFailed { .. })));
    }
}
```

**File**: `src/application/commands/register/handler.rs` (in tests module)

### Integration Tests

Test with real Docker containers:

```rust
#[tokio::test]
async fn it_should_register_docker_container() {
    // Start container with SSH
    let container = TestContainer::start().await?;

    // Create services
    let services = TestServices::new()?;

    // Execute register command
    let env = services
        .register_handler
        .execute(
            EnvironmentName::new("test-register".to_string())?,
            container.ssh_socket_addr(),
            container.ssh_credentials(),
        )
        .await?;

    // Verify environment state
    assert!(matches!(env.state_type(), StateType::Provisioned));

    // Cleanup
    container.stop().await?;
}
```

**File**: `tests/integration/register_command.rs`

### End-to-End Tests

Test full import → configure → release workflow:

```rust
#[tokio::test]
async fn it_should_register_and_deploy_on_container() {
    let test_context = TestContext::new_container()?;

    // Register container
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

**File**: `tests/e2e_register_command.rs`

### Manual Testing

Steps for manual verification:

1. Start a Docker container with SSH enabled
2. Run register command: `torrust-tracker-deployer register test --host 127.0.0.1 --ssh-user torrust --ssh-key ./fixtures/testing_rsa --ssh-port 2222`
3. Verify environment created: Check `data/test-env/environment.json` shows `Provisioned` state
4. Run configure command: `torrust-tracker-deployer configure test`
5. Verify configuration succeeded
6. Check that all subsequent commands work normally

## 📚 Related Documentation

- [Development Principles](../../development-principles.md) - Observability, testability, user friendliness
- [Error Handling Guide](../../contributing/error-handling.md) - Clear, actionable errors
- [DDD Layer Placement](../../contributing/ddd-layer-placement.md) - Correct layer for import handler
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

## �🔗 References

### E2E Testing Architecture

Per Q&A answer about testing infrastructure replacement:

**Current Approach** (`run_provision_simulation.rs`):

1. Creates Docker container to act as instance
2. Creates internal state to simulate provisioned instance (data + build contents)
3. Environment state created directly in E2E bootstrap code

**Future Approach** (with register command):

1. Create container (same as before)
2. Use register command to create environment state (replaces manual state setup)
3. Eliminates `run_provision_simulation.rs` entirely

**Code Reference**:

```rust
// OLD: Direct state manipulation
let created_env = test_context
    .environment
    .clone()
    .try_into_created()
    .context("Environment must be in Created state for config tests")?;
let provisioned_env = created_env.start_provisioning().provisioned();

// NEW: Use register command
let provisioned_env = register_container(&test_context).await?;
```

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

### Risk Mitigation

Per Q&A answers on risk assessment:

**Users importing incompatible instances**:  
→ Subsequent commands will fail and inform the user of the issues

**SSH connectivity issues**:  
→ No special handling needed, just inform the user of connectivity issues

**State management complexity**:  
→ Flow should be the same as normal environments after initial registration

**Destroy command accidentally destroying user infrastructure**:  
→ Prevent destroying registered instances (require confirmation or explicit flag)

### Additional References

- Current simulation hack: `src/testing/e2e/tasks/container/run_provision_simulation.rs`
- GitHub Actions E2E issues: `docs/github-actions-issues/`
- State management ADR: `docs/decisions/command-state-return-pattern.md`
- Actionable errors ADR: `docs/decisions/actionable-error-messages.md`

---

**Created**: November 19, 2025  
**Last Updated**: November 19, 2025  
**Status**: ✅ Questions Answered - Ready for Implementation
