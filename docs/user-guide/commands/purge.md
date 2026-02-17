# Purge Command

The purge command permanently removes all local data for a deployment environment. It deletes data directories, build artifacts, and environment registry entries, allowing you to completely clean up after an environment is no longer needed or to reuse an environment name.

## Command Syntax

```bash
torrust-tracker-deployer purge <ENVIRONMENT> [OPTIONS]
```

**Arguments**:

- `<ENVIRONMENT>` - Name of the environment to purge (required)

**Options**:

- `--force` - Skip confirmation prompt (for automation)
- `--help` - Display help information
- `--working-dir <DIR>` - Set the working directory (default: current directory)
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

### Interactive Purge (with confirmation)

Purge an environment with confirmation prompt:

```bash
torrust-tracker-deployer purge my-environment
```

You'll be prompted to confirm:

```text
⚠️  WARNING: This will permanently delete all local data for 'my-environment':
• data/my-environment/ directory
• build/my-environment/ directory
• Environment registry entry

This operation CANNOT be undone!

Are you sure you want to continue? (y/N):
```

### Automated Purge (skip confirmation)

For scripts and automation, use `--force` to skip the confirmation prompt:

```bash
torrust-tracker-deployer purge my-environment --force
```

**Output**:

```text
⏳ [1/3] Validating environment...
⏳   ✓ Done (took 0ms)
⏳ [2/3] Purging local data...
⏳   ✓ Done (took 0ms)
✅ Environment 'full-stack-docs' purged successfully
```

### With Verbose Logging

See detailed progress during purge:

```bash
torrust-tracker-deployer purge my-environment --force --log-output file-and-stderr
```

## What Gets Purged

The purge command removes:

1. **Data Directory** (`data/<environment-name>/`)
   - Environment state file (`environment.json`)
   - State history and metadata
   - Trace files and logs

2. **Build Directory** (`build/<environment-name>/`)
   - Generated Ansible playbooks
   - Generated Docker Compose files
   - Generated OpenTofu/Terraform files
   - Any other build artifacts

3. **Environment Registry Entry**
   - Removes the environment from the deployer's internal registry
   - Allows the environment name to be reused

**Important**: Purge does NOT destroy infrastructure. You must run `destroy` first to tear down VMs and resources.

## Common Use Cases

### Normal Workflow: Destroy Then Purge

The typical cleanup sequence:

```bash
# Step 1: Destroy infrastructure
torrust-tracker-deployer destroy my-env

# Step 2: Purge local data
torrust-tracker-deployer purge my-env --force
```

### Reusing Environment Names

After purging, you can reuse the environment name:

```bash
# Cleanup old environment
torrust-tracker-deployer destroy old-env
torrust-tracker-deployer purge old-env --force

# Create new environment with same name
torrust-tracker-deployer create environment --env-file new-config.json
# (where new-config.json has environmentName: "old-env")
```

### Cleaning Up Failed Deployments

Remove local data from a failed deployment:

```bash
# If infrastructure was never created or already destroyed
torrust-tracker-deployer purge failed-env --force
```

### Automated Cleanup Script

Automate cleanup of multiple environments:

```bash
#!/bin/bash
# cleanup-environments.sh

ENVIRONMENTS=("dev-1" "dev-2" "staging-temp")

for env in "${ENVIRONMENTS[@]}"; do
    echo "Cleaning up $env..."

    # Destroy infrastructure
    torrust-tracker-deployer destroy "$env" 2>/dev/null || true

    # Purge local data
    torrust-tracker-deployer purge "$env" --force

    echo "✅ $env cleaned up"
done
```

### Cleaning Up After Manual Infrastructure Removal

If you destroyed infrastructure manually (outside the deployer):

```bash
# Infrastructure already gone, just clean up local data
torrust-tracker-deployer purge my-env --force
```

## When to Use Purge

### ✅ Use Purge When

- **After `destroy` command**: Normal cleanup workflow
- **Freeing up disk space**: Remove old environment data
- **Reusing environment names**: Allow name to be used again
- **Manual infrastructure removal**: Infrastructure destroyed outside deployer
- **Failed deployments**: Clean up after errors in early stages

### ❌ Don't Use Purge When

- **Infrastructure still running**: Run `destroy` first to tear down VMs
- **Need to keep data for debugging**: Purge removes all local state
- **Want to resume deployment**: Purge makes environment unrecoverable
- **Auditing required**: Purge removes state history

## Idempotent Operation

The purge command is **idempotent** - you can run it multiple times safely:

- If data is already purged, the command reports an error (environment not found)
- Individual directory removals are idempotent (succeed if already removed)
- Useful in cleanup scripts where you want to ensure complete removal

```bash
# First purge succeeds
torrust-tracker-deployer purge my-env --force

# Second purge fails with "environment not found" (expected)
torrust-tracker-deployer purge my-env --force
```

## Exit Codes

- `0` - Success (environment purged successfully)
- `1` - Error (purge failed or environment not found)

## Troubleshooting

### Error: "Environment not found"

**Symptom**: Purge fails because the environment doesn't exist in the registry.

```text
❌ Purge command failed: Failed to purge environment 'my-env': Environment not found: my-env
```

**Possible Causes**:

1. Environment was already purged
2. Environment was never created
3. Wrong environment name
4. Wrong working directory

**Solution**:

```bash
# List existing environments
torrust-tracker-deployer list

# Verify working directory
pwd
ls data/

# Use correct environment name from list
torrust-tracker-deployer purge correct-name --force
```

