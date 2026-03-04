---
name: debug-command-failure
description: Guide for debugging and investigating deployer command failures. Covers reading error output, locating trace files, inspecting environment state, examining build artifacts, and running manual verification steps. Use when any deployer command (provision, configure, release, run, etc.) fails. Triggers on "command failed", "debug failure", "investigate error", "why did it fail", "trace", "deployer error", or "command error".
metadata:
  author: torrust
  version: "1.0"
---

# Debugging Deployer Command Failures

This skill walks through collecting and interpreting diagnostic information when any deployer
command fails.

## Investigation Layers (in order)

```text
1. Console error output  →  immediate symptom + tip
2. Environment state     →  data/{env}/environment.json
3. Trace log             →  data/{env}/traces/{timestamp}-{command}.log
4. Build artifacts       →  build/{env}/
5. Manual verification   →  SSH, curl, provider console
```

Work top-to-bottom. Each layer provides richer context than the previous.

---

## Layer 1 — Console Error Output

A failed command prints:

```text
❌ <command> command failed: <error summary>
Tip: <actionable hint>
Tip: Check logs and try running with --log-output file-and-stderr for more details
```

Note the **error summary** and the **tip** lines. The summary often names the failed step and the
kind of error.

---

## Layer 2 — Environment State

After any command failure, the deployer writes machine-readable state:

```text
data/{env-name}/environment.json
```

Key fields to inspect:

```json
{
  "state": {
    "context": {
      "failed_step": "WaitSshConnectivity",
      "error_kind": "NetworkConnectivity",
      "error_summary": "SSH connectivity failed: ...",
      "failed_at": "2026-03-03T15:33:32Z",
      "execution_started_at": "2026-03-03T15:30:00Z",
      "execution_duration": { "secs": 212, "nanos": 885591647 },
      "trace_id": "bcba0ee9-b2cf-4302-be0e-5ed04c665141",
      "trace_file_path": "./data/{env-name}/traces/20260303-153332-provision.log"
    }
  }
}
```

| Field                | What it tells you                                          |
| -------------------- | ---------------------------------------------------------- |
| `failed_step`        | Which internal step failed (maps to deployer source code)  |
| `error_kind`         | Category: `NetworkConnectivity`, `TemplateRendering`, etc. |
| `error_summary`      | Human-readable description of the error                    |
| `execution_duration` | How long the command ran before failing                    |
| `trace_file_path`    | Exact path to the full trace log                           |

```bash
# Quick inspection
cat data/{env-name}/environment.json | python3 -m json.tool
# or
jq '.state.context' data/{env-name}/environment.json
```

---

## Layer 3 — Trace Log

The trace log records every step, sub-step, and decision the deployer made:

```text
data/{env-name}/traces/{YYYYMMDD-HHMMSS}-{command}.log
```

The exact path is in `environment.json → state.context.trace_file_path`.

```bash
# Read the full log
cat data/{env-name}/traces/20260303-153332-provision.log

# Focus on errors and warnings
grep -E 'ERROR|WARN|failed|error' data/{env-name}/traces/20260303-153332-provision.log

# Show the last 50 lines (where failures are usually recorded)
tail -50 data/{env-name}/traces/20260303-153332-provision.log
```

The trace contains structured log lines with timestamps, log levels, and context fields. Look for
`ERROR` lines and the step names that precede them.

---

## Layer 4 — Build Artifacts

The `build/` directory holds rendered templates and intermediate files generated before
infrastructure is touched:

```text
build/{env-name}/
├── tofu/
│   └── hetzner/ (or lxd/)
│       ├── main.tf              # OpenTofu infrastructure definition
│       ├── cloud-init.yml       # cloud-init script run on first boot
│       └── *.tf                 # Other Terraform/OpenTofu files
└── ansible/
    ├── inventory.ini            # Ansible inventory
    └── playbooks/               # Ansible playbooks
```

Common inspections:

```bash
# Verify SSH public key was correctly injected into cloud-init
grep -A3 'ssh_authorized_keys' build/{env-name}/tofu/hetzner/cloud-init.yml

# Compare with the actual public key
cat ~/.ssh/torrust_tracker_deployer_ed25519.pub

# Inspect the infrastructure definition
cat build/{env-name}/tofu/hetzner/main.tf
```

**Why this matters**: Build artifacts are generated from your config file without touching the
cloud provider. If the artifact is wrong, the root cause is in the environment config or a
template bug — not in the network or provider.

---

## Layer 5 — Manual Verification

When the deployer fails but the cloud resource appears to be up, verify the resource directly.

### SSH connectivity

```bash
# Test SSH manually with verbose output (-v shows handshake details)
ssh -v -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@{server-ip} "whoami && cloud-init status"
```

A successful response looks like:

```text
torrust
status: done
```

If `cloud-init status` returns `status: running`, cloud-init is still executing — wait and retry.

### Cloud-init timing

```bash
# Check cloud-init completion and timing
ssh -i ~/.ssh/torrust_tracker_deployer_ed25519 torrust@{server-ip} \
    "cloud-init status --long && sudo journalctl -u ssh --since '5 minutes ago' | tail -20"
```

**Note**: If the clock timestamp shows `1970-01-01`, the system clock was not yet NTP-synced when
cloud-init completed — this is normal and does not indicate a failure.

### Port availability

```bash
# Check if SSH port is open (times out quickly if no service is listening)
nc -zv {server-ip} 22

# Check if HTTP tracker port is open
nc -zv {server-ip} 6969
```

---

## Common Error Patterns

| `failed_step`             | `error_kind`          | Likely Cause                                                         |
| ------------------------- | --------------------- | -------------------------------------------------------------------- |
| `RenderOpenTofuTemplates` | `TemplateRendering`   | SSH key path not found — check container vs host path in config      |
| `WaitSshConnectivity`     | `NetworkConnectivity` | Server SSH not ready within timeout — server may need more boot time |
| `RunAnsiblePlaybook`      | `Ansible`             | SSH key rejected or unreachable — verify `~/.ssh/known_hosts`        |
| `CreateServer`            | `ProviderApi`         | API token invalid or quota exceeded — check Hetzner console          |

---

## After Investigation

Once the root cause is identified, the recovery path depends on how far the command progressed:

- **Failed before any cloud resources were created** (e.g., `TemplateRendering`): fix the config,
  `purge --force`, `create environment`, retry command.

- **Failed after cloud resources were created** (e.g., `WaitSshConnectivity`): the deployer state
  is `ProvisionFailed` or `ConfigureFailed`. Resources exist in the cloud. Must `destroy` to clean
  up both cloud resources and local state, then `create environment` and retry.

```bash
# Destroy cloud resources + local state
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  destroy {env-name}

# Recreate local environment
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  torrust/tracker-deployer:latest \
  create environment --env-file envs/{env-name}.json
```
