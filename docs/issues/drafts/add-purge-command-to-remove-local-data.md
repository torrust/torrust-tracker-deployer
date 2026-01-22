# Draft: Add Purge Command to Remove Local Environment Data

**Status**: Draft (not yet created on GitHub)
**Related Roadmap Section**: 10. Improve usability (UX)

## Summary

Add a `purge` command that removes local data for destroyed environments, allowing users to reuse environment names and clean up disk space.

## Context

When the `destroy` command runs, it:

1. Tears down the real infrastructure (VMs, networks, etc.)
2. Changes the environment state to "Destroyed"
3. **Preserves local data** in the `data/` folder

**Why local data is preserved:**

- If destruction fails, users can access environment data to debug
- Contains configuration, state, and generated artifacts
- Allows inspection of what was deployed

**Problem:**

- If destruction succeeds, the environment name is still "taken"
- User cannot create a new environment with the same name
- Forces manual cleanup (removing `data/{env-name}` folder)
- Users shouldn't need to know internal storage details

## Proposed Solution

Add a `purge` command with clear separation from `destroy`:

| Command   | What it does                                  | When to use                                     |
| --------- | --------------------------------------------- | ----------------------------------------------- |
| `destroy` | Destroys infrastructure, keeps local data     | Normal teardown                                 |
| `purge`   | Removes local data for destroyed environments | After destroy, to reuse name or free disk space |

### Command Usage

```bash
# After destroying an environment
torrust-tracker-deployer purge lxd-local-example
```

### Example Output

**Successful purge:**

```text
⏳ [1/2] Validating environment...
⏳   ✓ Environment 'lxd-local-example' is in Destroyed state (took 0ms)
⏳ [2/2] Purging local data...
⏳   ✓ Local data removed (took 5ms)
✅ Environment 'lxd-local-example' purged successfully
```

**Attempting to purge an active environment (error):**

```text
❌ Cannot purge 'lxd-local-example': environment is in 'Running' state

Tip: Destroy the environment first with:
  torrust-tracker-deployer destroy lxd-local-example
```

**Hint after destroy command:**

```text
✅ Environment 'lxd-local-example' destroyed successfully

Infrastructure has been torn down. Local data preserved for debugging.
To fully remove: torrust-tracker-deployer purge lxd-local-example
```

## Design Decisions

### Decision 1: No `--clean` flag on destroy

**Rejected option:** `destroy --clean` to auto-purge on success

**Reason:** You don't know beforehand if destruction will fail. Using `--clean` could:

- Leave orphaned infrastructure resources without any local data to debug
- Lose valuable information if something goes wrong

**Conclusion:** Better to always require explicit `purge` after successful `destroy`.

### Decision 2: No auto-clean on success

**Rejected option:** Automatically purge data if `destroy` succeeds

**Reason:** System should behave consistently regardless of operation result. Either:

- `destroy` always preserves data, OR
- `destroy` always removes data

Mixing behaviors based on success/failure is confusing and unpredictable.

**Conclusion:** `destroy` always preserves data. `purge` is always explicit.

### Decision 3: Use "purge" not "clean"

**Rejected option:** `clean` command

**Reason:** "Clean" is ambiguous - users might confuse whether to run `destroy` or `clean`.

**Why "purge" is better:**

- Common in package managers (`apt purge` vs `apt remove`)
- Sounds more final/destructive than "clean"
- Clear semantic: "destroy" = infrastructure, "purge" = local data
- Unambiguous: you can't "purge" running infrastructure

### Decision 4: Only purge destroyed environments

`purge` should only work on environments in these states:

- **Destroyed**: Normal use case after `destroy`
- **Created**: Abandoned environments that were never provisioned (optional)

Cannot purge environments in: Running, Provisioned, Configured, Released states.

## Implementation Notes

- Check environment state before purging
- Remove `data/{env-name}/` directory
- Remove `build/{env-name}/` directory (generated artifacts)
- Provide clear error if environment is not in purgeable state
- Update `destroy` output to hint about `purge` command

## What Gets Purged

The `purge` command removes:

```text
data/{env-name}/          # Environment state and configuration
├── environment.json      # Serialized environment state
├── templates/            # Any copied templates
└── ...

build/{env-name}/         # Generated artifacts
├── ansible/              # Generated Ansible playbooks
├── docker-compose/       # Generated docker-compose files
├── tofu/                 # Generated OpenTofu files
└── ...
```

## Open Questions

1. Should `purge` also work on "Created" state (never provisioned)?
2. Should there be a `--force` flag to purge any state (dangerous)?
3. Should `list` command show "Destroyed" environments differently?

## Reference

Current `destroy` output (see what needs to change):

- [lxd-local-example.md](../../reference/command-outputs/lxd-local-example.md)
