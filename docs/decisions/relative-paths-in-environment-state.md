# Decision: Use Relative Paths in Environment State

## Status

Accepted

## Date

2025-11-18

## Context

The application persists environment state to JSON files, including `build_dir` and `data_dir` paths. We discovered an inconsistency between how the CLI and E2E tests store these paths:

- **CLI/Manual usage**: Uses relative paths (default `working_dir = "."`)
  - Results in: `./data/env-name`, `./build/env-name`
- **E2E tests**: Uses absolute paths (`std::env::current_dir()`)
  - Results in: `/home/user/project/data/env-name`, `/home/user/project/build/env-name`

This inconsistency was discovered when comparing manual test state files with E2E test state files. The environment state is an internal database that persists:

- Environment name and instance identifiers
- SSH credentials (using absolute paths - user-controlled external resources)
- Build and data directories (application-managed internal resources)
- Runtime outputs like instance IP addresses

The question is: should `build_dir` and `data_dir` be stored as relative or absolute paths?

## Decision

**Use relative paths for `build_dir` and `data_dir` in persisted environment state.**

We will:

1. **Fix E2E tests** to use `PathBuf::from(".")` instead of `std::env::current_dir()`
2. **Ensure consistency** between CLI and E2E test behavior
3. **Keep SSH credential paths absolute** as they reference user-controlled external resources

The `InternalConfig::with_working_dir()` method will continue to join paths as-is (preserving the relativity/absoluteness of the input `working_dir`), but all callers will use relative paths.

## Consequences

### Positive

✅ **Portability** - Environments can be moved to new locations

- Copy entire workspace from `/opt/deployments` to `/home/user/deployments`
- State files remain valid without modification
- No path fixup required

✅ **Environment Independence** - Not tied to specific users or systems

- State files don't contain `/home/username/...` paths
- Works across different developers' machines
- No user-specific information in state files

✅ **Backup/Restore Friendly** - State is location-agnostic

- Backup `data/` directory
- Restore to any location
- Environment state travels with workspace

✅ **Version Control Friendly** - Consistent across all developers

- Relative paths don't expose user-specific information
- Same state file format for everyone
- Easier code reviews (no path noise)

✅ **Container/Docker Ready** - Works in ephemeral environments

- Mount workspace at any path (`/app`, `/workspace`, etc.)
- Relative paths work regardless of mount point
- No path remapping needed

✅ **Testing Friendly** - Tests can run anywhere

- CI/CD can use `/tmp/ci-build`, `/github/workspace`, etc.
- Local development in any directory
- No assumptions about absolute locations

✅ **CLI Consistency** - Matches CLI default behavior

- CLI uses `working_dir = "."` by default
- E2E tests now behave identically to CLI
- Predictable behavior across all use cases

### Negative

⚠️ **Current Directory Dependency** - Must run from correct directory

- Users must `cd /path/to/project` before running commands
- Mitigated by: `--working-dir` CLI flag for flexibility
- Acceptable trade-off for portability benefits

⚠️ **Path Resolution at Runtime** - Relative paths resolved during operations

- Minimal overhead - standard library handles efficiently
- Not a performance concern in practice

### Design Rationale

**Why relative for internal resources, absolute for external:**

- **SSH credentials** (absolute): User-controlled external files

  - User chooses where to store keys (`~/.ssh/id_rsa`, `/opt/keys/deploy.key`)
  - Application doesn't manage these files
  - Must reference exact location

- **Build/data directories** (relative): Application-managed internal resources
  - Application creates and manages these directories
  - Part of the workspace that moves together
  - Should be portable with the environment state

This separation of concerns ensures:

- External resources referenced by user-chosen absolute paths
- Internal resources use relative paths for portability
- Clear distinction between user-controlled and app-controlled resources

## Alternatives Considered

### Alternative 1: Use Absolute Paths

**Pros:**

- Explicit location - no ambiguity
- Works from any directory without `cd`

**Cons:**

- ❌ Not portable - tied to specific filesystem location
- ❌ User-specific paths in state files
- ❌ Cannot move workspace without breaking state
- ❌ Different paths for each developer
- ❌ Container/Docker issues (hardcoded paths don't exist)
- ❌ Backup/restore requires same absolute path

**Decision:** Rejected due to portability concerns. The CLI already has `--working-dir` flag for cases where users need to run from different directories.

### Alternative 2: Normalize Absolute to Relative in Domain Layer

Add normalization logic in `InternalConfig::with_working_dir()` to convert absolute paths to relative:

```rust
pub fn with_working_dir(env_name: &EnvironmentName, working_dir: &Path) -> Self {
    let normalized = if working_dir.is_absolute() {
        std::env::current_dir()
            .ok()
            .and_then(|cur| working_dir.strip_prefix(&cur).ok())
            .map(|rel| rel.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        working_dir.to_path_buf()
    };
    // ... use normalized path
}
```

**Pros:**

- Defensive programming - handles both cases
- More robust to caller mistakes

**Cons:**

- Added complexity in domain layer
- Hides the problem instead of fixing it at the source
- Unnecessary - callers should just use relative paths

**Decision:** Not implemented initially. We fix the E2E test to use relative paths like the CLI. If future use cases require absolute paths, we can add this normalization logic as defensive programming.

## Related Decisions

- [Environment Variable Prefix](./environment-variable-prefix.md) - Consistent naming for configuration
- [Error Context Strategy](./error-context-strategy.md) - How we persist trace information

## References

- Issue discovered in manual Phase 7 testing for provision command
- Analysis in conversation: relative vs absolute paths pros/cons
- File: `src/domain/environment/internal_config.rs` - Path construction logic
- File: `src/bin/e2e_tests_full.rs` - E2E test working directory setup
- File: `src/presentation/input/cli/args.rs` - CLI default `working_dir = "."`
