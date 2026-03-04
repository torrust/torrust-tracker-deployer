# OS Updates

**Instance**: Hetzner demo tracker (`46.225.234.201`)
**OS**: Ubuntu 24.04.3 LTS

Apply OS security and package updates periodically to keep the server patched.

## When to Run

- After initial deployment (first login often shows pending updates)
- When the login banner reports security updates available
- At least once a month as routine maintenance

## Procedure

### 1. SSH into the server

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@46.225.234.201
```

The login banner shows the number of pending updates:

```text
59 updates can be applied immediately.
37 of these updates are standard security updates.
```

### 2. Update package lists and upgrade

```bash
sudo apt update && sudo apt upgrade -y
```

This upgrades all installed packages. It may prompt to restart services — accept
the defaults (press Enter or choose "Ok").

### 3. Remove unused packages

```bash
sudo apt autoremove -y
```

### 4. Check if a reboot is required

```bash
[ -f /var/run/reboot-required ] && echo "REBOOT REQUIRED" || echo "No reboot needed"
```

Kernel and libc updates typically require a reboot.

### 5. Reboot if required

```bash
sudo reboot
```

Wait ~30 seconds for the server to come back up, then reconnect and verify
services:

```bash
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@46.225.234.201
cd /opt/torrust && sudo docker compose ps
```

All services should show `running` or `healthy`. If any container failed to
start, check the logs:

```bash
sudo docker compose logs --tail=30 <service-name>
```

## Log

| Date       | Updates Applied  | Reboot Required | Notes                            |
| ---------- | ---------------- | --------------- | -------------------------------- |
| 2026-03-04 | 59 (37 security) | TBD             | First post-deployment update run |
