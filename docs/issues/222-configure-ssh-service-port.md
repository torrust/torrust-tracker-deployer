# Configure SSH Service Port During VM Provisioning

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

**Result**: The `provision` command fails during the `WaitForCloudInitStep` because Ansible cannot connect to the instance on the configured custom port (the playbook uses the inventory which specifies the custom port, but SSH is still on port 22). The deployment cannot proceed beyond provisioning.

## Solution Implemented: Cloud-Init SSH Port Configuration with Reboot

After extensive analysis and testing, we determined the best solution is to **configure the SSH port via cloud-init during VM initialization with a system reboot**, following Hetzner's cloud-config best practices. This approach:

- ✅ Configures SSH port BEFORE any SSH connections are attempted
- ✅ Ensures clean SSH restart with no lingering processes on port 22
- ✅ No special connection handling or port overrides needed
- ✅ Works seamlessly with both `WaitForSSHConnectivityStep` and `WaitForCloudInitStep`
- ✅ Simpler and more reliable than post-provisioning reconfiguration or manual service management

### Implementation Overview

**Phase**: `provision` (VM initialization)
**Mechanism**: Cloud-init `write_files` + `runcmd` with reboot
**Component**: `templates/tofu/common/cloud-init.yml.tera`

The SSH port is configured by:

1. **During Template Rendering** (before VM creation):

   - `TofuProjectGenerator` accepts `ssh_port` from environment configuration
   - `CloudInitRenderer` receives the SSH port value
   - `CloudInitContext` includes the port in template context
   - `cloud-init.yml.tera` renders with the configured port value

2. **During VM Initialization** (first boot):

   - Cloud-init creates `/etc/ssh/sshd_config.d/99-custom-port.conf` with the port setting
   - Cloud-init triggers system reboot via `runcmd: [reboot]`
   - System reboots, SSH service starts cleanly with new configuration
   - This happens BEFORE any Ansible connection attempts

3. **During SSH Connectivity Checks**:
   - Provision handler's `wait_for_readiness()` uses the configured custom port (not default port 22)
   - SSH connectivity timeout increased to 120 seconds to account for cloud-init + reboot time (~70-80s)
   - Both `WaitForSSHConnectivityStep` and `WaitForCloudInitStep` connect using the custom port
   - SSH service is listening only on the correct port - connection succeeds

### Critical Implementation Details

#### Cloud-Init Reboot Pattern

