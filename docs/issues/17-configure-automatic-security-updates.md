# Configure Automatic Security Updates

**Issue**: #17
**Parent Epic**: #16 - Finish ConfigureCommand - System Security Configuration
**Related**: [Parent Epic](./16-epic-finish-configure-command-system-security.md), [ConfigureCommand](../../src/application/commands/configure.rs)

## Overview

Implement automatic security updates configuration in the `ConfigureCommand`. This task adds a new step that configures unattended-upgrades on provisioned instances to ensure they automatically receive and install security patches with scheduled reboots.

This is the first phase of completing the system security configuration, chosen because it has lower implementation risk and provides immediate security value.

## Goals

- [ ] **Automatic Security Updates**: Configure unattended-upgrades for automatic security patching
- [ ] **Scheduled Reboots**: Enable automatic reboots at 2:00 AM for security updates that require restart
- [ ] **New Domain Step**: Add `ConfigureSecurityUpdates` to the `ConfigureStep` enum
- [ ] **Ansible Integration**: Create new Ansible playbook for security updates configuration
- [ ] **Error Handling**: Implement proper error handling with actionable messages
- [ ] **Testing**: Ensure E2E tests validate the security updates configuration

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
}
```

### New Application Step

Create `src/application/steps/system/configure_security_updates.rs`:

```rust
use std::sync::Arc;
use tracing::{info, instrument};
use crate::adapters::ansible::AnsibleClient;
use crate::shared::command::CommandError;

/// Step to configure automatic security updates using unattended-upgrades
///
/// This step configures the system to automatically install security updates
/// and reboot at 2:00 AM when necessary. It uses unattended-upgrades package
/// which is the standard Ubuntu/Debian solution for automatic updates.
pub struct ConfigureSecurityUpdatesStep {
    ansible_client: Arc<AnsibleClient>,
}

impl ConfigureSecurityUpdatesStep {
    /// Create a new security updates configuration step
    pub fn new(ansible_client: Arc<AnsibleClient>) -> Self {
        Self { ansible_client }
    }

    /// Execute the security updates configuration
    ///
    /// # Errors
    ///
    /// Returns `CommandError` if:
    /// - Ansible playbook execution fails
    /// - Package installation fails
    /// - Service configuration fails
    #[instrument(
        name = "configure_security_updates_step",
        skip_all,
        fields(step_type = "system")
    )]
    pub fn execute(&self) -> Result<(), CommandError> {
        info!("Configuring automatic security updates with unattended-upgrades");

        self.ansible_client.run_playbook("configure-security-updates.yml")?;

        info!("Automatic security updates configured successfully");
        Ok(())
    }
}
```

### Ansible Playbook

Create `templates/ansible/configure-security-updates.yml` (static template, no variables needed):

```yaml
---
# Configure Automatic Security Updates
# This playbook configures unattended-upgrades for automatic security patching
# with scheduled reboots at 2:00 AM when updates require restart.

- name: Configure automatic security updates
  hosts: torrust_servers
  become: yes
  gather_facts: yes

  tasks:
    - name: Install unattended-upgrades package
      ansible.builtin.apt:
        name: unattended-upgrades
        state: present
        update_cache: yes
      tags:
        - security
        - updates
        - packages

    - name: Enable automatic security updates
      ansible.builtin.lineinfile:
        path: /etc/apt/apt.conf.d/20auto-upgrades
        regexp: "^APT::Periodic::Unattended-Upgrade"
        line: 'APT::Periodic::Unattended-Upgrade "1";'
        create: yes
        backup: yes
      tags:
        - security
        - updates
        - config

    - name: Enable automatic reboot for security updates
      ansible.builtin.lineinfile:
        path: /etc/apt/apt.conf.d/50unattended-upgrades
        regexp: "^Unattended-Upgrade::Automatic-Reboot"
        line: 'Unattended-Upgrade::Automatic-Reboot "true";'
        backup: yes
      tags:
        - security
        - updates
        - config

    - name: Set automatic reboot time to 2:00 AM
      ansible.builtin.lineinfile:
        path: /etc/apt/apt.conf.d/50unattended-upgrades
        regexp: "^Unattended-Upgrade::Automatic-Reboot-Time"
        line: 'Unattended-Upgrade::Automatic-Reboot-Time "02:00";'
        backup: yes
      tags:
        - security
        - updates
        - config

    - name: Enable and start unattended-upgrades service
      ansible.builtin.systemd:
        name: unattended-upgrades
        enabled: yes
        state: started
      tags:
        - security
        - updates
        - service

    - name: Verify unattended-upgrades configuration
      ansible.builtin.command:
        cmd: unattended-upgrade --dry-run
      register: unattended_upgrades_test
      changed_when: false
      failed_when: unattended_upgrades_test.rc != 0
      tags:
        - security
        - updates
        - verification

    - name: Display unattended-upgrades status
      ansible.builtin.debug:
        msg: "Unattended-upgrades configured successfully and running"
      tags:
        - security
        - updates
