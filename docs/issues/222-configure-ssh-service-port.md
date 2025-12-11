# Configure SSH Service Port During Infrastructure Configuration

**Issue**: #222
**Parent Epic**: #TBD - Infrastructure Configuration Epic
**Related**:

- [docs/codebase-architecture.md](../docs/codebase-architecture.md)
- [docs/contributing/templates.md](../docs/contributing/templates.md)
- [docs/contributing/error-handling.md](../docs/contributing/error-handling.md)

## Overview

The deployer currently accepts a custom SSH port configuration in the environment JSON file (`ssh_credentials.port`), and this value is correctly propagated to firewall rules, Ansible inventory, and all connection attempts. However, **the SSH service (`sshd`) on the remote instance is never reconfigured to listen on the custom port** - it continues listening only on the default port 22.

This creates a critical configuration mismatch:

- ✅ Firewall is configured to allow traffic on the custom port (e.g., 2222)
- ✅ Ansible attempts to connect on the custom port
- ✅ SSH client attempts to connect on the custom port
- ❌ **SSH service is listening only on port 22**

**Result**: The `provision` command fails immediately after VM creation because Ansible cannot connect to the instance on the configured custom port. The deployment cannot proceed beyond provisioning.

## Reproduction Evidence

### Manual Test Results (December 11, 2025)

A manual test was performed to reproduce and document the issue:

**Test Setup**:

- Created environment with `ssh_port: 2222` in configuration
- LXD VM provisioned successfully (IP: 10.140.190.205)
- VM is running and healthy

**Findings**:

1. **SSH Service Configuration** (checked via SSH on port 22):

   ```bash
   $ ssh -i fixtures/testing_rsa torrust@10.140.190.205 -p 22 "grep '^Port\|^#Port' /etc/ssh/sshd_config"
   #Port 22
   ```

   ✅ **Confirmed**: SSH service uses default port 22 (commented line means default)

2. **SSH Service Listening Ports** (checked via `ss -tlnp`):

   ```bash
   $ ssh -i fixtures/testing_rsa torrust@10.140.190.205 -p 22 "ss -tlnp | grep :22"
   LISTEN 0      4096         0.0.0.0:22        0.0.0.0:*
   LISTEN 0      4096            [::]:22           [::]:*
   ```

   ✅ **Confirmed**: SSH daemon listening ONLY on port 22 (both IPv4 and IPv6)

3. **Port 22 Connectivity** (direct SSH test):

   ```bash
   $ ssh -i fixtures/testing_rsa torrust@10.140.190.205 -p 22 "echo hello"
   hello
   ```

   ✅ **Confirmed**: Port 22 is accessible and working

4. **Port 2222 Connectivity** (direct SSH test):

   ```bash
   $ ssh -i fixtures/testing_rsa torrust@10.140.190.205 -p 2222 "echo hello"
   ssh: connect to host 10.140.190.205 port 2222: Connection refused
   ```

   ✅ **Confirmed**: Port 2222 is not accepting connections (service not listening)

5. **Ansible Configuration** (generated files):

   ```yaml
   # build/test-port-2222/ansible/variables.yml
   ssh_port: 2222

   # build/test-port-2222/ansible/inventory.yml
   ansible_port: 2222
   ```

   ✅ **Confirmed**: Ansible is configured to connect on port 2222

6. **Provision Failure** (error from traces):

   ```text
   Task failed: Failed to connect to the host via ssh:
   ssh: connect to host 10.140.190.205 port 2222: Connection refused

   Failed Step: WaitSshConnectivity
   ```

   ✅ **Confirmed**: Ansible cannot connect because SSH is on port 22, not 2222

### What We Learned

1. **The problem occurs during PROVISION, not CONFIGURE**: The `provision` command includes a "wait-cloud-init" step that requires SSH connectivity. Since Ansible is configured to use port 2222 but SSH is listening on port 22, this step fails immediately.

2. **Port 22 works, port 2222 doesn't**: The VM is healthy and SSH is working perfectly on port 22. Port 2222 is not open because the SSH service was never configured to listen on it.

3. **Configuration flows correctly through templates**: The user-specified port 2222 is correctly propagated to:

   - Ansible variables (`ssh_port: 2222`)
   - Ansible inventory (`ansible_port: 2222`)
   - All subsequent connection attempts

   But the SSH service configuration (`/etc/ssh/sshd_config`) is never modified.

