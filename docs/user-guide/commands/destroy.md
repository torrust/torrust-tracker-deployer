# Destroy Command

The destroy command removes all infrastructure and resources for a deployment environment. It safely tears down virtual machines, networks, and related resources created during provisioning.

## Command Syntax

```bash
torrust-tracker-deployer destroy <ENVIRONMENT>
```

**Arguments**:

- `<ENVIRONMENT>` - Name of the environment to destroy (required)

**Options**:

- `--help` - Display help information
- `--log-output <OUTPUT>` - Logging output mode (default: `file-only`)
  - `file-only`: Write logs to file only (production mode)
  - `file-and-stderr`: Write logs to both file and stderr (development/testing mode)
- `--log-file-format <FORMAT>` - Format for file logging (default: `compact`)
  - `pretty`: Pretty-printed output for development (no ANSI codes in files)
  - `json`: JSON output for production environments
  - `compact`: Compact output for minimal verbosity
- `--log-stderr-format <FORMAT>` - Format for stderr logging (default: `pretty`)
  - `pretty`: Pretty-printed output with colors for development
  - `json`: JSON output for machine processing
  - `compact`: Compact output with colors for minimal verbosity
- `--log-dir <DIR>` - Log directory (default: `./data/logs`)

## Basic Usage

Destroy an environment:

```bash
torrust-tracker-deployer destroy my-environment
```

**Output**:

```text
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: full-stack-docs (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Tearing down infrastructure...
⏳   ✓ Infrastructure torn down (took 2.6s)
✅ Environment 'full-stack-docs' destroyed successfully

💡 Local data preserved for debugging. To completely remove and reuse the name:
   torrust-tracker-deployer purge full-stack-docs --force
```

With verbose logging to see progress:

```bash
torrust-tracker-deployer destroy my-environment --log-output file-and-stderr
```

## What Gets Destroyed

The destroy command removes:

1. **Virtual Machine Infrastructure**
   - LXD containers or VMs
   - Network interfaces and bridges
   - Allocated IP addresses

2. **Local State Files**
   - Environment data directory (`data/<environment-name>/`)
   - Build artifacts (`build/<environment-name>/`)
   - OpenTofu state files

3. **Environment State**
   - Updates environment state to `Destroyed`
   - Preserves state history for auditing

## Common Use Cases

### Cleaning Up After Testing

Remove a test environment after validation:

```bash
# Note: Only destroy command is currently implemented
torrust-tracker-deployer destroy test-env
```

### Removing Failed Deployments

Clean up after a failed deployment:

```bash
torrust-tracker-deployer destroy failed-env
```

### Scheduled Environment Cleanup

Automate cleanup of temporary environments:

```bash
#!/bin/bash
# cleanup-old-environments.sh

ENVIRONMENTS=("dev-1" "dev-2" "staging-temp")

for env in "${ENVIRONMENTS[@]}"; do
    echo "Destroying $env..."
    torrust-tracker-deployer destroy "$env"
done
```

### Emergency Teardown

Quickly remove an environment in case of issues:

```bash
torrust-tracker-deployer destroy emergency-env --log-output file-and-stderr
```

## Idempotent Operation

The destroy command is **idempotent** - you can run it multiple times safely:

- If infrastructure is already destroyed, the command succeeds without error
- Running destroy twice on the same environment won't cause issues
- Useful in automation scripts where you want to ensure cleanup

```bash
# Safe to run multiple times
torrust-tracker-deployer destroy my-env
torrust-tracker-deployer destroy my-env  # Still succeeds
```

## Exit Codes

- `0` - Success (infrastructure destroyed successfully)
- `1` - Error (destruction failed)

## Troubleshooting

### Destruction Takes Too Long

**Symptom**: The destroy command seems to hang or take a very long time.

**Solution**:

1. Check if OpenTofu is waiting for resources:

   ```bash
   cd build/tofu/lxd
   tofu show
   ```

2. Manually check LXD containers:

   ```bash
   lxc list
   ```

3. Stop hung containers manually if needed:

   ```bash
   lxc stop <container-name> --force
   lxc delete <container-name>
   ```

### Partial Cleanup After Failure

**Symptom**: Destroy command fails but some resources are removed.

**Solution**:

1. Run destroy again (it's idempotent):

   ```bash
   torrust-tracker-deployer destroy my-env
   ```

2. If it continues to fail, manually clean up using OpenTofu:

   ```bash
   cd build/tofu/lxd
   tofu destroy -auto-approve
   ```

3. Remove leftover local directories:

   ```bash
   rm -rf data/my-env build/my-env
   ```

### Error: "OpenTofu command failed"

**Symptom**: Destroy fails with OpenTofu errors.

```text
Error: OpenTofu command failed: resource still in use
```

**Solution**:

1. Check OpenTofu state for locked resources:

   ```bash
   cd build/tofu/lxd
   tofu state list
   ```

2. Unlock any locked resources:

   ```bash
   tofu force-unlock <lock-id>
   ```

3. Try destroying again:

   ```bash
   torrust-tracker-deployer destroy my-env
   ```

4. If the issue persists, manually inspect and remove resources:

   ```bash
   lxc list
   lxc delete <container-name> --force
   ```

### Error: "Failed to clean up state files"

**Symptom**: Infrastructure destroyed but local files remain.

```text
Error: Failed to clean up state files at 'data/my-env': Permission denied
```

**Solution**:

1. Check file permissions:

   ```bash
   ls -la data/my-env
   ```

2. Manually remove directories with appropriate permissions:

   ```bash
   sudo rm -rf data/my-env build/my-env
   ```

3. Verify cleanup:

   ```bash
   ls data/ build/
   ```

### Resource Conflicts

**Symptom**: Destroy fails because resources are in use.

**Solution**:

1. Stop any running applications first
2. Ensure no active SSH connections to the VM
3. Check for dependent resources:

   ```bash
   lxc list
   lxc network list
   ```

4. Manually stop and remove conflicting resources

### Inspecting Logs

If destroy fails, check the logs for detailed information:

```bash
# View logs
cat data/logs/log.txt

# With pretty format for debugging
torrust-tracker-deployer destroy my-env \
    --log-output file-and-stderr \
    --log-stderr-format pretty
```

The logs will show:

- Which step failed during destruction
- Detailed error messages from OpenTofu
- Resource cleanup progress
- State transition information

## Safety Considerations

### Data Loss Warning

⚠️ **The destroy command is destructive and irreversible.**

- All data on the virtual machine will be permanently deleted
- Application data, logs, and configurations will be lost
- State files and build artifacts will be removed
- There is no built-in backup mechanism

**Best Practices**:

1. **Backup Important Data**: Always backup critical data before destroying
2. **Double-Check Environment Names**: Verify you're destroying the correct environment
3. **Test in Non-Production First**: Practice destruction workflow in test environments
4. **Document Destruction**: Keep records of when and why environments were destroyed

### Backup Before Destroy

Always backup important data before destroying:

```bash
# Example: Backup application data
lxc exec my-env -- tar czf /tmp/backup.tar.gz /opt/torrust
lxc file pull my-env/tmp/backup.tar.gz ./my-env-backup.tar.gz

# Now safe to destroy
torrust-tracker-deployer destroy my-env
```

### Confirm Before Destruction

In scripts, add confirmation prompts:

```bash
#!/bin/bash
read -p "Destroy environment '$1'? (yes/no): " confirm

if [ "$confirm" = "yes" ]; then
    torrust-tracker-deployer destroy "$1"
else
    echo "Destruction cancelled"
fi
```

### Production Environments

For production environments:

1. **Require manual approval** in automation workflows
2. **Enable audit logging** to track destruction events
3. **Implement backup policies** before any destruction
4. **Use separate credentials** for destroy operations
5. **Document destruction procedures** in runbooks

## Automated Cleanup

### CI/CD Integration

Example GitHub Actions workflow:

```yaml
name: Cleanup Test Environments

on:
  schedule:
    - cron: "0 2 * * *" # Run at 2 AM daily

jobs:
  cleanup:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup tools
        run: |
          # Install OpenTofu, LXD, etc.

      - name: Destroy old environments
        run: |
          for env in test-1 test-2 staging-temp; do
            echo "Destroying $env..."
            torrust-tracker-deployer destroy "$env" || true
          done
```

### Cleanup Script Template

```bash
#!/bin/bash
# cleanup-environments.sh

set -euo pipefail

ENVIRONMENTS_TO_DESTROY=(
    "dev-feature-1"
    "dev-feature-2"
    "test-temp"
)

for env in "${ENVIRONMENTS_TO_DESTROY[@]}"; do
    echo "═══════════════════════════════════════════"
    echo "Destroying environment: $env"
    echo "═══════════════════════════════════════════"

    if torrust-tracker-deployer destroy "$env"; then
        echo "✓ Successfully destroyed $env"
    else
        echo "✗ Failed to destroy $env"
        # Continue with other environments
    fi

    echo ""
done

echo "Cleanup complete!"
```

## Verification

After running destroy, verify complete cleanup:

### Check LXD Resources

```bash
# List all containers
lxc list

# List all networks
lxc network list

# List all storage pools
lxc storage list
```

### Check Local State Files

```bash
# Verify data directory is removed
ls -la data/

# Verify build directory is removed
ls -la build/

# Check OpenTofu state
ls -la build/tofu/lxd/
```

### Verify Environment State

The environment state should show as `Destroyed`:

```bash
# Check state file (if preserved)
cat data/state.json

# Output should show:
# {
#   "state": "Destroyed",
#   "environment": "my-env",
#   ...
# }
```

## Related Commands

- [Command Index](../commands.md) - Overview of all commands

## See Also

- [Logging Guide](../logging.md) - Configure logging output and formats
- [E2E Testing Guide](../../e2e-testing.md) - How destroy is tested in E2E tests
