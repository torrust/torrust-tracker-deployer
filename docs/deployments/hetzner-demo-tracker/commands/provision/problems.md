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

The actual root cause is **the 120-second SSH connectivity timeout was too short for this
Hetzner server**. The server booted at ~15:30 UTC and the deployer gave up at ~15:33:32 (roughly
3.5 minutes after the server appeared). By the time we tested manually (>14 minutes after boot),
sshd was fully available.

**cloud-init timing note**: `cloud-init status --long` reports
`last_update: Thu, 01 Jan 1970 00:00:13 +0000` — the epoch-based timestamp shows the system clock
was not yet NTP-synced when cloud-init completed, which is consistent with cloud-init completing
very early in the boot sequence before network time was established.

The deployer's SSH probe loop (60 × 2-second intervals = 120 seconds total) is designed for
fast LXD VMs which are typically ready in under 30 seconds. Hetzner bare-metal-class servers with
cloud-init configuration take significantly longer.

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

The retry is expected to succeed because the underlying cause (timeout) is non-deterministic —
the server may be faster on a second attempt. If it times out again, see the Prevention section.

### Prevention

The deployer should expose a configurable SSH connectivity timeout (currently hardcoded at 60
attempts / 120 seconds). For Hetzner servers with cloud-init scripts, a longer timeout is needed.

Potential improvements:

- A `--ssh-timeout` parameter on the `provision` command.
- A `wait-for-ssh` command to decouple the retry loop (useful when provision partially succeeds).
- Document in the Hetzner provider guide that the first SSH probe may take 3–5 minutes.