4. **Cloud-init doesn't configure SSH port**: The `cloud-init.yml.tera` template creates the user and installs SSH keys, but doesn't modify the SSH service port. Cloud-init just enables and starts SSH with default configuration.

5. **Missing configuration step**: There is no Ansible playbook to reconfigure the SSH service port. The configuration is needed AFTER provisioning but BEFORE any Ansible playbooks try to connect.

### Implications

- **Current workaround**: Users must use port 22 (cannot customize SSH port)
- **Future functionality**: After fixing this issue, custom SSH ports will work end-to-end
- **Configuration phase**: The SSH port reconfiguration should happen during the `configure` command (after provision, before any other configuration steps)
- **Sequencing requirement**: SSH port must be reconfigured BEFORE firewall rules are applied (to maintain connectivity)

## Problem Discovery

The issue was identified through code review when examining the complete SSH configuration workflow, then confirmed through manual testing with a real LXD VM instance.

## Goals

- [ ] Create a new Ansible playbook to reconfigure SSH service port
- [ ] Add a new configuration step to execute the playbook conditionally
- [ ] Ensure the step only executes when port ≠ 22 (skip for default)
- [ ] Update the `execute_configuration_with_tracking` method to include the new step
- [ ] Add comprehensive tests for both default and custom port scenarios
- [ ] Document the SSH port configuration process

## 🏗️ Architecture Requirements

**DDD Layer**: Application Layer (Steps) + Infrastructure Layer (Ansible playbook template)
**Module Path**:

- Application: `src/application/steps/system/configure_ssh_port.rs`
- Infrastructure: `templates/ansible/configure-ssh-port.yml`

**Pattern**: Step (Application) + Ansible Playbook (Infrastructure)

### Module Structure Requirements

- [ ] Follow DDD layer separation (see [docs/codebase-architecture.md](../docs/codebase-architecture.md))
- [ ] System configuration belongs in `src/application/steps/system/` (like `configure_firewall.rs`)
- [ ] Ansible playbook belongs in `templates/ansible/` (static, no `.tera` extension needed)
- [ ] Step uses `AnsibleClient` to execute the playbook
- [ ] Respect dependency flow: Application → Infrastructure (Ansible adapter)

### Architectural Constraints

- [ ] No business logic in presentation layer
- [ ] Error handling follows project conventions (see [docs/contributing/error-handling.md](../docs/contributing/error-handling.md))
- [ ] Use structured logging with `tracing` crate for observability
- [ ] Follow the three-level architecture pattern: Command → Step → Action (Ansible playbook)
- [ ] State tracking must record which step failed for proper error context

### Anti-Patterns to Avoid

- ❌ Modifying SSH config during provisioning (belongs in configure phase)
- ❌ Hardcoding port values instead of reading from configuration
- ❌ Skipping validation that SSH service restarted successfully
- ❌ Not verifying connectivity after port change

## Specifications

### 1. New Ansible Playbook: `configure-ssh-port.yml`

Create a new static Ansible playbook at `templates/ansible/configure-ssh-port.yml` that:

1. **Reads the target SSH port** from `variables.yml` (`ssh_port` variable)
2. **Backs up the original sshd_config** to `/etc/ssh/sshd_config.backup`
3. **Modifies `/etc/ssh/sshd_config`** to set `Port {{ ssh_port }}`
4. **Validates the configuration** using `sshd -t`
5. **Restarts the SSH service** using `systemctl restart ssh`
6. **Verifies the service is listening** on the correct port using `ss -tlnp | grep sshd`

**Example Playbook Structure**:

```yaml
---
# Configure SSH Service Port
# This playbook reconfigures the SSH service to listen on a custom port.
#
# CRITICAL: This playbook must be executed FIRST in the configure phase, before
# any other configuration steps, because:
# 1. The provision phase already failed trying to connect on the custom port
# 2. Ansible inventory is configured for the custom port (e.g., 2222)
# 3. SSH is currently listening only on port 22 (default)
# 4. We need to connect on port 22 initially, reconfigure, then switch to custom port
#
# SPECIAL HANDLING REQUIRED:
# - This playbook must temporarily override ansible_port to 22 for the connection
# - After SSH service restarts on custom port, subsequent tasks use the custom port
# - The playbook should skip if ssh_port == 22 (idempotent for default config)
#
# Variables are loaded from variables.yml for centralized management.

- name: Configure SSH service port
  hosts: all
  become: yes
  gather_facts: yes
  vars_files:
    - variables.yml
  vars:
    # CRITICAL: Override ansible_port to 22 for initial connection
    # The inventory has ansible_port set to the custom port (e.g., 2222)
    # but SSH is currently listening on 22, so we need to connect there first
    ansible_port: 22

  tasks:
    - name: Check if SSH port reconfiguration is needed
      ansible.builtin.set_fact:
        needs_reconfiguration: "{{ ssh_port != 22 }}"
      tags:
        - ssh
        - check

    - name: Skip SSH port reconfiguration for default port
      ansible.builtin.debug:
        msg: "SSH port is already 22 (default), skipping reconfiguration"
      when: not needs_reconfiguration
      tags:
        - ssh
        - check

    - name: Backup current sshd_config
      ansible.builtin.copy:
        src: /etc/ssh/sshd_config
        dest: /etc/ssh/sshd_config.backup
        remote_src: yes
        backup: yes
      when: needs_reconfiguration
      tags:
        - ssh
        - config
        - backup

    - name: Configure SSH port in sshd_config
      ansible.builtin.lineinfile:
        path: /etc/ssh/sshd_config
        regexp: "^#?Port "
        line: "Port {{ ssh_port }}"
        state: present
        backup: yes
      when: needs_reconfiguration
      tags:
        - ssh
        - config

    - name: Validate SSH configuration
      ansible.builtin.command:
        cmd: sshd -t
      changed_when: false
      when: needs_reconfiguration
      tags:
        - ssh
        - validation

    - name: Restart SSH service
      ansible.builtin.systemd:
        name: ssh
        state: restarted
      when: needs_reconfiguration
      tags:
        - ssh
        - service

    - name: Wait for SSH to be available on new port
      ansible.builtin.wait_for:
        port: "{{ ssh_port }}"
        host: "{{ ansible_host }}"
        delay: 2
        timeout: 30
      delegate_to: localhost
      when: needs_reconfiguration
      tags:
        - ssh
        - validation

    - name: Verify SSH is listening on configured port
      ansible.builtin.shell:
        cmd: "ss -tlnp | grep ':{{ ssh_port }}' | grep sshd"
      register: ssh_port_check
      changed_when: false
      failed_when: ssh_port_check.rc != 0
      when: needs_reconfiguration
      # NOTE: This verification connects on the NEW port
      # The previous task waited for the port to be available
      tags:
        - ssh
        - validation
```

### 2. New Application Step: `ConfigureSshPortStep`

Create a new step at `src/application/steps/system/configure_ssh_port.rs` following the pattern of existing system configuration steps.

**Key Requirements**:

```rust
//! SSH service port configuration step
//!
//! This module provides the `ConfigureSshPortStep` which handles reconfiguration
//! of the SSH service to listen on a custom port via Ansible playbooks.
//!
//! ## Key Features
//!
//! - Reconfigures SSH service port via Ansible playbook execution
//! - Only executes when custom port is specified (skips default port 22)
//! - Validates SSH service configuration before restart
//! - Verifies connectivity on new port after restart
//! - Comprehensive error handling for service restart failures
//!
//! ## Configuration Process
//!
//! The step executes the "configure-ssh-port" Ansible playbook which handles:
//! - Backing up current SSH configuration
//! - Modifying /etc/ssh/sshd_config with new port
//! - Validating configuration syntax
//! - Restarting SSH service
//! - Verifying service is listening on new port

use std::sync::Arc;
use tracing::{info, instrument, warn};

use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step that configures SSH service port on a remote host via Ansible
pub struct ConfigureSshPortStep {
    ansible_client: Arc<AnsibleClient>,
}

impl ConfigureSshPortStep {
    #[must_use]
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute SSH port configuration
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - SSH configuration validation fails
    /// - SSH service restart fails
    /// - Port verification fails
    #[instrument(
        name = "configure_ssh_port",
        skip_all,
        fields(step_type = "system", component = "ssh", method = "ansible")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        warn!(
            step = "configure_ssh_port",
            action = "configure_sshd",
            "Reconfiguring SSH service port with variables from variables.yml"
        );

        self.ansible_client.run_playbook("configure-ssh-port", &[])?;

        info!(
            step = "configure_ssh_port",
            status = "success",
            "SSH service port configuration completed"
        );

        Ok(())
    }
}
```

### 3. Register Static Playbook in Project Generator

