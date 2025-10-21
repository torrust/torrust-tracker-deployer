# Configure UFW Firewall

**Issue**: #18
**Parent Epic**: #16 - Finish ConfigureCommand - System Security Configuration
**Depends On**: #17 - Configure Automatic Security Updates
**Related**: [Parent Epic](./16-epic-finish-configure-command-system-security.md), [Security Updates Task](./17-configure-automatic-security-updates.md)

## Overview

Implement UFW (Uncomplicated Firewall) configuration in the `ConfigureCommand`. This task adds a new step that safely configures a restrictive firewall while maintaining SSH access to prevent lockout situations.

This is the second phase of completing system security configuration, implemented after automatic security updates because it has higher risk due to potential SSH lockout if not configured correctly.

## Goals

- [ ] **Safe Firewall Configuration**: Configure UFW with restrictive policies while preserving SSH access
- [ ] **SSH Port Awareness**: Use configurable SSH port from `user_inputs.ssh_port`
- [ ] **New Domain Step**: Add `ConfigureFirewall` to the `ConfigureStep` enum
- [ ] **Ansible Integration**: Create new Ansible playbook for firewall configuration
- [ ] **Lockout Prevention**: Implement safeguards to prevent SSH access loss
- [ ] **Testing**: Ensure E2E tests validate firewall configuration without losing access

## Specifications

### Domain Integration

Update `ConfigureStep` enum in `src/domain/environment/state/configure_failed.rs`:

```rust
/// Steps in the configure workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigureStep {
    /// Installing Docker
    InstallDocker,
    /// Installing Docker Compose
    InstallDockerCompose,
    /// Configuring automatic security updates
    ConfigureSecurityUpdates,
    /// Configuring UFW firewall
    ConfigureFirewall,
}
```

### New Application Step

Create `src/application/steps/system/configure_firewall.rs`:

```rust
use std::sync::Arc;
use tracing::{info, instrument, warn};
use crate::adapters::ansible::AnsibleClient;
use crate::domain::environment::Environment;
use crate::shared::command::CommandError;

/// Step to configure UFW firewall with safe SSH access preservation
///
/// This step configures a restrictive UFW firewall policy while ensuring
/// SSH access is maintained. The SSH port is resolved during template rendering
/// and embedded in the final Ansible playbook. The configuration follows the
/// principle of "allow SSH BEFORE enabling firewall" to prevent lockout.
pub struct ConfigureFirewallStep {
    ansible_client: Arc<AnsibleClient>,
}

impl ConfigureFirewallStep {
    /// Create a new firewall configuration step
    ///
    /// # Arguments
    ///
    /// * `ansible_client` - Ansible client for running playbooks
    ///
    /// # Note
    ///
    /// SSH port configuration is resolved during template rendering phase,
    /// not at step execution time. The rendered playbook contains the
    /// resolved SSH port value.
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }    /// Execute the firewall configuration
    ///
    /// # Safety
    ///
    /// This method is designed to prevent SSH lockout by:
    /// 1. Resetting UFW to clean state
    /// 2. Allowing SSH access BEFORE enabling firewall
    /// 3. Using the correct SSH port from user configuration
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - UFW commands fail
    /// - SSH rules cannot be applied
    #[instrument(
        name = "configure_firewall_step",
        skip_all,
        fields(step_type = "system")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        warn!("Configuring UFW firewall - CRITICAL: SSH access will be restricted to configured port");

        // Run Ansible playbook (SSH port already resolved during template rendering)
        self.ansible_client.run_playbook("configure-firewall.yml")?;

        info!("UFW firewall configured successfully with SSH access preserved");
        Ok(())
    }
}
```

### Ansible Playbook

Create `templates/ansible/configure-firewall.yml.tera` (Tera template for SSH port resolution):

