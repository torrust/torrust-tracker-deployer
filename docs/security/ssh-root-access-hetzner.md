# SSH Root Access on Hetzner Cloud Deployments

This document explains the SSH key behavior specific to Hetzner Cloud deployments and provides guidance for users who want stricter security.

## Overview

When deploying to Hetzner Cloud, the deployer configures SSH access through **two independent mechanisms**:

| Mechanism                        | User      | When Applied                  | Purpose                   |
| -------------------------------- | --------- | ----------------------------- | ------------------------- |
| OpenTofu `hcloud_ssh_key`        | `root`    | Server creation (before boot) | Emergency/debug access    |
| cloud-init `ssh_authorized_keys` | `torrust` | First boot                    | Normal application access |

**Result**: Both `root` and `torrust` users have SSH access after deployment.

## Why Root Access Is Enabled

The primary reason is **debugging capability**. If cloud-init fails, the server would be completely inaccessible without root SSH access.

### Failure Scenarios Where Root Access Helps

- **YAML syntax errors** in cloud-init configuration
- **Network issues** during package installation
- **User creation failures** in cloud-init
- **Script execution errors** in cloud-init
- **Timeouts** during cloud-init execution

With root access, you can:

```bash
# SSH as root to diagnose
ssh -i ~/.ssh/your_key root@<server-ip>

# Check cloud-init status
cloud-init status --wait

# View cloud-init logs
cat /var/log/cloud-init-output.log
journalctl -u cloud-init
```

## Security Implications

### Risks

- **Elevated privileges**: Root has unrestricted system access
- **Larger attack surface**: Compromised SSH key grants full system control
- **Principle of least privilege**: Violated by default

### Mitigations Already in Place

- **Application runs as non-root**: The `torrust` user runs the tracker
- **Passwordless sudo**: `torrust` can escalate when needed
- **Same SSH key**: No additional key exposure (both mechanisms use the same key)

## Disabling Root SSH Access

For production deployments where you want stricter security, you can disable root SSH access after verifying the deployment succeeded.

### Option 1: Remove Root's Authorized Keys

The simplest approach - removes the SSH key from root's configuration:

```bash
ssh torrust@<server-ip> "sudo rm /root/.ssh/authorized_keys"
```

**Effect**: Root can no longer SSH in. You can still access via `torrust` user with sudo.

### Option 2: Disable Root Login in SSH Config

Modifies the SSH daemon to reject root logins entirely:

```bash
ssh torrust@<server-ip> "sudo sed -i 's/#PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config && sudo systemctl restart sshd"
```

**Effect**: SSH daemon rejects all root login attempts, regardless of authentication method.

### Option 3: Remove SSH Key from Hetzner Console

Removes the key from Hetzner's account-level registry:

1. Go to [Hetzner Cloud Console](https://console.hetzner.cloud/)
2. Navigate to **Security** → **SSH Keys**
3. Find the key named `torrust-tracker-vm-<environment>-ssh-key`
4. Click **Delete**

**Effect**: Key is removed from your Hetzner account. Does NOT affect existing servers - only prevents the key from being used for future server creation.

## Verification

After disabling root access, verify the change:

```bash
# This should fail (connection refused or permission denied)
ssh -i ~/.ssh/your_key root@<server-ip>

# This should still work
ssh -i ~/.ssh/your_key torrust@<server-ip>
```

## Provider Comparison

| Provider    | Root SSH Access       | Reason                                                |
| ----------- | --------------------- | ----------------------------------------------------- |
| **Hetzner** | ✅ Enabled by default | Debugging capability for cloud-init failures          |
| **LXD**     | ❌ Not applicable     | `lxc exec` provides direct console access without SSH |

The LXD provider doesn't need this pattern because:

- LXD runs locally on your machine
- `lxc exec <instance> -- bash` gives direct console access
- No SSH required for debugging

## Recommendations

### For Development/Testing

Keep root access enabled. The debugging capability is valuable when iterating on configurations.

### For Production

Consider disabling root access after successful deployment:

1. Verify deployment completed successfully
2. Test that you can SSH as `torrust` user
3. Apply one of the disable options above
4. Verify root access is blocked

## Related Documentation

- [ADR: Hetzner SSH Key Dual Injection Pattern](../decisions/hetzner-ssh-key-dual-injection.md)
- [Hetzner Provider Documentation](../user-guide/providers/hetzner.md)
- [SSH Keys Guide](../tech-stack/ssh-keys.md)
