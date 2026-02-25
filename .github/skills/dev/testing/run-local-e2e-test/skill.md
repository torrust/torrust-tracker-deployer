---
name: run-local-e2e-test
description: Run a complete local end-to-end test using the LXD provider to verify a feature, fix, or change works as expected from the user's perspective. Covers creating an environment config, provisioning a VM, deploying the tracker, verifying it works, and cleaning up. Use when manually testing changes, validating a new feature locally, running a full deployment smoke test, or checking that the deployer works end-to-end. Triggers on "local e2e", "manual e2e test", "test my changes locally", "run deployer end-to-end", "smoke test", "test on LXD", "verify feature works", or "full deployment test".
metadata:
  author: torrust
  version: "1.0"
---

# Run Local E2E Test (LXD Provider)

This skill walks you through a complete manual end-to-end test of the deployer using a local LXD VM. Use it whenever you want to verify that a feature, fix, or change works as an end-user would experience it.

## When to Use This Skill

- After implementing a new feature — verify the full workflow works
- After fixing a bug — confirm the fix in a real deployment
- Before opening a PR — smoke-test your changes locally
- Investigating a user-reported issue — reproduce it end-to-end

## Prerequisites

```bash
# Verify all required tools are installed
cargo run -p torrust-dependency-installer --bin dependency-installer -- check

# Install missing tools (LXD, OpenTofu, Ansible, Docker)
cargo run -p torrust-dependency-installer --bin dependency-installer -- install
```

> **Note**: `cargo run --bin dependency-installer` does not work from the workspace root because the binary lives in a sub-package. Always use `-p torrust-dependency-installer`.

## Complete Workflow

### Step 1: Create the Environment Config

```bash
# Generate a fresh config template
cargo run -- create template --provider lxd envs/my-local-test.json
```

The generated template contains three `REPLACE_WITH_*` placeholders that must be filled before proceeding:

| Placeholder                                  | Example value                                     |
| -------------------------------------------- | ------------------------------------------------- |
| `REPLACE_WITH_ENVIRONMENT_NAME`              | `my-local-test`                                   |
| `REPLACE_WITH_SSH_PRIVATE_KEY_ABSOLUTE_PATH` | `/absolute/path/to/repo/fixtures/testing_rsa`     |
| `REPLACE_WITH_SSH_PUBLIC_KEY_ABSOLUTE_PATH`  | `/absolute/path/to/repo/fixtures/testing_rsa.pub` |
| `REPLACE_WITH_LXD_PROFILE_NAME`              | `torrust-profile-my-local-test`                   |

> **Use absolute paths** for SSH keys. The `fixtures/testing_rsa` key pair is ready to use for local testing. The profile name must be unique per environment.

### Step 2: Create Environment

```bash
cargo run -- create environment --env-file envs/my-local-test.json
```

Creates `data/my-local-test/` and `build/my-local-test/`.

### Step 3: Provision VM (~30–90s)

```bash
cargo run -- provision my-local-test --log-output file-and-stderr
```

Creates the LXD profile and VM, waits for SSH and cloud-init to complete. The command prints the instance IP and an SSH connection string on success.

```bash
# Re-query the IP at any point after provisioning
# Note: pipe 2>/dev/null to suppress cargo build output before jq
export INSTANCE_IP=$(cargo run -- show my-local-test --output-format json 2>/dev/null | jq -r '.infrastructure.instance_ip')
echo "VM IP: $INSTANCE_IP"
```

### Step 4: Configure Software (~40–60s)

```bash
cargo run -- configure my-local-test
```

Installs Docker and Docker Compose on the VM.

```bash
# Get IP and verify Docker is installed on the VM
export INSTANCE_IP=$(cargo run -- show my-local-test --output-format json 2>/dev/null | jq -r '.infrastructure.instance_ip')
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null \
  torrust@$INSTANCE_IP "docker --version && docker compose version"
```

### Step 5: Release (~7–10s)

```bash
cargo run -- release my-local-test
```

Pulls the tracker Docker image and prepares the runtime environment.

### Step 6: Run Tracker (~10–15s)

```bash
cargo run -- run my-local-test
```

Starts the tracker container and waits for health checks. On success the command prints all service URLs (UDP, HTTP tracker, API, Grafana) — no need to run `show` just to find them.

### Step 7: Test (~5–15s)

