# LXD Emergency Cleanup Tool

## Overview

The `lxd-cleanup` binary provides an emergency cleanup tool for LXD deployment environments when the normal cleanup process cannot be used. It removes all associated infrastructure and artifacts, reusing the same cleanup logic as the E2E test suite to ensure consistency and reliability.

## ⚠️ IMPORTANT: Normal Environment Removal Process

**For normal operations, always use the standard commands:**

1. **`destroy`** - Destroys the infrastructure (VM, profiles, OpenTofu state, etc.)
2. **`purge`** - Removes the internal deployer data about the environment

Example:

```bash
cargo run -- destroy my-environment
cargo run -- purge my-environment
```

**This emergency tool should ONLY be used when:**

- The environment's internal data is **missing** (manually deleted)
- The environment's internal data is **corrupted** and cannot be read
- E2E tests were interrupted and left orphaned LXD resources
- Emergency recovery is needed for LXD environments

## Installation

The tool is already available in the project. Build it with:

```bash
cargo build --bin lxd_cleanup
```

## Usage

### Clean up a single LXD environment

```bash
cargo run --bin lxd_cleanup -- <environment-name>
```

Example:

```bash
cargo run --bin lxd_cleanup -- cleanup-test
```

### Clean up multiple LXD environments

```bash
cargo run --bin lxd_cleanup -- <env1> <env2> <env3> ...
```

Example:

```bash
cargo run --bin lxd_cleanup -- full-stack-lxd json-provision-https-test lxd-local-example
```

### Dry run mode

Preview what would be cleaned without making any changes:

```bash
cargo run --bin lxd_cleanup -- --dry-run <environment-name>
```

### Custom logging format

Use JSON or compact logging instead of the default pretty format:

```bash
cargo run --bin lxd_cleanup -- --log-format json <environment-name>
cargo run --bin lxd_cleanup -- --log-format compact <environment-name>
```

## What Gets Cleaned

For each LXD environment, this tool removes:

1. **Build directory** - `./build/{environment_name}/`
2. **Templates directory** - `./templates/{environment_name}/`
3. **Data directory** - `./data/{environment_name}/`
4. **OpenTofu infrastructure** - Runs `tofu destroy` if state exists
5. **LXD resources** - Deletes VM instance (`torrust-tracker-vm-{environment_name}`) and profile (`torrust-profile-{environment_name}`)

## When to Use

**Use this emergency tool ONLY when:**

- The environment's internal data is **missing** (manually deleted from `data/` directory)
- The environment's internal data is **corrupted** and the normal `destroy` command fails
- E2E tests were interrupted and left orphaned LXD resources without proper state
- You need emergency recovery for LXD test environments (e.g., bulk cleanup after test failures)

**For all other cases, use the normal commands:**

```bash
cargo run -- destroy <environment-name>  # Destroy infrastructure
cargo run -- purge <environment-name>    # Remove internal data
```

## Safety Considerations

⚠️ **This tool is destructive** and will permanently delete:

- All infrastructure provisioned for the environment
- All generated configuration files
- All environment state data

There is no confirmation prompt, so use with caution in production-like environments.

## Examples

### Example 1: Normal environment removal (PREFERRED)

```bash
# Always try this first - the normal way
cargo run -- destroy my-environment
cargo run -- purge my-environment
```

### Example 2: Emergency cleanup when data is missing

```bash
# Someone manually deleted data/my-environment/
# The 'destroy' command can't work without environment data
# Use emergency cleanup:
cargo run --bin lxd_cleanup -- my-environment

# Verify cleanup
lxc list | grep my-environment  # Should return nothing
ls data/my-environment          # Should not exist
```

### Example 3: Clean up after failed E2E test

```bash
# E2E test crashed and left resources behind
cargo run --bin lxd_cleanup -- e2e-infrastructure

# Verify cleanup
lxc list | grep e2e-infrastructure  # Should return nothing
ls data/e2e-infrastructure          # Should not exist
```

### Example 4: Batch cleanup of LXD test environments

```bash
# Clean up multiple LXD test environments at once
# (e.g., after running many E2E tests that failed to cleanup)
cargo run --bin lxd_cleanup -- \
    manual-test-mysql \
    manual-test-sqlite \
    manual-https-test \
    manual-cron-test
```

### Example 5: Preview cleanup with dry run

```bash
# Check what would be cleaned without actually cleaning
cargo run --bin lxd_cleanup -- --dry-run full-stack-lxd

# Output shows:
#   Would clean:
#   - Build directory: ./build/full-stack-lxd
#   - Templates directory: ./templates/full-stack-lxd
#   - Data directory: ./data/full-stack-lxd
#   - OpenTofu state (if exists)
#   - LXD instance: torrust-tracker-vm-full-stack-lxd
#   - LXD profile: torrust-profile-full-stack-lxd
```

## Implementation Details

The tool reuses the same cleanup functions used by the E2E test suite:

- `run_preflight_cleanup()` from `src/testing/e2e/tasks/black_box/preflight_cleanup.rs`
- `preflight_cleanup_previous_resources()` from `src/testing/e2e/tasks/virtual_machine/preflight_cleanup.rs`

This ensures consistent behavior between automated tests and manual cleanup operations.

## Troubleshooting

### Normal cleanup failed - should I use this tool?

**First, always try the normal process:**

```bash
cargo run -- destroy <environment-name>
cargo run -- purge <environment-name>
```

**Only use the emergency tool if:**

- The `destroy` command fails because environment data is missing/corrupted
- The LXD resources exist but the deployer has no record of them

### Error: "Failed to clean OpenTofu infrastructure"

This usually means:

- OpenTofu state exists but resources are already gone (safe to ignore)
- Resources exist but are in an inconsistent state
- Permission issues accessing the tofu directory

**Solution**: Check if resources actually exist with `lxc list` and manually verify the build directory.

### Error: "Failed to clean LXD instance/profile"

This usually means:

- Resources don't exist (safe to ignore in most cases)
- LXD daemon is not running
- Permission issues with LXD

**Solution**: Check LXD status with `lxc list` and verify your user is in the `lxd` group.

### Multiple environments in different clones

If you have multiple repository clones and LXD environments span across them:

```bash
# Clean in agent-01
cd ~/path/to/agent-01
cargo run --bin lxd_cleanup -- environment-name

# Clean in agent-02
cd ~/path/to/agent-02
cargo run --bin lxd_cleanup -- environment-name

# Note: LXD cleanup only needs to run once (from any clone)
# since LXD resources are system-wide
```

## See Also

- [E2E Testing Documentation](../e2e-testing/README.md)
- [Preflight Cleanup Implementation](../../src/testing/e2e/tasks/virtual_machine/preflight_cleanup.rs)
- [LXD Provider Documentation](../user-guide/providers/lxd.md)
