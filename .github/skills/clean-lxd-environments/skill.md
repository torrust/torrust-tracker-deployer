---
name: clean-lxd-environments
description: Emergency cleanup of orphaned LXD deployment environments when normal 'destroy' and 'purge' commands cannot be used. Use ONLY when environment data is missing or corrupted, or E2E tests left orphaned resources. This is NOT for normal cleanup - always try 'destroy' + 'purge' first. Triggers on "clean lxd", "cleanup orphaned lxd", "remove lxd vms", "emergency cleanup", "lxd cleanup", "stuck lxd resources", or "orphaned vms".
metadata:
  author: torrust
  version: "1.0"
---

# Clean LXD Environments (Emergency Tool)

Emergency cleanup tool for orphaned LXD deployment environments. **This is NOT the normal cleanup process.**

## ⚠️ IMPORTANT: Normal Cleanup Process

**Always try this first:**

```bash
cargo run -- destroy <environment-name>  # Destroy infrastructure
cargo run -- purge <environment-name>    # Remove internal data
```

## When to Use This Emergency Tool

**ONLY use this tool when:**

- Environment internal data is **missing** (manually deleted from `data/`)
- Environment internal data is **corrupted** and `destroy` command fails
- E2E tests crashed and left orphaned LXD resources
- Emergency bulk cleanup of test environments

## What Gets Cleaned

For each LXD environment:

1. **Build directory** - `./build/{environment_name}/`
2. **Templates directory** - `./templates/{environment_name}/`
3. **Data directory** - `./data/{environment_name}/`
4. **OpenTofu infrastructure** - Runs `tofu destroy` if state exists
5. **LXD resources** - VM instance and profile

## Usage

### Clean Single Environment

```bash
cargo run --bin lxd_cleanup -- <environment-name>
```

**Example:**

```bash
cargo run --bin lxd_cleanup -- manual-test-mysql
```

### Clean Multiple Environments

```bash
cargo run --bin lxd_cleanup -- <env1> <env2> <env3>
```

**Example:**

```bash
cargo run --bin lxd_cleanup -- \
    manual-test-mysql \
    manual-test-sqlite \
    manual-https-test
```

### Dry Run (Preview)

```bash
cargo run --bin lxd_cleanup -- --dry-run <environment-name>
```

Shows what would be cleaned without making changes.

## Common Scenarios

### Scenario 1: Data Directory Manually Deleted

```bash
# Someone deleted data/my-env/ but VM still exists
lxc list | grep my-env  # VM exists

# Normal cleanup won't work (no data)
cargo run -- destroy my-env  # FAILS: No environment data

# Use emergency cleanup
cargo run --bin lxd_cleanup -- my-env
```

### Scenario 2: E2E Test Crashed

```bash
# E2E test failed and left resources
lxc list  # Shows: torrust-tracker-vm-e2e-infrastructure

# Environment data might be incomplete/corrupted
cargo run --bin lxd_cleanup -- e2e-infrastructure
```

### Scenario 3: Bulk Cleanup After Many Tests

```bash
# List orphaned VMs
lxc list --format csv | grep "torrust-tracker-vm-"

# Clean all at once
cargo run --bin lxd_cleanup -- \
    full-stack-lxd \
    json-provision-https-test \
    lxd-local-example \
    manual-cron-test
```

## Verification Steps

### Before Cleanup

```bash
# Check what exists
lxc list | grep torrust-tracker-vm
ls data/
ls build/
```

### After Cleanup

```bash
# Verify everything removed
lxc list | grep <environment-name>  # Should return nothing
ls data/<environment-name>          # Should not exist
ls build/<environment-name>         # Should not exist
```

## Troubleshooting

### Error: "Failed to clean OpenTofu infrastructure"

- Usually means resources already gone (safe to ignore)
- Or OpenTofu state is inconsistent
- Check: `lxc list` to see if resources actually exist

### Error: "Failed to clean LXD instance/profile"

- Resources don't exist (safe to ignore)
- LXD daemon not running: `lxc list`
- Permission issues: verify user in `lxd` group

### When Normal Cleanup Works

```bash
# Try normal cleanup first - it ALWAYS works better when possible
cargo run -- destroy my-environment
cargo run -- purge my-environment

# Only use emergency tool if above fails
```

## Safety Note

This tool is **destructive** and has **no confirmation prompt**. It permanently deletes:

- All generated configurations
- All environment state
- All provisioned infrastructure

Cannot be undone. Use with caution.

## See Also

- Full documentation: `docs/tools/lxd-cleanup.md`
- LXD provider guide: `docs/tech-stack/lxd.md`
- Normal cleanup: User guide destroy/purge commands
- E2E testing: `docs/e2e-testing/README.md`
