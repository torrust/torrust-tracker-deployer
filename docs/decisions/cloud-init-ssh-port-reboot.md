# Decision: Cloud-Init SSH Port Configuration with Reboot

## Status

Accepted

## Date

2025-12-11

## Context

The deployer needs to support custom SSH ports for security and flexibility. The SSH port configuration must be applied **during VM provisioning** (not later in the configure phase) because:

1. **Provision phase dependencies**: The `WaitForCloudInitStep` runs during provision and uses Ansible to wait for cloud-init completion. Ansible connects using the custom port from the inventory configuration.

2. **Timing requirement**: If SSH is not already listening on the custom port when `WaitForCloudInitStep` executes, the provision command fails with connection errors.

3. **Architectural correctness**: SSH port is infrastructure configuration, not application configuration. It should be set during infrastructure provisioning, not as a post-provisioning step.

The challenge was ensuring SSH service reliably restarts with the new port configuration during cloud-init execution. Multiple approaches were tested:

- **systemctl restart**: Does not kill the old SSH process when port changes, resulting in SSH listening on both ports 22 and the custom port
- **pkill + systemctl start**: Works but is brittle and non-standard
- **bootcmd disable + runcmd restart**: Ineffective because systemd automatically re-enables and starts SSH after bootcmd completes

## Decision

We configure the custom SSH port via cloud-init using the **`write_files` + `reboot` pattern**, following Hetzner's cloud-config best practices:

1. **Write SSH configuration file** using cloud-init's `write_files` directive:

   ```yaml
   {% if ssh_port != 22 %}
   write_files:
     - path: /etc/ssh/sshd_config.d/99-custom-port.conf
       content: |
         # Custom SSH port configuration
         Port {{ ssh_port }}
       permissions: "0644"
       owner: root:root
   ```

2. **Trigger system reboot** in cloud-init's `runcmd` phase:

   ```yaml
   runcmd:
     - reboot
   {% endif %}
   ```

**Conditional Configuration**: The SSH port configuration and reboot only execute when `ssh_port != 22`, avoiding unnecessary reboots for environments using the default SSH port.

The reboot ensures:

- SSH service cleanly restarts with the new configuration
- No old SSH processes remain on port 22
- All services start in a consistent state
- Package updates are applied (if cloud-init installed packages)

Additionally, we made two critical fixes to the provision handler:

1. **Use configured SSH port**: Changed `wait_for_readiness()` to use `SocketAddr::new(ip, ssh_port)` instead of `SshConfig::with_default_port()`, ensuring the provision handler waits for SSH on the correct custom port (not port 22).

2. **Increase SSH connectivity timeout**: Raised `DEFAULT_MAX_RETRY_ATTEMPTS` from 30 to 60 attempts (120 seconds total), accounting for the ~70-80 second cloud-init completion time plus reboot time.

## Consequences

### Positive

- **Clean SSH restart**: Reboot guarantees SSH only listens on the custom port, no lingering processes on port 22
- **Industry best practice**: Follows Hetzner's documented cloud-config pattern for SSH port changes
- **Simple and reliable**: Single `reboot` command is simpler than managing service lifecycle manually
- **Correct architecture**: Infrastructure configuration happens during infrastructure provisioning
- **No special cases**: Ansible can connect normally using the configured port without overrides or workarounds
- **Compile-time safety**: Provision handler correctly waits for the configured port, preventing connection failures
- **Conditional execution**: Only reboots when custom port is needed (ssh_port != 22), avoiding unnecessary reboots for default configurations

### Negative

- **Slower provisioning**: Reboot adds ~10-20 seconds to VM initialization time
- **Additional wait time**: Provision handler must wait longer (120s instead of 60s) for cloud-init and reboot to complete
- **Complexity**: Three separate changes required (cloud-init template, provision handler port usage, timeout increase)

### Risks

- **Reboot timing**: If reboot takes longer than expected, SSH connectivity check might timeout (mitigated by 120-second timeout)
- **Cloud-init failure**: If reboot fails or cloud-init has errors, the provision will fail (acceptable - we want to catch infrastructure issues early)

## Alternatives Considered

### Alternative 1: Ansible Playbook in Configure Phase

**Approach**: Use an Ansible playbook during the `configure` phase to reconfigure SSH port after provisioning.

**Why Rejected**:

- **Timing problem**: `WaitForCloudInitStep` in provision already fails before reaching configure phase
- **Architectural mismatch**: SSH port is infrastructure config, should be set during VM initialization
- **Added complexity**: Requires special connection handling (connect on 22, reconfigure, reconnect on custom port)
- **More failure points**: Port transition adds potential for connection issues

### Alternative 2: systemctl restart Without Reboot

**Approach**: Use cloud-init `runcmd` to execute `systemctl restart ssh` without full system reboot.

**Why Rejected**:

- **Doesn't kill old process**: `systemctl restart` doesn't terminate the existing SSH daemon when port changes
- **Dual port listening**: Results in SSH listening on both port 22 (old) and custom port (new)
- **Testing showed failure**: Multiple test attempts confirmed SSH remained on port 22 after cloud-init "completion"

### Alternative 3: pkill + systemctl start

**Approach**: Kill SSH processes with `pkill -9 sshd`, then start fresh with `systemctl start ssh`.

**Why Rejected**:

- **Non-standard**: Violates best practices for service management
- **Brittle**: Process killing is less reliable than clean reboot
- **Not industry pattern**: No documentation or precedent for this approach

### Alternative 4: Wait for Port 22, Then Handle Port Change

**Approach**: Keep provision handler waiting for port 22, handle port transition separately.

**Why Rejected**:

- **Wrong abstraction**: Provision handler should use the configured port, not hardcode defaults
- **Added complexity**: Would require special logic to detect port changes mid-provision
- **Race conditions**: SSH might move to custom port at unpredictable times during cloud-init

## Related Decisions

- [Register Command SSH Port Override](./register-ssh-port-override.md) - Relates to SSH port handling in different commands
- [Environment Variable Prefix](./environment-variable-prefix.md) - Relates to configuration management patterns

## References

- [Hetzner Cloud-Config Tutorial](https://community.hetzner.com/tutorials/basic-cloud-config) - Section 5.3 documents the reboot pattern for SSH configuration
- [Cloud-Init Documentation](https://cloudinit.readthedocs.io/en/latest/) - Official cloud-init reference
- [Issue #222: Configure SSH Service Port](../issues/222-configure-ssh-service-port.md) - Original issue specification
- [OpenSSH sshd_config.d](https://manpages.debian.org/bookworm/openssh-server/sshd_config.5.en.html#Include) - Ubuntu SSH configuration directory pattern
