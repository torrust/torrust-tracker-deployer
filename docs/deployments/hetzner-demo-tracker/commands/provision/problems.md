# Problems: provision

Issues encountered while running the `provision` command.

> This is a living document — problems are added as they occur.

<!--
Template for each problem:

## Problem: [Short description]

**Command**: `provision`
**Severity**: [Blocker / Major / Minor]

### Symptom

What we observed (error messages, unexpected behavior).

### Root Cause

Why it happened.

### Resolution

How we fixed it.

### Prevention

How to avoid this in the future (if applicable).
-->

## Problem: SSH key paths in config must be container-internal paths when running via Docker

**Command**: `provision`
**Severity**: Blocker

### Symptom

Running `provision` via Docker fails immediately with:

```text
❌ OpenTofu template rendering failed: Failed to render cloud-init template:
   SSH public key file not found or unreadable
```

The SSH key files exist on the host but the deployer cannot find them.

Trace file: `data/torrust-tracker-demo/traces/20260303-152806-provision.log`

```text
Error Summary: OpenTofu template rendering failed: Failed to render cloud-init template:
               SSH public key file not found or unreadable
Failed Step:   RenderOpenTofuTemplates
Error Kind:    TemplateRendering
```

### Root Cause

The `ssh_credentials` paths in the environment config file (`envs/torrust-tracker-demo.json`) were
set to the host machine's paths (e.g. `/home/josecelano/.ssh/torrust_tracker_deployer_ed25519`).
When running the deployer via `docker run`, the SSH directory is mounted into the container at a
different location:

```bash
-v ~/.ssh:/home/deployer/.ssh:ro
```

Inside the container, all SSH keys are under `/home/deployer/.ssh/`, not
`/home/<host-user>/.ssh/`. The deployer reads the path literally from the config, so it looks for
a file that does not exist inside the container.

### Resolution

Always use the container-internal path in `ssh_credentials` when running via Docker:

```json
{
  "ssh_credentials": {
    "private_key_path": "/home/deployer/.ssh/torrust_tracker_deployer_ed25519",
    "public_key_path": "/home/deployer/.ssh/torrust_tracker_deployer_ed25519.pub"
  }
}
```

After fixing the paths, the environment must be recreated (the previous environment was purged
with `--force` because no infrastructure had been created yet):

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  torrust/tracker-deployer:latest \
  purge torrust-tracker-demo --force

docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  torrust/tracker-deployer:latest \
  create environment --env-file envs/torrust-tracker-demo.json
```

### Prevention

The documentation for Docker-based usage should explicitly state that all paths in the config file
(SSH keys, etc.) must be the paths as seen **inside the container**, not on the host. The
`validate` command could also check that the referenced SSH key files are readable from within the
process's filesystem context.

---

## Problem: Provisioning fails — SSH connectivity to newly created server times out

**Command**: `provision`
**Severity**: Blocker

### Symptom

The deployer creates the Hetzner server successfully (the VM appears in the Hetzner console) but
then fails while waiting for SSH to become available:

```text
❌ Provision command failed: Failed to provision environment 'torrust-tracker-demo':
   SSH connectivity failed: Failed to establish SSH connectivity to 46.225.234.201
   after 60 attempts (120s total)
Tip: Check if instance is fully booted and SSH service is running
Tip: Check logs and try running with --log-output file-and-stderr for more details
```

The server IP `46.225.234.201` is reachable (Hetzner reports it as running) but SSH on port 22
never responds within the 120-second window.

Trace file: `data/torrust-tracker-demo/traces/20260303-153332-provision.log`

The deployment environment state recorded in `data/torrust-tracker-demo/environment.json`:

```json
{
  "state": {
    "context": {
      "failed_step": "WaitSshConnectivity",
      "error_kind": "NetworkConnectivity",
      "error_summary": "SSH connectivity failed: ...",
      "failed_at": "2026-03-03T15:33:32.933487060Z",
      "execution_started_at": "2026-03-03T15:30:00.047895413Z",
      "execution_duration": { "secs": 212, "nanos": 885591647 },
      "trace_id": "bcba0ee9-b2cf-4302-be0e-5ed04c665141",
      "trace_file_path": "./data/torrust-tracker-demo/traces/20260303-153332-provision.log"
    }
  }
}
```

### Root Cause

**The SSH public key was correctly injected** — confirmed by inspecting the rendered
`build/torrust-tracker-demo/tofu/hetzner/cloud-init.yml`, which contains the correct public key:

```yaml
ssh_authorized_keys:
  - ssh-ed25519 <KEY> torrust-tracker-deployer