The cloud-init template uses the **reboot pattern** as documented in [Hetzner's cloud-config tutorial](https://community.hetzner.com/tutorials/basic-cloud-config):

```yaml
write_files:
  - path: /etc/ssh/sshd_config.d/99-custom-port.conf
    content: |
      # Custom SSH port configuration
      Port {{ ssh_port }}
    permissions: "0644"
    owner: root:root

runcmd:
  # Reboot to apply SSH port configuration
  # The reboot ensures SSH service fully restarts with the new port from write_files
  # This is the recommended approach per Hetzner cloud-config best practices
  - reboot
```

**Why reboot?** Three critical reasons (from Hetzner documentation):

1. Package updates may require reboot for patches to work properly
2. Service configurations (like SSH port changes) are applied cleanly
3. System starts in a consistent state with all configurations active

**Why not `systemctl restart ssh`?** Testing revealed multiple issues:

- `systemctl restart` doesn't kill the old SSH process when the port changes
- Results in SSH listening on **both** port 22 (old PID) and custom port (new PID)
- Cloud-init's runcmd execution of `systemctl restart ssh` often completes without actually restarting SSH
- systemd automatically re-enables and starts SSH after bootcmd, making bootcmd-based approaches ineffective

#### Provision Handler Port Configuration

Two critical fixes were required in the provision handler:

1. **Use Configured Port** (`src/application/command_handlers/provision/handler.rs`):

   ```rust

   // Before: Always waited for default port 22
   let ssh_config = SshConfig::with_default_port(instance_ip);

   // After: Wait for configured custom port
   let ssh_port = environment.ssh_port();
   let ssh_config = SshConfig::new(SocketAddr::new(instance_ip, ssh_port));

   ```

2. **Increase Timeout** (`src/adapters/ssh/config.rs`):

   ```rust

   // Changed from 30 to 60 attempts (120 seconds total)
   // Accounts for cloud-init completion (~70-80s) + reboot time
   pub const DEFAULT_MAX_RETRY_ATTEMPTS: u32 = 60;

   ```

### Files Modified

#### Template Files

- **`templates/tofu/common/cloud-init.yml.tera`**: Added `write_files` section to create SSH port configuration file, and `runcmd: [reboot]` to trigger system reboot for clean SSH restart

#### Infrastructure Layer (DDD)

- **`src/infrastructure/templating/tofu/template/common/wrappers/cloud_init/context.rs`**: Added `ssh_port: u16` field to `CloudInitContext` with builder method
- **`src/infrastructure/templating/tofu/template/common/renderer/cloud_init.rs`**: Updated `CloudInitRenderer` to accept and use `ssh_port` parameter
- **`src/infrastructure/templating/tofu/template/common/renderer/project_generator.rs`**: Updated `TofuProjectGenerator` constructor to accept `ssh_port` parameter

#### Application Layer (DDD)

- **`src/application/command_handlers/provision/handler.rs`**: Updated to pass `ssh_port` from environment to `TofuProjectGenerator` and to `wait_for_readiness()`, changed to use `SocketAddr::new(ip, ssh_port)` instead of `SshConfig::with_default_port()`

#### Adapters Layer

- **`src/adapters/ssh/config.rs`**: Increased `DEFAULT_MAX_RETRY_ATTEMPTS` from 30 to 60 (120 seconds total timeout) to account for cloud-init completion and reboot time

### Why We Discarded Alternative Approaches

#### Alternative 1: Ansible-Based Approach (Configure Phase)

**Initial Plan**: Use an Ansible playbook in the `configure` phase to reconfigure SSH port after provisioning.

**Why It Was Discarded**:

1. **Timing Problem**: The provision phase already fails before reaching the configure phase

   - `WaitForCloudInitStep` (in provision) uses Ansible with the custom port from inventory
   - But SSH service is still on port 22 at this point
   - Connection fails, provision never completes

2. **Complexity**: Would require special handling:

   - Override Ansible connection port to 22 for this one playbook
   - Disconnect and reconnect after port change
   - Additional error handling for port transition
   - More moving parts and potential failure points

3. **Architectural Mismatch**: SSH port is infrastructure configuration, not application configuration
   - Should be set during VM initialization (provision phase)
   - Not during application setup (configure phase)
   - Cloud-init is the proper tool for initial system configuration

#### Alternative 2: systemctl restart Without Reboot

**Approach**: Use cloud-init `runcmd` to execute `systemctl restart ssh` without full system reboot.

**Why It Was Discarded**:

- Testing revealed `systemctl restart ssh` doesn't kill the old SSH process when port changes
- Results in SSH listening on **both** port 22 (old PID) and custom port (new PID)
- Cloud-init runcmd execution often completes without SSH actually restarting
- Multiple test attempts confirmed SSH remained on port 22 after cloud-init reported "completion"

#### Alternative 3: bootcmd disable + runcmd restart

**Approach**: Use `bootcmd` to disable SSH before it auto-starts, then use `runcmd` to restart it with new config.

**Why It Was Discarded**:

- systemd automatically re-enables and starts SSH approximately 3 seconds after bootcmd disables it
- Testing showed SSH started at 19:19:51 despite bootcmd completing at 19:19:48
- systemd service management overrides cloud-init's bootcmd attempts

#### Alternative 4: pkill + systemctl start

**Approach**: Kill SSH processes with `pkill -9 sshd`, then start fresh with `systemctl start ssh`.

**Why It Was Discarded**:

- Non-standard approach, violates best practices for service management
- More brittle than clean system reboot
- No industry precedent or documentation for this pattern

**Conclusion**: The cloud-init with reboot approach is the cleanest, most reliable, and follows industry best practices (Hetzner). It configures infrastructure settings during infrastructure provisioning with a guaranteed clean service restart.

## Acceptance Criteria

- ✅ SSH service on remote instance listens on the port specified in `environment.ssh_credentials.port`
- ✅ Cloud-init template correctly renders with SSH port configuration
- ✅ SSH connectivity checks succeed using the custom port
- ✅ Ansible playbooks can connect using the custom port
- ✅ Firewall rules continue to work with custom ports
- ✅ E2E tests pass with custom SSH port (e.g., 2222)
- ✅ Default behavior (port 22) still works when not explicitly configured

## Testing Strategy

### Unit Tests

- ✅ `CloudInitContext` builder accepts and stores `ssh_port` value
- ✅ `CloudInitRenderer` passes `ssh_port` to context correctly
- ✅ `TofuProjectGenerator` accepts and propagates `ssh_port` parameter

### Integration Tests

- Cloud-init template renders with correct SSH port configuration
- Generated configuration file creates proper `sshd_config.d` override
- SSH service restart command is included in cloud-init

### E2E Tests

- Full workflow test with custom SSH port (e.g., 2222)
- Verify provision completes successfully
- Verify SSH connectivity using custom port
- Verify Ansible playbooks execute successfully

## Technical Notes

### Cloud-Init Configuration Format

The cloud-init template uses `write_files` + `reboot` pattern following Hetzner best practices:

```yaml
write_files:
  - path: /etc/ssh/sshd_config.d/99-custom-port.conf
    content: |
      # Custom SSH port configuration
      Port {{ ssh_port }}
    permissions: "0644"
    owner: root:root

runcmd:
  # Reboot to apply SSH port configuration
  # The reboot ensures SSH service fully restarts with the new port from write_files
  # This is the recommended approach per Hetzner cloud-config best practices
  - reboot
```

This approach:

- Uses Ubuntu's drop-in configuration directory pattern
- Avoids modifying the main `/etc/ssh/sshd_config` file
- Ensures clean SSH restart via system reboot (no lingering processes on port 22)
- Follows industry best practices documented by Hetzner
- Simpler than manual service lifecycle management

### Provision Handler Timeout Considerations

The provision handler waits up to **120 seconds** (60 attempts × 2 seconds) for SSH connectivity:

- Cloud-init completion takes approximately 70-80 seconds
- System reboot adds approximately 10-20 seconds
- Total time typically 80-100 seconds
- 120-second timeout provides sufficient buffer

This timeout increase (from the previous 60 seconds) ensures reliable provisioning with custom SSH ports.

- Overrides the default port without modifying main config
- Takes effect immediately after service restart
- Works on all Ubuntu versions with systemd

### Builder Pattern Usage

The `CloudInitContext` uses the builder pattern for clean construction:

```rust
CloudInitContextBuilder::new()
    .with_hostname(hostname)
    .with_ssh_port(ssh_port)  // ← New method
    .with_ssh_keys(ssh_keys)
    .build()
```

Default value is `22` if not explicitly configured.

## Follow-up Tasks

1. Consider adding validation for SSH port range (1024-65535 recommended)
2. Update user documentation about SSH port configuration
3. Consider adding SSH port to environment JSON schema validation

## Related Documentation

- [Cloud-Init Documentation](https://cloudinit.readthedocs.io/)
- [Ubuntu SSH Configuration](https://ubuntu.com/server/docs/service-openssh)
- [Ansible Connection Configuration](https://docs.ansible.com/ansible/latest/user_guide/intro_inventory.html)