```

### ConfigureCommand Integration

Update `src/application/commands/configure.rs` to include the new step:

```rust
// In execute_configuration_with_tracking method, add after InstallDockerCompose:

let current_step = ConfigureStep::ConfigureSecurityUpdates;
ConfigureSecurityUpdatesStep::new(Arc::clone(&self.ansible_client))
    .execute()
    .map_err(|e| (e.into(), current_step))?;
```

### Module Export

Update `src/application/steps/system/mod.rs`:

```rust
pub use configure_security_updates::ConfigureSecurityUpdatesStep;

mod configure_security_updates;
```

## Implementation Plan

This task should be implemented as a **single PR** with the following subtasks:

### Subtask 1: Domain and Infrastructure (1 day)

- [ ] **Update ConfigureStep enum**: Add `ConfigureSecurityUpdates` variant in `src/domain/environment/state/configure_failed.rs`
- [ ] **Create Ansible template**: Implement `templates/ansible/configure-security-updates.yml` with all required tasks
- [ ] **Test template syntax**: Verify Ansible playbook syntax is valid
- [ ] **Template integration**: Ensure template is properly embedded and accessible

### Subtask 2: Application Step Implementation (1 day)

- [ ] **Create step module**: Implement `src/application/steps/system/configure_security_updates.rs`
- [ ] **Step execution logic**: Add proper error handling and logging
- [ ] **Module exports**: Update `src/application/steps/system/mod.rs`
- [ ] **Unit tests**: Write comprehensive unit tests for the new step

### Subtask 3: ConfigureCommand Integration (0.5 days)

- [ ] **Add step to workflow**: Integrate new step into `ConfigureCommand.execute_configuration_with_tracking()`
- [ ] **Error handling**: Ensure proper error mapping and context building
- [ ] **Step ordering**: Place security updates step after Docker installation
- [ ] **Integration tests**: Verify command integration works correctly

### Subtask 4: End-to-End Validation (0.5 days)

- [ ] **E2E test validation**: Run full E2E tests to ensure security updates are configured
- [ ] **Manual verification**: SSH into test instance and verify unattended-upgrades status
- [ ] **Configuration verification**: Confirm automatic reboot settings are applied
- [ ] **Service status check**: Verify unattended-upgrades service is running

## Acceptance Criteria

- [ ] **Domain Updated**: `ConfigureStep::ConfigureSecurityUpdates` enum variant added and properly serializable
- [ ] **Step Implemented**: `ConfigureSecurityUpdatesStep` executes without errors
- [ ] **Ansible Playbook**: Template runs successfully and configures all security update settings
- [ ] **Service Running**: unattended-upgrades service is enabled and active on configured instances
- [ ] **Automatic Reboots**: System configured to reboot at 2:00 AM for security updates
- [ ] **Error Handling**: Clear, actionable error messages for all failure scenarios
- [ ] **Tests Pass**: All existing tests continue to pass
- [ ] **E2E Validation**: Full deployment workflow includes security updates configuration
- [ ] **Verification**: Can manually verify configuration with `systemctl status unattended-upgrades`

## Related Documentation

- [Original Bash PoC Security Configuration](https://github.com/torrust/torrust-tracker-deploy-bash-poc/blob/main/infrastructure/cloud-init/user-data.yaml.tpl#L226-L234)
- [ConfigureCommand Implementation](../../src/application/commands/configure.rs)
- [Domain ConfigureStep enum](../../src/domain/environment/state/configure_failed.rs)
- [Error Handling Guide](../contributing/error-handling.md)
- [Ubuntu Unattended Upgrades Documentation](https://help.ubuntu.com/community/AutomaticSecurityUpdates)

## Notes

### Security Considerations

- **Automatic Reboots**: Reboots are scheduled for 2:00 AM to minimize service disruption
- **Security Only**: Only security updates are installed automatically (not all packages)
- **Backup Configuration**: Configuration files are backed up before modification
- **Service Verification**: Playbook includes verification step to ensure configuration is valid

### Implementation Rationale

This task is implemented first because:

1. **Lower Risk**: Security updates don't affect network access (unlike firewall configuration)
2. **Immediate Value**: Provides security benefits immediately after configuration
3. **Foundation**: Establishes patterns for system configuration steps
4. **Testing**: Easier to test and verify without risk of losing SSH access

### Future Enhancements

- **Notification Configuration**: Could add email notifications for update status
- **Maintenance Windows**: Could make reboot time configurable via user inputs
- **Update Policies**: Could add more granular control over which packages to update
- **Logging**: Could configure more detailed logging for update activities