```yaml
---
# Configure UFW Firewall with Safe SSH Access
# This playbook configures UFW with restrictive policies while preserving SSH access.
# CRITICAL: SSH access is allowed BEFORE enabling firewall to prevent lockout.

- name: Configure UFW firewall safely
  hosts: torrust_servers
  become: yes
  gather_facts: yes

  tasks:
    - name: Install UFW (should already be present on Ubuntu)
      ansible.builtin.apt:
        name: ufw
        state: present
        update_cache: yes
      tags:
        - security
        - firewall
        - packages

    - name: Reset UFW to clean state
      community.general.ufw:
        state: reset
      tags:
        - security
        - firewall
        - reset

    - name: Set UFW default policy - deny incoming
      community.general.ufw:
        default: deny
        direction: incoming
      tags:
        - security
        - firewall
        - policy

    - name: Set UFW default policy - allow outgoing
      community.general.ufw:
        default: allow
        direction: outgoing
      tags:
        - security
        - firewall
        - policy

    # CRITICAL: Allow SSH BEFORE enabling firewall to prevent lockout
    - name: Allow SSH access on configured port (BEFORE enabling firewall)
      community.general.ufw:
        rule: allow
        port: "{{ ssh_port }}"
        proto: tcp
        comment: "SSH access (configured port {{ ssh_port }})"
      tags:
        - security
        - firewall
        - ssh

    - name: Allow SSH service by name (additional safety measure)
      community.general.ufw:
        rule: allow
        name: ssh
        comment: "SSH service (standard SSH)"
      tags:
        - security
        - firewall
        - ssh

    - name: Enable UFW firewall (AFTER SSH rules are in place)
      community.general.ufw:
        state: enabled
      tags:
        - security
        - firewall
        - enable

    - name: Verify UFW status
      ansible.builtin.command:
        cmd: ufw status numbered
      register: ufw_status
      changed_when: false
      tags:
        - security
        - firewall
        - verification

    - name: Display UFW status
      ansible.builtin.debug:
        var: ufw_status.stdout_lines
      tags:
        - security
        - firewall
        - verification

    - name: Verify SSH port is allowed
      ansible.builtin.shell:
        cmd: "ufw status | grep -E '{{ ssh_port }}/tcp.*ALLOW'"
      register: ssh_port_check
      changed_when: false
      failed_when: ssh_port_check.rc != 0
      tags:
        - security
        - firewall
        - verification
        - ssh

    - name: Confirm firewall configuration complete
      ansible.builtin.debug:
        msg:
          - "UFW firewall configured successfully"
          - "SSH access preserved on port {{ ssh_port }}"
          - "Default policy: deny incoming, allow outgoing"
          - "Active rules protect against unauthorized access"
      tags:
        - security
        - firewall
```

### ConfigureCommand Integration

Update `src/application/commands/configure.rs` to include the new step:

```rust
// In execute_configuration_with_tracking method, add after ConfigureSecurityUpdates:

let current_step = ConfigureStep::ConfigureFirewall;
ConfigureFirewallStep::new(Arc::clone(&self.ansible_client))
    .execute()
    .map_err(|e| (e.into(), current_step))?;
```

### Module Export

Update `src/application/steps/system/mod.rs`:

```rust
pub use configure_firewall::ConfigureFirewallStep;
pub use configure_security_updates::ConfigureSecurityUpdatesStep;

mod configure_firewall;
mod configure_security_updates;
```

### Template Rendering Integration

The SSH port will be resolved during the template rendering phase when the environment is being configured. The `ConfigureFirewallStep` works with the already-rendered Ansible playbook that contains the resolved SSH port value.

Template rendering process:

1. `templates/ansible/configure-firewall.yml.tera` (contains `{{ ssh_port }}`)
2. Template rendering resolves `{{ ssh_port }}` with actual value (e.g., `22`)
3. `build/{env}/ansible/configure-firewall.yml` (contains resolved value `port: "22"`)
4. `ConfigureFirewallStep` executes the resolved playbook

## Implementation Plan

This task should be implemented as a **single PR** with the following subtasks:

### Subtask 1: Prerequisites and Safety Measures (0.5 days)

- [ ] **Update ConfigureStep enum**: Add `ConfigureFirewall` variant in domain
- [ ] **Create safety documentation**: Document lockout prevention measures
- [ ] **Verify template rendering**: Ensure SSH port template resolution works correctly

### Subtask 2: Ansible Playbook Implementation (1 day)

- [ ] **Create firewall template**: Implement `templates/ansible/configure-firewall.yml.tera`
- [ ] **SSH port templating**: Ensure SSH port variable is properly templated
- [ ] **Safety sequence**: Implement correct order (allow SSH BEFORE enable firewall)
- [ ] **Verification tasks**: Add tasks to verify firewall status and SSH access
- [ ] **Test playbook syntax**: Validate Ansible playbook syntax

### Subtask 3: Application Step Implementation (0.5 days)

- [ ] **Create step module**: Implement `src/application/steps/system/configure_firewall.rs`
- [ ] **SSH port integration**: Accept and use SSH port from user inputs
- [ ] **Safety logging**: Add warning logs about firewall configuration
- [ ] **Error handling**: Implement comprehensive error handling
- [ ] **Unit tests**: Write unit tests for the firewall step

### Subtask 4: ConfigureCommand Integration (0.5 days)

- [ ] **Add step to workflow**: Integrate firewall step into configure command
- [ ] **Step ordering**: Ensure firewall comes after security updates
- [ ] **Error context**: Proper error mapping and failure context
- [ ] **Integration tests**: Test command integration with new step

### Subtask 5: End-to-End Validation and Testing (1 day)

- [ ] **Controlled E2E testing**: Test firewall configuration in safe environment
- [ ] **SSH connectivity validation**: Verify SSH access is maintained throughout process
- [ ] **Firewall rule verification**: Confirm correct rules are applied
- [ ] **Rollback testing**: Test recovery procedures if configuration fails
- [ ] **Documentation**: Document testing procedures and safety measures

