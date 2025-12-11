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

## Solution Implemented: Cloud-Init SSH Port Configuration

After analysis, we determined the best solution is to **configure the SSH port via cloud-init during VM initialization**, rather than trying to reconfigure it later in the configure phase. This approach:

- ✅ Configures SSH port BEFORE any SSH connections are attempted
- ✅ No special connection handling or port overrides needed
- ✅ Works seamlessly with both `WaitForSSHConnectivityStep` and `WaitForCloudInitStep`
- ✅ Simpler and more reliable than post-provisioning reconfiguration

### Implementation Overview

**Phase**: `provision` (VM initialization)
**Mechanism**: Cloud-init `write_files` directive
**Component**: `templates/tofu/common/cloud-init.yml.tera`

The SSH port is configured by:

1. **During Template Rendering** (before VM creation):

   - `TofuProjectGenerator` accepts `ssh_port` from environment configuration
   - `CloudInitRenderer` receives the SSH port value
   - `CloudInitContext` includes the port in template context
   - `cloud-init.yml.tera` renders with the configured port value

2. **During VM Initialization** (first boot):

   - Cloud-init creates `/etc/ssh/sshd_config.d/99-custom-port.conf` with the port setting
   - Cloud-init restarts the SSH service to apply the configuration
   - This happens BEFORE any Ansible connection attempts

3. **During SSH Connectivity Checks**:
   - Both `WaitForSSHConnectivityStep` and `WaitForCloudInitStep` connect using the configured custom port
   - SSH service is already listening on the correct port - connection succeeds immediately

### Files Modified

#### Template Files

- **`templates/tofu/common/cloud-init.yml.tera`**: Added `write_files` section to create SSH port configuration file, and `runcmd` to restart SSH service

#### Infrastructure Layer (DDD)

- **`src/infrastructure/templating/tofu/template/common/wrappers/cloud_init/context.rs`**: Added `ssh_port: u16` field to `CloudInitContext` with builder method
- **`src/infrastructure/templating/tofu/template/common/renderer/cloud_init.rs`**: Updated `CloudInitRenderer` to accept and use `ssh_port` parameter
- **`src/infrastructure/templating/tofu/template/common/renderer/project_generator.rs`**: Updated `TofuProjectGenerator` constructor to accept `ssh_port` parameter

#### Application Layer (DDD)

- **`src/application/command_handlers/provision/handler.rs`**: Updated to pass `ssh_port` from environment to `TofuProjectGenerator`

### Why We Discarded the Ansible-Based Approach

**Initial Plan**: We initially considered using an Ansible playbook in the `configure` phase to reconfigure SSH port after provisioning.

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

**Conclusion**: The cloud-init approach is cleaner, more reliable, and architecturally correct. It configures infrastructure settings during infrastructure provisioning, not as a post-provisioning step.

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

The cloud-init template uses `write_files` to create a drop-in configuration file:

```yaml
write_files:
  - path: /etc/ssh/sshd_config.d/99-custom-port.conf
    content: |
      Port {{ ssh_port }}
    permissions: "0644"

runcmd:
  - systemctl restart ssh
```

This approach:

- Uses Ubuntu's drop-in configuration directory pattern
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