**CRITICAL**: The new `configure-ssh-port.yml` playbook is **static** (no `.tera` extension), so it must be explicitly registered in the Ansible project generator to be copied to the build directory.

**File to modify**: `src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs`

Add to the `copy_static_templates` method:

```rust
fn copy_static_templates(&self) -> Result<(), AnsibleError> {
    // ... existing templates ...
    self.copy_static_template("configure-ssh-port.yml")?;
    // ... rest of method ...
}
```

**Reference Documentation**: See [docs/contributing/templates.md](../docs/contributing/templates.md) section "Static Playbooks Registration".

### 4. Update Configure Command Handler - CRITICAL SEQUENCING

**⚠️ IMPORTANT**: Based on reproduction testing, the SSH port configuration must happen as the **FIRST step** in the configure command, immediately after entering the `Configuring` state. This is because:

1. The `provision` command already tried to connect on the custom port (and failed)
2. All subsequent Ansible playbooks need SSH connectivity on the configured port
3. SSH port reconfiguration requires connecting on port 22 first, then switching to the custom port

Modify `src/application/command_handlers/configure/handler.rs` to include the new step in the `execute_configuration_with_tracking` method.

**Location to modify**: As the **FIRST configuration step**, before all other steps

```rust
fn execute_configuration_with_tracking(
    environment: &Environment<Configuring>,
) -> StepResult<Environment<Configured>, ConfigureCommandHandlerError, ConfigureStep> {
    let ansible_client = Arc::new(AnsibleClient::new(environment.ansible_build_dir()));

    // CRITICAL FIRST STEP: Configure SSH port if custom port is specified
    // This must happen BEFORE any other steps because:
    // 1. Provision already failed trying to connect on custom port
    // 2. We need to reconfigure SSH while we can still connect on port 22
    // 3. After reconfiguration, all subsequent steps will connect on custom port
    let current_step = ConfigureStep::ConfigureSshPort;

    // Skip SSH port configuration in container environments or if using default port 22
    let skip_ssh_port_config = std::env::var("TORRUST_TD_SKIP_SSH_PORT_CONFIG_IN_CONTAINER")
        .map(|v| v == "true")
        .unwrap_or(false);

    if skip_ssh_port_config {
        info!(
            command = "configure",
            step = "configure_ssh_port",
            status = "skipped",
            "Skipping SSH port reconfiguration due to TORRUST_TD_SKIP_SSH_PORT_CONFIG_IN_CONTAINER"
        );
    } else {
        // NOTE: The playbook should check if port is already 22 and skip if so
        // This allows the step to be idempotent and safe for default configurations
        ConfigureSshPortStep::new(Arc::clone(&ansible_client))
            .execute()
            .map_err(|e| (e.into(), current_step))?;
    }

    // Now continue with rest of configuration steps...
    let current_step = ConfigureStep::InstallDocker;
    // This is already rendered, so we need to parse it or use environment variable
    // For now, we'll always run it - the playbook can be idempotent
    ConfigureSshPortStep::new(Arc::clone(&ansible_client))
        .execute()
        .map_err(|e| (e.into(), current_step))?;
}
```

### 5. Update ConfigureStep Enum

Add the new step to the `ConfigureStep` enum in `src/domain/environment/state.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigureStep {
    InstallDocker,
    InstallDockerCompose,
    ConfigureSecurityUpdates,
    ConfigureFirewall,
    ConfigureSshPort,          // NEW STEP
    ConfigureTrackerFirewall,
}
```

### 6. Export New Step in Module Hierarchy

Update `src/application/steps/system/mod.rs`:

```rust
pub use configure_ssh_port::ConfigureSshPortStep;
```

Update `src/application/steps/mod.rs`:

```rust
pub use system::{
    ConfigureFirewallStep,
    ConfigureSecurityUpdatesStep,
    ConfigureSshPortStep,        // NEW EXPORT
    ConfigureTrackerFirewallStep,
    WaitForCloudInitStep,
};
```

## Implementation Plan

### Phase 1: Create Ansible Playbook (1-2 hours)

- [ ] Create `templates/ansible/configure-ssh-port.yml` following the specification
- [ ] Test playbook syntax: `ansible-playbook --syntax-check configure-ssh-port.yml`
- [ ] Add comprehensive comments explaining each task
- [ ] Ensure playbook is idempotent (can run multiple times safely)
- [ ] Add proper tags for task filtering