### Error: "Permission denied"

**Symptom**: Purge fails due to file permission issues.

```text
❌ Failed to purge environment 'my-env': Failed to remove data directory: Permission denied
```

**Solution**:

```bash
# Check directory permissions
ls -la data/my-env/
ls -la build/my-env/

# Fix permissions if needed
chmod -R u+w data/my-env/
chmod -R u+w build/my-env/

# Try purge again
torrust-tracker-deployer purge my-env --force

# If still failing, manually remove with sudo
sudo rm -rf data/my-env/ build/my-env/
```

### Error: "Failed to read user input"

**Symptom**: Purge fails when trying to read confirmation in non-interactive environment.

```text
❌ Failed during reading user confirmation: cannot read from stdin
```

**Solution**:

Use `--force` flag to skip the interactive prompt:

```bash
# In CI/CD, automation, or piped scripts
torrust-tracker-deployer purge my-env --force
```

### Directories Remain After Purge

**Symptom**: Purge command succeeds but directories still exist.

**Solution**:

This shouldn't happen in normal operation. If it does:

```bash
# Manually verify what remains
ls -la data/my-env/ 2>/dev/null
ls -la build/my-env/ 2>/dev/null

# Manually remove if necessary
rm -rf data/my-env/ build/my-env/

# Check for file locks
lsof +D ./data/my-env/ 2>/dev/null
lsof +D ./build/my-env/ 2>/dev/null
```

### Infrastructure Still Running

**Symptom**: You purged local data but infrastructure still exists.

**Important**: Purge does NOT destroy infrastructure!

**Solution**:

```bash
# You must destroy infrastructure separately
# This may fail if local state is gone, requiring manual cleanup

# Option 1: If you have infrastructure details
lxc list  # Find your container/VM
lxc delete <container-name> --force

# Option 2: Use external tools
# OpenTofu cleanup if you have state files elsewhere
cd /path/to/tofu/state
tofu destroy -auto-approve
```

### Inspecting Logs

If purge fails, check the logs for detailed information:

```bash
# View logs
cat data/logs/log.txt

# With verbose output for debugging
torrust-tracker-deployer purge my-env --force \
    --log-output file-and-stderr \
    --log-stderr-format pretty
```

The logs will show:

- Which step failed during purge
- File system operation details
- Path information for directories
- Error context and troubleshooting hints

## Safety Considerations

### ⚠️ Data Loss Warning

**The purge command is destructive and irreversible.**

- All local environment data is permanently deleted
- Environment state and history are lost
- Build artifacts and generated files are removed
- Environment name becomes available for reuse
- **There is NO recovery mechanism**

### Best Practices

1. **Destroy Infrastructure First**
   - Always run `destroy` before `purge`
   - Ensures infrastructure is properly torn down
   - Prevents orphaned resources

2. **Backup Important Data**
   - Save any configuration files you need
   - Export environment details with `show` command
   - Keep copies of custom templates

3. **Double-Check Environment Names**
   - Use `list` command to verify environment name
   - Confirm you're purging the correct environment
   - Use tab completion to avoid typos

4. **Understand --force Flag**
   - Only use `--force` in automation
   - Interactive mode provides a safety check
   - Confirmation prompt prevents accidental purges

5. **Document Purge Operations**
   - Keep records of when environments were purged
   - Note why the environment was removed
   - Track environment lifecycle for auditing

### Backup Checklist Before Purge

Before purging, consider backing up:

- [ ] Environment configuration file (original JSON)
- [ ] Generated build artifacts (if needed for reference)
- [ ] Environment state information (`show` command output)
- [ ] Any custom modifications to generated files
- [ ] Trace files and logs (if needed for debugging)

## Comparison with Destroy

| Aspect                  | `destroy`                          | `purge`                             |
| ----------------------- | ---------------------------------- | ----------------------------------- |
| **Infrastructure**      | ✅ Removes VMs, networks           | ❌ Does nothing with infrastructure |
| **Local Data**          | ⚠️ Preserves for debugging         | ✅ Removes completely               |
| **Environment State**   | ✅ Sets to "Destroyed"             | ✅ Removes from registry            |
| **Name Reuse**          | ❌ Name remains reserved           | ✅ Name becomes available           |
| **Reversibility**       | ⚠️ Can view state after            | ❌ Completely irreversible          |
| **Typical Usage**       | Tear down running infrastructure   | Clean up after destroy              |
| **Required For**        | Stopping VMs and freeing resources | Reusing names, freeing disk space   |
| **Can Run Standalone**  | ✅ Yes (normal operation)          | ✅ Yes (for manual cleanup)         |
| **Confirmation Prompt** | ❌ No (destructive but necessary)  | ✅ Yes (unless --force)             |

### Recommended Workflow

For complete cleanup, always use both commands in order:

```bash
# 1. Destroy infrastructure first
torrust-tracker-deployer destroy my-env

# 2. Then purge local data
torrust-tracker-deployer purge my-env --force
```

This ensures:

- Infrastructure is properly torn down
- No orphaned resources remain
- Local data is completely removed
- Environment name can be reused

## See Also

- **[destroy command](destroy.md)** - Tear down infrastructure before purging
- **[show command](show.md)** - View environment details before purging
- **[list command](../commands.md)** - See all environments that can be purged
- **[create command](create.md)** - Create new environments after purging
- **[Quick Start Guides](../quick-start/README.md)** - Complete deployment workflows
