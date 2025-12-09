# Decision: Register Command SSH Port Override

## Status

✅ Accepted

## Date

2025-12-09

## Context

The E2E configuration tests were failing on GitHub Actions runners due to an SSH port conflict. The issue manifested in these ways:

### Problem Analysis

1. **GitHub Actions Environment**: GitHub-hosted runners have SSH service running on port 22
2. **Docker Host Networking Limitation**: When using host networking mode (`--network host`), the container's SSH port 22 directly conflicts with the runner's SSH port 22
3. **Bridge Networking Challenge**: Switching to Docker bridge networking resolves the port conflict (Docker maps container port 22 to a random host port like 33061), but creates a new problem:
   - The `register` command reads SSH port from environment configuration (port 22)
   - The actual SSH server is accessible on the mapped port (e.g., 33061)
   - SSH connectivity validation fails with "Connection refused"
4. **Ansible Inventory Issue**: Even if we could manually update the environment config file, Ansible inventory files are rendered with the SSH port from configuration, causing the `configure` command to fail

### Real-World Use Case

Beyond E2E testing, this feature addresses legitimate production scenarios:

- Registering instances where SSH runs on non-standard ports for security
- Working with containerized environments where port mapping is common
- Connecting to instances behind port-forwarding configurations
- Testing against development environments with alternative SSH configurations

## Decision

Implement an optional `--ssh-port` CLI argument for the `register` command that overrides the SSH port from environment configuration for both:

1. **SSH connectivity validation** during registration
2. **Ansible inventory generation** for subsequent configuration steps

### Implementation Strategy

**Layer-by-layer propagation**:

```text
CLI Argument (--ssh-port 33061)
  ↓
Presentation Layer (RegisterCommandController)
  ↓
Application Layer (RegisterCommandHandler)
  ├─→ SSH Connectivity Validation (use custom port)
  └─→ Ansible Template Service (use custom port in inventory)
```

**Key Design Decisions**:

- **Optional Parameter**: Make `--ssh-port` optional to maintain backward compatibility
- **Port Priority**: Custom port takes precedence over environment configuration
- **Service Layer Support**: Add `ssh_port_override: Option<u16>` to `AnsibleTemplateService.render_templates()`
- **Clean Propagation**: Pass custom port explicitly through all layers (no global state)

### Code Changes

1. **CLI** (`src/presentation/input/cli/commands.rs`):

   ```rust
   Register {
       environment: String,
       #[arg(long, value_name = "IP_ADDRESS")]
       instance_ip: String,
       #[arg(long, value_name = "PORT")]
       ssh_port: Option<u16>,
   }
   ```

2. **Application Service** (`src/application/services/ansible_template_service.rs`):

   ```rust
   pub async fn render_templates(
       &self,
       user_inputs: &UserInputs,
       instance_ip: IpAddr,
       ssh_port_override: Option<u16>,
   ) -> Result<(), AnsibleTemplateServiceError> {
       let effective_ssh_port = ssh_port_override.unwrap_or(user_inputs.ssh_port);
       // Use effective_ssh_port for inventory generation
   }
   ```

3. **E2E Testing** (`src/bin/e2e_config_and_release_tests.rs`):

   ```rust
   let ssh_port = runtime_env.container_ports.ssh_port;
   test_runner.register_instance(&instance_ip, Some(ssh_port))?;
   ```

## Consequences

### Positive

- ✅ **E2E Tests Work on GitHub Actions**: No more SSH port conflicts on CI runners
- ✅ **Production Feature**: Addresses real-world scenarios (non-standard SSH ports, containerized environments)
- ✅ **Backward Compatible**: Existing workflows unchanged (provision uses environment config)
- ✅ **Clean Architecture**: Port override flows through all layers without side effects
- ✅ **Ansible Integration**: Custom port correctly propagated to inventory files
- ✅ **Type Safety**: Optional parameter makes the override explicit and self-documenting

### Neutral

- 🔷 **Additional Parameter**: Adds one more optional CLI argument (documented and justified)
- 🔷 **E2E Complexity**: E2E tests need to track both config port and runtime mapped port (already necessary with bridge networking)

### Negative

- ⚠️ **Potential Confusion**: Users might wonder why they need to specify SSH port when it's in the environment config
  - **Mitigation**: Clear documentation explaining use cases (non-standard ports, port forwarding, testing)
- ⚠️ **Not Persisted**: Custom SSH port is not saved to environment state (only used for registration)
  - **Rationale**: This is intentional - the custom port is for initial connectivity, not permanent configuration
  - **Future Enhancement**: If needed, we could add a flag like `--update-config` to persist the custom port

## Alternatives Considered

### 1. Modify Environment Config File During E2E Tests

**Approach**: Update `environment.json` with the mapped SSH port before calling register.

**Rejected because**:

- ❌ Modifies test input data (bad practice - tests should not mutate their configuration)
- ❌ Creates coupling between container setup and config file management
- ❌ Doesn't address real-world use cases where SSH port differs from configuration
- ❌ Harder to maintain and reason about (implicit state mutation)

### 2. Skip Register Command in E2E Tests

**Approach**: Manually create the Provisioned state without using the register command.

**Rejected because**:

- ❌ Doesn't test the actual register command workflow
- ❌ Reduces test coverage (register command is a critical user-facing feature)
- ❌ Misses potential bugs in register command logic
- ❌ Doesn't solve the real-world use case of non-standard SSH ports

### 3. Revert to Host Networking

**Approach**: Keep using `--network host` and find another solution for GitHub Actions.

**Rejected because**:

- ❌ Doesn't solve the fundamental port conflict on GitHub Actions
- ❌ Host networking has other limitations and security concerns
- ❌ Bridge networking is the standard Docker networking mode
- ❌ Would require custom GitHub Actions configuration (self-hosted runners)

### 4. Auto-Detect Mapped Port

**Approach**: Automatically discover the mapped SSH port from Docker and use it.

**Rejected because**:

- ❌ Only works for Docker environments (not for real VMs or physical servers)
- ❌ Adds Docker API dependency to production code
- ❌ Doesn't help users who genuinely have non-standard SSH ports
- ❌ More complex implementation with limited benefit

### 5. Environment Variable Override

**Approach**: Use an environment variable like `TORRUST_TD_OVERRIDE_SSH_PORT=33061`.

**Rejected because**:

- ❌ Less explicit than CLI argument (harder to discover and understand)
- ❌ Environment variables should be for operational configuration, not runtime overrides
- ❌ CLI argument is more testable and easier to reason about
- ❌ Doesn't follow project conventions (CLI-first approach)

## Related Decisions

- [Docker Testing Evolution](./docker-testing-evolution.md) - Evolution of Docker strategy for E2E testing
- [Environment Variable Prefix](./environment-variable-prefix.md) - Project environment variable naming convention

## References

- **GitHub Issue**: [#221 - Tracker Slice - Release and Run Commands](https://github.com/torrust/torrust-tracker-deployer/pull/221)
- **Implementation Commit**: `f16d6cd` - feat: [#221] add optional --ssh-port argument to register command
- **E2E Testing Guide**: [docs/e2e-testing.md](../e2e-testing.md)
- **Register Command User Guide**: [docs/user-guide/commands/register.md](../user-guide/commands/register.md)
- **Docker Bridge Networking**: <https://docs.docker.com/network/bridge/>
- **GitHub Actions SSH Port Conflict**: SSH service on runners uses port 22 by default