```bash
cargo run -- test my-local-test
```

Runs automated health checks against the running deployment (HTTP tracker, HTTP API, health endpoints). A good first gate before doing any manual verification.

### Step 8: Verify (manual)

What to verify depends on what you are testing. Refer to the relevant guide in [`docs/e2e-testing/manual/`](../../docs/e2e-testing/manual/):

| Guide                                                                                  | When to use                                       |
| -------------------------------------------------------------------------------------- | ------------------------------------------------- |
| [tracker-verification.md](../../docs/e2e-testing/manual/tracker-verification.md)       | Always — core health checks, announce/scrape, API |
| [mysql-verification.md](../../docs/e2e-testing/manual/mysql-verification.md)           | When testing with MySQL as the database           |
| [prometheus-verification.md](../../docs/e2e-testing/manual/prometheus-verification.md) | When testing metrics collection                   |
| [grafana-verification.md](../../docs/e2e-testing/manual/grafana-verification.md)       | When testing dashboards                           |
| [backup-verification.md](../../docs/e2e-testing/manual/backup-verification.md)         | When testing the backup service                   |
| [render-verification.md](../../docs/e2e-testing/manual/render-verification.md)         | When testing template rendering                   |

### Step 9: Clean Up

```bash
# Tear down the VM and infrastructure
cargo run -- destroy my-local-test

# Remove local data/build directories and registry entry
cargo run -- purge my-local-test --force
```

`destroy` removes the infrastructure (LXD VM, profile, OpenTofu state) but keeps the local environment data. Its output already reminds you to run `purge`. `purge` removes the remaining `data/` and `build/` directories so the environment name can be reused. Use `--force` to skip the interactive confirmation prompt.

```bash
# Verify cleanup: show exits with code 1 once the environment is purged
cargo run -- show my-local-test; echo "exit: $?"
```

## Quick Reference: Show Command

Use the `show` command instead of parsing internal state files:

```bash
# Human-readable summary (includes IP, SSH command, service URLs)
cargo run -- show <env-name>

# Machine-readable JSON (for scripting)
cargo run -- show <env-name> --output-format json

# Extract IP for scripting (available once provisioned)
# Note: 2>/dev/null suppresses cargo build output so jq only receives JSON
export INSTANCE_IP=$(cargo run -- show <env-name> --output-format json 2>/dev/null | jq -r '.infrastructure.instance_ip')

# Get current state name
cargo run -- show <env-name> --output-format json | jq -r '.state'
```

## Recovery After Interruption

If a command is interrupted mid-run:

```bash
# Check current state
cargo run -- show my-local-test --output-format json 2>/dev/null | jq -r '.state'
```

- **`Provisioning`** (interrupted): destroy and retry from Step 3
- **`Configuring`** (interrupted): re-run `configure` command
- **`Releasing`** / **`Running`** (interrupted): re-run `release` or `run`
- **`Destroying`** (interrupted): re-run `destroy`, then `purge --force`

```bash
# Force cleanup if destroy fails
lxc delete torrust-tracker-vm-my-local-test --force 2>/dev/null
lxc profile delete torrust-profile-my-local-test 2>/dev/null
rm -rf data/my-local-test build/my-local-test
```

## Choosing a Config for Different Test Scenarios

| Scenario          | Config to use / modify                                    |
| ----------------- | --------------------------------------------------------- |
| Basic smoke test  | SQLite (default template)                                 |
| MySQL integration | Add `"driver": "mysql"` + MySQL fields in `core.database` |
| Monitoring stack  | Add `prometheus` + `grafana` sections                     |
| Backup feature    | Add `backup` section                                      |
| HTTPS/TLS         | Add `https` section + set `bind_address` to HTTPS ports   |

## Further Reference

- **Full manual test guide**: [`docs/e2e-testing/manual/README.md`](../../docs/e2e-testing/manual/README.md)
- **Service-specific verification**: MySQL, Prometheus, Grafana, Backup guides in [`docs/e2e-testing/manual/`](../../docs/e2e-testing/manual/)
- **Config structure reference**: [`docs/contributing/ddd-practices.md`](../../docs/contributing/ddd-practices.md), [`schemas/environment-config.json`](../../schemas/environment-config.json)
- **Troubleshooting**: [`docs/e2e-testing/troubleshooting.md`](../../docs/e2e-testing/troubleshooting.md)