## Acceptance Criteria

- [ ] **Domain Updated**: `ConfigureStep::ConfigureFirewall` enum variant added
- [ ] **Step Implemented**: `ConfigureFirewallStep` executes without SSH lockout
- [ ] **Template Rendering**: SSH port properly resolved in final Ansible playbook
- [ ] **Firewall Active**: UFW is enabled with restrictive default policies
- [ ] **SSH Preserved**: SSH access maintained on configured port throughout process
- [ ] **Port Configuration**: Uses actual SSH port from `user_inputs.ssh_port`
- [ ] **Safety Verified**: Firewall configuration cannot cause SSH lockout
- [ ] **Rules Correct**: Only SSH port is allowed, all other incoming traffic denied
- [ ] **Tests Pass**: All existing tests continue to pass with firewall enabled
- [ ] **E2E Validation**: Full deployment includes working firewall configuration
- [ ] **Manual Verification**: Can verify firewall rules with `sudo ufw status numbered`

## Related Documentation

- [Original Bash PoC Firewall Configuration](https://github.com/torrust/torrust-tracker-deploy-bash-poc/blob/main/infrastructure/cloud-init/user-data.yaml.tpl#L235-L244)
- [ConfigureCommand Implementation](../../src/application/commands/configure.rs)
- [Domain UserInputs SSH Port](../../src/domain/environment/user_inputs.rs)
- [Error Handling Guide](../contributing/error-handling.md)
- [UFW Documentation](https://help.ubuntu.com/community/UFW)

## Safety and Risk Mitigation

### SSH Lockout Prevention

This is the highest risk aspect of this implementation. Safety measures include:

1. **Correct Order**: SSH rules are added BEFORE enabling firewall
2. **Multiple SSH Rules**: Both port-specific and service-name rules are added
3. **Port Configuration**: Uses actual SSH port from user inputs, not hardcoded values
4. **Verification Steps**: Ansible tasks verify SSH access is preserved
5. **Comprehensive Logging**: Detailed logging of each firewall configuration step

### Testing Strategy

1. **Isolated Testing**: Test in disposable VM environments first
2. **SSH Monitoring**: Monitor SSH connectivity throughout configuration process
3. **Rollback Plan**: Document manual recovery procedures if lockout occurs
4. **E2E Validation**: Verify entire workflow maintains connectivity

### Error Scenarios

- **UFW Installation Failure**: Clear error message with package installation instructions
- **Rule Application Failure**: Detailed error context about which rule failed
- **SSH Rule Verification Failure**: Actionable guidance about manual rule addition
- **Firewall Enable Failure**: Instructions for manual firewall management

### Recovery Procedures

If SSH lockout occurs during development/testing:

1. **Console Access**: Use provider console/VNC to access instance
2. **Disable Firewall**: `sudo ufw disable` to restore access
3. **Check Rules**: `sudo ufw status numbered` to see current rules
4. **Manual Fix**: Add SSH rule manually: `sudo ufw allow 22/tcp`
5. **Re-enable**: `sudo ufw enable` after fixing rules

## Notes

### Implementation Rationale

This task is implemented second because:

1. **Higher Risk**: Firewall misconfiguration can cause SSH lockout
2. **Dependency**: Builds on patterns established in security updates task
3. **Incremental Value**: Provides additional security layer after basic patching
4. **Testing Complexity**: Requires more careful testing and validation

### SSH Port Considerations

- **Current Default**: SSH port defaults to 22 in current implementation
- **Future Customization**: Infrastructure exists to support custom SSH ports
- **Template Variables**: Ansible templates already support `ansible_port` variable
- **Port Changes**: Future SSH port customization would require SSH daemon reconfiguration

### Future Enhancements

- **Service-Specific Rules**: Add firewall rules as new services are implemented
- **Dynamic Port Management**: Automatically manage firewall rules for new services
- **Fail2ban Integration**: Could add fail2ban for intrusion detection
- **Monitoring**: Could add monitoring for firewall rule violations

### Alternative Ansible Variables Approach (Future Optimization)

Similar to OpenTofu's `variables.tfvars.tera` pattern, Ansible could use a centralized variables file:

```yaml
# templates/ansible/variables.yml.tera → build/{env}/ansible/variables.yml
ssh_port: {{ ssh_port }}
# other variables...

# In playbooks (no .tera extension needed):
- hosts: all
  vars_files:
    - variables.yml
  tasks:
    - ufw: port="{{ ssh_port }}"  # Ansible resolves from variables.yml
```

**Benefits**: Reduces Tera template complexity, only one file needs variable resolution
**Consideration**: Could be explored in future issue to reduce boilerplate code