```

This matches the content of `~/.ssh/torrust_tracker_deployer_ed25519.pub` exactly. A manual SSH
from the host confirms the server accepted the key:

```bash
$ ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@46.225.234.201 "whoami && cloud-init status"
torrust
status: done
```

**Precise timeline reconstructed from `data/logs/log.txt`:**

| Time (UTC)    | Event                                                     |
| ------------- | --------------------------------------------------------- |
| `15:30:13`    | `tofu apply` started                                      |
| `15:30:32`    | `tofu apply` done — server `46.225.234.201` created (19s) |
| `15:30:32`    | SSH probe starts (attempt 1)                              |
| `15:30:32–46` | Attempts 1–3: ~7s each (`ConnectTimeout=5` + 2s sleep)    |
| `15:30:49+`   | Attempts 4–60: ~2–3s each                                 |
| `15:33:32`    | Probe exhausted — 60 attempts, 120s total                 |
| `> 15:44:00`  | Manual SSH succeeds (`cloud-init status: done`)           |

The change in per-attempt duration is the key signal:

- **Attempts 1–3 (~7s each)**: port 22 not yet open — the `ssh` process hangs for the full
  `ConnectTimeout=5` seconds before returning. sshd was not listening.
- **Attempts 4–60 (~2–3s each)**: TCP connection to port 22 **succeeds** but sshd rejects the
  authentication immediately. sshd is running but the `torrust` user and `~/.ssh/authorized_keys`
  do not exist yet — cloud-init has not finished creating them.

**The real bottleneck is cloud-init user provisioning**, not sshd startup. sshd was listening
within ~17 seconds of the server appearing. But cloud-init had not yet created the `torrust` user
and written their `authorized_keys`. Every one of the 60 probe attempts was rejected with
"permission denied" for this reason.

Cloud-init finished some time between `15:33:32` (when the deployer gave up) and `~15:44:00`
(when manual SSH succeeded) — meaning cloud-init took **more than 3 minutes 32 seconds** on this
`ccx23` server. The 120-second probe budget was not enough.

**cloud-init timing note**: `cloud-init status --long` reports
`last_update: Thu, 01 Jan 1970 00:00:13 +0000` — the epoch-based timestamp shows the system clock
was not yet NTP-synced when cloud-init completed, which is consistent with cloud-init completing
very early in the boot sequence before network time was established.

The deployer's SSH probe loop (60 × 2-second intervals = 120 seconds total) is designed for
fast LXD VMs which are typically ready in under 30 seconds. Hetzner `ccx23` servers with
cloud-init user provisioning take significantly longer.

### Resolution

The environment is in `ProvisionFailed` state. The deployer does not allow re-running provision
from a failed state; the environment must be destroyed and recreated:

```bash
# Destroy the Hetzner server (cleans up cloud resources and local state)
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  destroy torrust-tracker-demo

# Recreate the environment
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  torrust/tracker-deployer:latest \
  create environment --env-file envs/torrust-tracker-demo.json