### Phase 2: Register Playbook in Project Generator (15 minutes)

- [ ] Modify `src/infrastructure/external_tools/ansible/template/renderer/project_generator.rs`
- [ ] Add `configure-ssh-port.yml` to `copy_static_templates` method
- [ ] Verify playbook is copied to build directory during template rendering
- [ ] Test with: `cargo run -- create template --provider lxd test.json`

### Phase 3: Create Application Step (1 hour)

- [ ] Create `src/application/steps/system/configure_ssh_port.rs`
- [ ] Follow the pattern from `configure_firewall.rs`
- [ ] Use `AnsibleClient` to execute the playbook
- [ ] Add comprehensive documentation
- [ ] Add tracing instrumentation
- [ ] Export step in `system/mod.rs` and `steps/mod.rs`

### Phase 4: Update Domain State (30 minutes)

- [ ] Add `ConfigureSshPort` variant to `ConfigureStep` enum
- [ ] Verify serialization/deserialization works correctly
- [ ] Update any Display/Debug implementations if needed

### Phase 5: Integrate into Configure Command (1 hour)

- [ ] Modify `execute_configuration_with_tracking` in `handler.rs`
- [ ] Add new step after firewall configuration
- [ ] Implement conditional execution logic (skip for port 22)
- [ ] Add appropriate logging
- [ ] Handle step tracking for failure context

### Phase 6: Testing (2-3 hours)

- [ ] **Unit Tests**: Test step creation and basic functionality
- [ ] **Integration Test**: Test playbook execution in isolation
- [ ] **E2E Test - Default Port**: Verify port 22 works (step should be no-op or verify)
- [ ] **E2E Test - Custom Port**: Create environment with port 2222
  - [ ] Verify configure command succeeds
  - [ ] Verify SSH service listens on port 2222
  - [ ] Verify firewall allows port 2222
  - [ ] Verify subsequent commands (release, run) connect successfully
- [ ] **E2E Test - Container Environment**: Verify skip logic works correctly

### Phase 7: Documentation (1 hour)

- [ ] Update `docs/user-guide/configuration.md` with SSH port configuration
- [ ] Add example showing custom SSH port setup
- [ ] Document the security implications of using non-standard SSH ports
- [ ] Update `docs/deployment-overview.md` to include SSH port configuration step
- [ ] Add ADR documenting the decision to configure SSH port during configure phase

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check. Use this as your pre-review checklist before submitting the PR to minimize back-and-forth iterations.

### Quality Checks

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

### Functional Requirements

- [ ] New Ansible playbook `configure-ssh-port.yml` exists in `templates/ansible/`
- [ ] Playbook is registered in `ProjectGenerator::copy_static_templates`
- [ ] Playbook modifies `/etc/ssh/sshd_config` with correct port
- [ ] Playbook validates SSH configuration before restart
- [ ] Playbook restarts SSH service successfully
- [ ] Playbook verifies SSH is listening on new port
- [ ] New step `ConfigureSshPortStep` exists in `src/application/steps/system/`
- [ ] Step is exported in module hierarchy
- [ ] Step executes playbook using `AnsibleClient`
- [ ] `ConfigureStep` enum includes new `ConfigureSshPort` variant
- [ ] Configure command handler integrates new step correctly
- [ ] Step is placed after firewall configuration, before tracker firewall

### Conditional Execution

- [ ] Step logic respects `TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER` environment variable
- [ ] Step execution is logged with appropriate tracing instrumentation
- [ ] Step failure updates environment state with correct failed step

### Testing Requirements

- [ ] Unit tests exist for `ConfigureSshPortStep`
- [ ] E2E test with default port 22 succeeds (step should be idempotent)
- [ ] E2E test with custom port (e.g., 2222) succeeds
- [ ] E2E test verifies SSH service listens on custom port after configuration
- [ ] E2E test verifies subsequent commands connect successfully on custom port
- [ ] Container-based tests skip step correctly when environment variable is set

### Error Handling

- [ ] SSH configuration validation errors are caught and reported clearly
- [ ] SSH service restart failures are caught and reported with recovery instructions
- [ ] Port verification failures include diagnostic information
- [ ] Failed step is tracked correctly in failure context
- [ ] Error messages follow actionability principles (see [error-handling.md](../docs/contributing/error-handling.md))

### Documentation

