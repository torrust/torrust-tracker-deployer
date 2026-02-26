---
name: troubleshoot-lxd-instances
description: Guide for debugging and troubleshooting local LXD VM instances created by the deployer. Covers SSH connectivity failures (too many authentication failures, key issues, connection refused), bypassing SSH with lxc exec, cloud-init problems, networking issues, and general LXD instance inspection. Use when a developer cannot access a deployed LXD VM, encounters SSH errors, or needs to inspect VM state without SSH. Triggers on "lxd troubleshoot", "ssh failed lxd", "can't ssh into vm", "lxd debug", "too many authentication failures", "lxc exec", "vm not reachable", "lxd instance problem", "debug lxd instance", or "lxd networking issue".
metadata:
  author: torrust
  version: "1.0"
---

# Troubleshooting LXD Instances

Debugging guide for local LXD VM instances created by the deployer. These tips apply when working with the LXD provider during development and testing.

## Key Concept: Bypass SSH with `lxc exec`

When SSH access to a VM fails for any reason, you can **always** access the instance directly via the LXD hypervisor:

```bash
# Run a single command inside the VM
lxc exec <instance-name> -- <command>

# Open an interactive shell
lxc exec <instance-name> -- bash
```

This bypasses SSH entirely — no keys, no network, no port needed. It works as long as the LXD instance is running.

**Finding the instance name**:

```bash
# List all LXD instances
lxc list

# Filter for torrust instances
lxc list | grep torrust-tracker-vm
```

The instance name follows the pattern `torrust-tracker-vm-<environment-name>`.

## Common Problems

### SSH: "Too many authentication failures"

**Symptom**:

```text
Received disconnect from 10.x.x.x port 22:2: Too many authentication failures
Disconnected from 10.x.x.x port 22
```

**Cause**: The SSH agent offers multiple keys before the correct one, exhausting the server's `MaxAuthTries` limit (default: 6).

**Workarounds**:

```bash
# 1. Force only the specific key (disabling agent)
ssh -i fixtures/testing_rsa -o IdentitiesOnly=yes \
    -o StrictHostKeyChecking=no ubuntu@<vm-ip>

# 2. Bypass SSH entirely with lxc exec
lxc exec torrust-tracker-vm-<env-name> -- cat /opt/torrust/docker-compose.yml

# 3. Clear SSH agent keys and retry
ssh-add -D
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no ubuntu@<vm-ip>
```

**Fix**: Add `-o IdentitiesOnly=yes` to prevent the agent from offering extra keys.

### SSH: "Connection refused"

**Symptom**: `ssh: connect to host 10.x.x.x port 22: Connection refused`

**Diagnosis**:

```bash
# Check if the VM is running
lxc list

# Check if SSH is running inside the VM
lxc exec torrust-tracker-vm-<env-name> -- systemctl status ssh

# Check cloud-init (SSH may not be ready yet)
lxc exec torrust-tracker-vm-<env-name> -- cloud-init status
```

**Cause**: Usually cloud-init hasn't finished setting up the SSH service yet.

**Fix**: Wait for cloud-init to complete, then retry.

### SSH: "Permission denied (publickey)"

**Symptom**: `ubuntu@10.x.x.x: Permission denied (publickey).`

**Diagnosis**:

```bash
# Check which keys are authorized in the VM
lxc exec torrust-tracker-vm-<env-name> -- cat /home/ubuntu/.ssh/authorized_keys

# Compare with the public key you're using
cat fixtures/testing_rsa.pub

# Verify key file permissions on host
ls -la fixtures/testing_rsa
# Should be: -rw------- (600)
```

**Fix**: Ensure the private key matches an authorized public key in the VM. Fix permissions with `chmod 600 fixtures/testing_rsa`.

### Cloud-init Not Completing

**Symptom**: VM is running but services aren't configured, SSH times out.

**Diagnosis**:

```bash
# Check cloud-init status
lxc exec torrust-tracker-vm-<env-name> -- cloud-init status

# View cloud-init logs
lxc exec torrust-tracker-vm-<env-name> -- cat /var/log/cloud-init-output.log

# Watch cloud-init progress in real time
lxc exec torrust-tracker-vm-<env-name> -- tail -f /var/log/cloud-init-output.log
```

**Common causes**: Slow network (package downloads), disk space, resource limits.

### VM Not Getting an IP Address

**Symptom**: `lxc list` shows the instance without an IPv4 address.

**Diagnosis**:

```bash
# Check LXD network
lxc network list
lxc network show lxdbr0

# Check VM network from inside
lxc exec torrust-tracker-vm-<env-name> -- ip addr show

# Check if DHCP is working
lxc exec torrust-tracker-vm-<env-name> -- journalctl -u systemd-networkd
```

**Fix**: Restart the LXD network bridge if needed:

```bash
lxc network delete lxdbr0
lxc network create lxdbr0
```

### LXD Daemon Not Running

**Symptom**: `lxc list` fails or hangs.

**Fix**:

```bash
sudo systemctl status snap.lxd.daemon
sudo systemctl restart snap.lxd.daemon
```

## Quick Inspection Commands

Once you have access (via SSH or `lxc exec`), these commands help inspect the VM state:

```bash
# System overview
lxc exec <instance> -- df -h              # Disk space
lxc exec <instance> -- free -h            # Memory
lxc exec <instance> -- systemctl status   # Service status

# Deployment state
lxc exec <instance> -- cat /opt/torrust/docker-compose.yml
lxc exec <instance> -- cat /opt/torrust/.env

# Detailed instance info from the host
lxc info <instance>
```

## Related Skills and Docs

- Emergency cleanup: use the `clean-lxd-environments` skill
- Expected test errors: use the `debug-test-errors` skill
- LXD provider docs: [`docs/user-guide/providers/lxd.md`](../../../../docs/user-guide/providers/lxd.md)
- E2E troubleshooting: [`docs/e2e-testing/troubleshooting.md`](../../../../docs/e2e-testing/troubleshooting.md)