# Retry provision
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  provision torrust-tracker-demo
```

The retry may still time out if cloud-init on the new server also takes more than 120 seconds.
If it does, see the Prevention section for options to increase the probe budget.

### Prevention

The deployer should expose a configurable SSH connectivity timeout (currently hardcoded at 60
attempts / 120 seconds). For Hetzner servers with cloud-init scripts, a longer timeout is needed.

Potential improvements:

- A `--ssh-timeout` parameter on the `provision` command.
- A `wait-for-ssh` command to decouple the retry loop (useful when provision partially succeeds).
- Document in the Hetzner provider guide that the first SSH probe may take 3–5 minutes.

---

## Problem: SSH probe always fails from Docker container — passphrase-protected private key

**Command**: `provision`
**Severity**: Blocker

### Symptom

All 60 SSH probe attempts fail with `Permission denied (publickey,password)` even after the server
is fully running and cloud-init has completed. The error appears in `data/logs/log.txt`:

```text
Permission denied, please try again.
Permission denied, please try again.
torrust@46.225.234.201: Permission denied (publickey,password).
```

Manual SSH from the host succeeds with the same key:

```bash
$ ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@46.225.234.201 "whoami && cloud-init status"
torrust
status: done
```

But the same command from inside the Docker container fails:

```bash
$ docker run --rm --entrypoint bash \
    -v ~/.ssh:/home/deployer/.ssh:ro \
    torrust/tracker-deployer:latest \
    -c "ssh -i /home/deployer/.ssh/torrust_tracker_deployer_ed25519 \
        -o StrictHostKeyChecking=no -o IdentitiesOnly=yes \
        torrust@46.225.234.201 'echo connected'"
Permission denied (publickey,password).
```

### Root Cause

**The private key `~/.ssh/torrust_tracker_deployer_ed25519` is protected by a passphrase.**

When SSH is invoked without an agent and without a TTY, it cannot decrypt the private key to
sign the authentication challenge. The connection reaches the public-key offer phase (the server
responds PK_OK), but signing fails silently because there is no passphrase source. The server
then rejects the unauthenticated attempt.

From the host, SSH works because the **GNOME Keyring / SSH agent** has already decrypted the key
and holds it in memory. The agent handles the signing transparently.

Inside the Docker container there is no SSH agent — and the key file, though readable, cannot be
used for authentication without its passphrase.

**This was the true root cause of all three provision failures.** The cloud-init timing issue
was an additional factor in attempts 1 and 2, but even with the 300-second probe budget of
attempt 3 (where cloud-init had finished in time), SSH from Docker still failed every attempt
because the passphrase prevented signing. Forwarding the SSH agent socket into the container
confirms this — with the agent socket forwarded, SSH succeeds immediately:

```bash
docker run --rm --entrypoint bash \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  -v "$SSH_AUTH_SOCK:/tmp/ssh-agent.sock" \
  -e SSH_AUTH_SOCK=/tmp/ssh-agent.sock \
  torrust/tracker-deployer:latest \
  -c "ssh -i /home/deployer/.ssh/torrust_tracker_deployer_ed25519 \
      -o StrictHostKeyChecking=no -o IdentitiesOnly=yes \
      torrust@46.225.234.201 'echo connected'"
# → CONNECTED
```

### Resolution

Remove the passphrase from the deployment key. Deployment keys used for automation do not benefit
from passphrases because:

- There is no interactive session to prompt for the passphrase.
- The key is already protected by filesystem permissions (`0600`).
- The server already restricts access to injected public keys only.

```bash
ssh-keygen -p -f ~/.ssh/torrust_tracker_deployer_ed25519
# Enter current passphrase when prompted
# Leave new passphrase empty (press Enter twice)
```

After removing the passphrase, provision works without any agent forwarding:

```bash
docker run --rm --entrypoint bash \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  -c "ssh -i /home/deployer/.ssh/torrust_tracker_deployer_ed25519 \
      -o StrictHostKeyChecking=no -o IdentitiesOnly=yes \
      torrust@46.225.234.201 'echo connected'"
# → CONNECTED
```

### Notes

- Users may prefer to keep the passphrase on their key (e.g. if the same key is also used for
  interactive logins). In that case, the alternative is to generate a separate passphrase-free key
  specifically for deployment automation and configure it in `ssh_credentials`.
- Hetzner allows setting a default SSH key in the Hetzner console that is automatically added to
  all new servers. If a user has already done this (for a key that may be different), SSH may
  succeed with the wrong key if `IdentitiesOnly=yes` is not set. Keeping `IdentitiesOnly=yes`
  ensures only the explicitly configured deployment key is tried.

### Prevention

The deployer `create environment` or `validate` command should detect passphrase-protected keys
early and warn the user:

```text
⚠ Warning: SSH private key appears to be passphrase-protected. When running via Docker,
  no SSH agent is available and the key cannot be used for authentication.
  Consider removing the passphrase from the deployment key, or document that the
  SSH agent socket must be forwarded into the container.
```