- [ ] Step is documented with comprehensive module-level documentation
- [ ] Playbook includes clear comments explaining each task
- [ ] User guide updated with SSH port configuration examples
- [ ] Deployment overview updated with new configuration step
- [ ] ADR created documenting design decisions

## Verification Steps for Issue Reporter

Before implementation, the issue reporter should verify the problem exists:

### Manual Verification Test

1. **Create environment with custom SSH port**:

   ```bash
   # Edit envs/manual-test.json and change port to 2222
   {
     "ssh_credentials": {
       "port": 2222
       // ... other fields
     }
   }
   ```

2. **Run complete workflow**:

   ```bash
   cargo run -- create environment --env-file envs/manual-test.json
   cargo run -- provision manual-test
   cargo run -- configure manual-test
   ```

3. **Verify SSH service configuration**:

   ```bash
   # SSH into instance using port 22 (should still work)
   ssh -i fixtures/testing_rsa torrust@<instance-ip> -p 22

   # Check sshd_config
   cat /etc/ssh/sshd_config | grep "^Port"
   # Expected: Port 22 (unchanged)

   # Check what port SSH is listening on
   ss -tlnp | grep sshd
   # Expected: Only shows port 22
   ```

4. **Expected Result**:
   - ❌ SSH service still listening on port 22
   - ❌ Firewall configured for port 2222
   - ❌ Next command (e.g., `release`) will fail with connection timeout

### Expected Behavior After Fix

After implementing this issue:

1. SSH service will be reconfigured to listen on port 2222
2. Firewall will allow port 2222
3. All subsequent commands will connect successfully on port 2222
4. Manual SSH connections require: `ssh -i key torrust@<ip> -p 2222`

## Related Documentation

- **Architecture**: [docs/codebase-architecture.md](../docs/codebase-architecture.md) - DDD layers and three-level pattern
- **Template System**: [docs/contributing/templates.md](../docs/contributing/templates.md) - Static playbook registration
- **Error Handling**: [docs/contributing/error-handling.md](../docs/contributing/error-handling.md) - Actionable error principles
- **E2E Testing**: [docs/e2e-testing/manual-testing.md](../docs/e2e-testing/manual-testing.md) - Manual test procedures
- **Existing Steps**:
  - `src/application/steps/system/configure_firewall.rs` - Similar pattern to follow
  - `src/application/steps/system/configure_security_updates.rs` - System config example
- **Ansible Integration**: `src/adapters/ansible/client.rs` - How to execute playbooks

## Notes

### Security Considerations

- **Non-standard SSH ports** provide security through obscurity but are not a replacement for proper authentication
- **Port scanning** attackers will still find the SSH service on non-standard ports
- **Benefits**: Reduces automated attacks and log noise from port 22 scanners
- **Recommendation**: Always use key-based authentication regardless of port

### Design Decisions

1. **Why configure port during `configure` phase, not `provision`?**

   - Provisioning uses cloud-init which is provider-dependent
   - Configuration uses Ansible which is consistent across all providers
   - Ansible provides better error handling and validation
   - Separation of concerns: provision creates infrastructure, configure sets it up

2. **Why skip for port 22?**

   - Avoids unnecessary SSH service restarts
   - Reduces risk of misconfiguration on default setup
   - Improves performance for most common use case

3. **Why place step after firewall configuration?**

   - Ensures firewall allows new port before SSH moves to it
   - Prevents potential lockout scenarios
   - Logical ordering: network rules before service configuration

4. **Why verify connectivity after port change?**
   - Provides immediate feedback if configuration failed
   - Prevents silent failures that break subsequent commands
   - Follows observability principle

### Implementation Alternatives Considered

1. **Configure via cloud-init**: Rejected because it's provider-specific and harder to validate
2. **Configure during provisioning**: Rejected to maintain clear phase separation
3. **Always execute step**: Rejected to avoid unnecessary service restarts for default port
4. **Manual SSH port change**: Rejected as it breaks automation principle

### Testing Strategy

The fix should be validated at three levels:

1. **Unit Tests**: Step creation and basic contract
2. **Integration Tests**: Ansible playbook execution in isolation
3. **E2E Tests**: Complete workflow with both default and custom ports

The E2E tests are critical because they verify the entire chain:

- Template rendering (variables.yml gets correct port)
- Playbook execution (sshd_config is modified)
- Service restart (SSH listens on new port)
- Firewall rules (new port is open)
- Subsequent connectivity (commands use new port)
