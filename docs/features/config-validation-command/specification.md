# Config Validation Command Specification

## 📋 Overview

This feature provides a mechanism to validate environment configuration files without producing any side effects on the application's internal state.

### Context

The Torrust Tracker Deployer follows a sequential command workflow:

1. **Create**: Parse and store environment configuration (modifies internal state)
2. **Provision**: Create infrastructure resources
3. **Configure**: Set up the provisioned instance
4. **Release**: Deploy the tracker software
5. **Run**: Start the tracker
6. **Test**: Verify the deployment
7. **Destroy**: Tear down the environment

The `create` command currently serves two purposes:

1. Validates the environment configuration JSON file
2. Persists the configuration to internal application state

This coupling means users cannot validate a configuration without committing to it.

### Problem Statement

Users and AI agents need a way to:

- **Experiment safely**: Try different configurations without affecting state
- **Validate before committing**: Ensure configs are correct before the `create` command
- **Learn the system**: Understand what valid configurations look like
- **Automate validation**: Check configs in CI/CD pipelines without side effects

Currently, there is no way to validate a configuration file without modifying the application's internal state.

## 🎯 Goals

### Primary Goals

- **Pure validation**: Validate config files without any side effects
- **Clear feedback**: Provide actionable error messages for invalid configs
- **Discoverability**: Make the feature easy to find and use

### Secondary Goals (Nice-to-Have)

- Structured output (JSON) for programmatic use
- Verbose mode showing all validation steps
- Integration with JSON Schema for editor support

### Future Enhancements (Not in Initial Implementation)

The following enhancements may be considered **after** the basic feature proves useful to actual users:

| Enhancement                  | Description                                    | Use Case                           |
| ---------------------------- | ---------------------------------------------- | ---------------------------------- |
| **Stdin support**            | `cat config.json \| validate -`                | Scripting and piping workflows     |
| **Multiple file validation** | `validate --env-file a.json --env-file b.json` | Batch validation in CI/CD          |
| **Watch mode**               | `validate --watch --env-file config.json`      | Auto-revalidate during development |
| **Editor integration docs**  | VSCode task configuration examples             | Real-time feedback while editing   |

**Note**: The `create` command can already be used safely since it only modifies internal state and is easily reversible with `destroy`. The `validate` command is a convenience feature to:

- Avoid cluttering the data directory with test environments
- Allow AI agents to experiment with configurations without cleanup

We intentionally keep the initial implementation minimal. Additional features will be considered only after real-world usage validates the need.

### Non-Goals

What this feature explicitly does NOT aim to do:

- Validate external resource availability (SSH keys, API connectivity)
- Check provider-specific constraints (e.g., LXD socket accessible)
- Modify any application state
- Create, update, or delete environments

## 💡 Proposed Solution

### Approach

Introduce a dedicated `validate` command that reuses the existing validation logic from the `create` command but stops before persisting any state.

### Design Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                    User/AI Agent                            │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  validate --env-file envs/config.json                       │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  Validation Pipeline                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ JSON Parse  │→ │ Schema      │→ │ Semantic            │  │
│  │             │  │ Validation  │  │ Validation          │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│  Success: "Configuration is valid"                          │
│  Failure: "Error: {detailed error message}"                 │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
                    [NO STATE CHANGES]
```

### Key Design Decisions

1. **Standalone `validate` command**: Chosen over `--dry-run` flag (see detailed rationale below)
2. **Config-intrinsic validation only**: Does NOT check state-dependent conditions (see validation levels below)
3. **Pure function**: Validation must have no side effects whatsoever
4. **Treat config as first environment**: Validate as if no other environments exist

### Validation Levels

There are three distinct levels of validation, and understanding them is crucial for this feature:

| Level | Name                           | Description                                             | `validate` | `--dry-run` (future) |
| ----- | ------------------------------ | ------------------------------------------------------- | ---------- | -------------------- |
| 1     | **Syntactic**                  | JSON is valid, required fields exist, types are correct | ✅         | ✅                   |
| 2     | **Config-intrinsic semantics** | Cross-field rules within the config itself              | ✅         | ✅                   |
| 3     | **State-dependent semantics**  | Rules depending on current app/environment state        | ❌         | ✅                   |

#### Level 2: Config-Intrinsic Semantics (Included)

These are rules that can be validated by looking **only** at the configuration file:

- If Grafana is enabled, Prometheus must be enabled
- If database type is MySQL, MySQL config section must be present
- Port numbers must be in valid range (1-65535)
- Provider-specific required fields are present
- TLS configuration is internally consistent

#### Level 3: State-Dependent Semantics (Excluded)

These are rules that require knowledge of the **current application state**:

- Environment name already exists in data directory
- SSH key file exists on filesystem
- Provider API is reachable and credentials are valid
- Port is not already in use

**Why exclude state-dependent validation?**

The `validate` command answers: "Is this configuration intrinsically valid?"

Not: "Will `create` succeed in my current data directory?"

The same configuration file:

- Is valid in a fresh data directory
- May cause a name conflict in a populated data directory
- May work with one SSH key path but not another

By treating the config as if it were the **first environment being created**, we provide a pure, deterministic validation that doesn't depend on external factors.

### Decision: `validate` Command vs `--dry-run` Flag

#### The Core Insight

A `validate` command and a `--dry-run` flag are **not interchangeable** - they make fundamentally different promises to users:

| Aspect               | `validate` command                        | `--dry-run` flag                      |
| -------------------- | ----------------------------------------- | ------------------------------------- |
| **Promise**          | "Is this configuration valid?"            | "Will `create` succeed here?"         |
| **Validation level** | Levels 1-2 (syntactic + config-intrinsic) | Levels 1-3 (includes state-dependent) |
| **Scope**            | Config in isolation                       | Config + current app state            |
| **Side effects**     | None, ever                                | None, but reads app state             |
| **Determinism**      | Same input → same output                  | May vary by data directory            |
| **Name conflict**    | Ignored (valid config)                    | Error (would fail)                    |
| **Mental model**     | "Is this config sane?"                    | "Will this work now?"                 |

#### Why `--dry-run` is Less Critical for `create`

Unlike commands that provision infrastructure, the `create` command has **minimal side effects**:

- It only modifies internal application state (creates files in `data/` directory)
- It does NOT create any infrastructure resources (VMs, networks, DNS)
- It does NOT incur costs or consume external resources
- It can be trivially undone with `destroy`

This means users can safely experiment with `create` and `destroy` without consequences. The primary motivation for `validate` is convenience:

- Avoid cluttering the data directory with test environments
- Provide quick feedback during config development
- Enable CI/CD validation without state management

A `--dry-run` flag would check state-dependent conditions (like name conflicts), but since `create` is easily reversible, the value is lower than for destructive operations like `provision`.

#### What `--dry-run` Implies

When users see a `--dry-run` flag, they have specific expectations:

1. **Same code path**: It runs the same code as the real command, stopping just before irreversible actions
2. **Environment interaction**: It may check ports, permissions, filesystem, network connectivity
3. **Predictive**: If dry-run succeeds, the real command is **very likely** to succeed
4. **Shows a plan**: Often displays what _would_ be created/modified

This is a **strong promise**. If we cannot deliver on it, we should not use this pattern.

#### What `validate` Implies

A `validate` command communicates:

1. **Static analysis**: Checks rules, schemas, invariants without touching the outside world
2. **Pure and deterministic**: Same input always produces same output
3. **No promises about execution**: A valid config may still fail at runtime due to external factors
4. **Safe for automation**: Ideal for CI, editors, pre-commit hooks

This is an **honest promise** that matches what our feature actually does.

#### Why `validate` is the Right Choice for This Feature

Our validation:

| Characteristic                             | Our Implementation |
| ------------------------------------------ | ------------------ |
| Checks schemas and invariants              | ✅ Yes             |
| Checks cross-field rules                   | ✅ Yes             |
| Requires filesystem access (beyond config) | ❌ No              |
| Requires network access                    | ❌ No              |
| Guarantees `create` will succeed           | ❌ No              |

Since we:

- Do NOT touch the outside world
- Do NOT exercise the full execution path
- Cannot guarantee operational success

A `--dry-run` flag would **overpromise**. Users would be surprised when `create --dry-run` succeeds but `create` fails due to SSH key issues, port conflicts, or provider connectivity problems.

#### Alignment with DDD Boundaries

This decision aligns well with Domain-Driven Design:

- **`validate`** operates at the **domain layer** - checking business rules and invariants
- **`--dry-run`** would need to operate at the **infrastructure layer** - checking external resources

Keeping these separate maintains clean architectural boundaries.

#### Future Consideration: Adding `--dry-run` Later

A `--dry-run` flag may be valuable in the future for commands with significant side effects. Consider:

| Command     | Side Effects             | `--dry-run` Value              |
| ----------- | ------------------------ | ------------------------------ |
| `create`    | Internal state only      | Low - easily reversible        |
| `provision` | Creates VMs, costs money | High - irreversible, costly    |
| `destroy`   | Deletes infrastructure   | High - irreversible, data loss |

If we add `--dry-run` to `create`, it would:

- Check if environment name already exists (fail if conflict)
- Verify SSH key file is accessible
- Validate provider connectivity
- Show what would be persisted

But this is a **separate feature** with different semantics. The user expectation would be:

> "If `create --dry-run` succeeds, then `create` will succeed."

We should only add it if we can honor that promise.

**Mental model for users**:

- `validate` → "Is this configuration intrinsically valid?" (pure, deterministic)
- `create --dry-run` → "Will `create` succeed right now?" (state-dependent, future feature)
- `create` → "Create the environment"

**Decision**: Standalone `validate` command

**Rationale**:

- Validates config-intrinsic rules only (levels 1-2)
- Treats config as if it were the first environment (no state dependencies)
- Pure and deterministic - same input always produces same output
- Does not check conditions that vary by data directory
- Ideal for CI, editors, and rapid iteration
- Keeps the CLI honest about what it checks
- `--dry-run` is less critical for `create` since it's easily reversible

### Alternatives Considered

#### Option 1: `--dry-run` flag on `create` command

- **Pros**: Familiar pattern, shows intent ("what would happen")
- **Cons**: Would overpromise executability, may confuse users about what's being checked
- **Decision**: Rejected - wrong semantic for static validation

#### Option 2: Separate `validate` command ✅ CHOSEN

- **Pros**: Clear intent, easy to discover, follows single responsibility, honest about what it checks
- **Cons**: Another command to learn (minor)
- **Decision**: Chosen as the right tool for static validation

#### Option 3: Both options

- **Pros**: Maximum flexibility
- **Cons**: Confusing, `--dry-run` would overpromise for this use case
- **Decision**: Rejected - keep it simple, add `--dry-run` only if needed for operational checks later

## 🔧 Implementation Details

### Architecture Changes

This feature reuses the existing validation logic from `CreateCommandHandler` but stops before persisting. The implementation approach is:

1. The `CreateCommandHandler` creates an `Environment` from the config file
2. Only if validation passes does it persist the environment
3. The new `ValidateCommandHandler` does step 1 but skips step 2

### Component Design

#### Component 1: ValidateCommandHandler (Application Layer)

**Purpose**: Validates environment configuration without side effects

**Location**: `src/application/command_handlers/validate/`

**Interface**:

```rust
pub struct ValidateCommandHandler {
    // No dependencies on state repositories
}

impl ValidateCommandHandler {
    pub fn execute(&self, ctx: &ExecutionContext, config_path: &Path) -> Result<(), Error> {
        // 1. Load config file
        // 2. Parse JSON and validate syntax
        // 3. Construct domain types (validates invariants)
        // 4. Check config-intrinsic semantic rules
        // 5. Report success/failure to user
        // 6. Return appropriate exit code
        //
        // NOTE: NO STATE ACCESS - does not read data directory
        // NOTE: NO PERSISTENCE - does not write anything
        // NOTE: Treats config as if it were the first environment
    }
}
```

**Key differences from CreateCommandHandler**:

- Does not access the environment repository (no state-dependent checks)
- Does not persist anything to disk
- Does not check for name conflicts

#### Component 2: ValidateCommand (Presentation Layer)

**Purpose**: CLI command handler for validation

**Interface**:

```rust
pub struct ValidateCommand {
    env_file: PathBuf,
}

impl ValidateCommand {
    pub fn execute(&self, ctx: &ExecutionContext) -> Result<(), Error> {
        // 1. Load config file
        // 2. Validate using EnvironmentConfigValidator
        // 3. Report results to user
        // 4. Return appropriate exit code
    }
}
```

### Data Model

No new data models required - uses existing `EnvironmentConfig` types.

### API Changes

New CLI command:

```bash
# Primary usage
torrust-tracker-deployer validate --env-file <path>

# Optional: with verbose output
torrust-tracker-deployer validate --env-file <path> --verbose

# Optional: JSON output for programmatic use
torrust-tracker-deployer validate --env-file <path> --output json
```

### Configuration

No new configuration required.

## 📊 Impact Analysis

### Files to Modify

| File Path                                  | Changes Required                  | Effort |
| ------------------------------------------ | --------------------------------- | ------ |
| `src/application/command_handlers/`        | Add validate command handler      | Medium |
| `src/application/command_handlers/create/` | Extract validation logic          | Medium |
| `src/presentation/cli/`                    | Add validate subcommand           | Low    |
| `docs/user-guide/commands/`                | Document validate command         | Low    |
| `docs/console-commands.md`                 | Add validate to command reference | Low    |

### Breaking Changes

None - this is a new additive feature.

### Performance Impact

Neutral - validation is fast and doesn't involve external resources.

### Security Considerations

None - this feature only reads files and validates them without network access or state changes.

## � Open Questions (To Resolve During Implementation)

The following items will be defined or discovered during implementation:

### Exit Codes

Define explicit exit codes for different outcomes:

- Exit code 0 = valid config
- Exit code 1 = invalid config (validation errors)
- Exit code 2 = file not found / IO error (distinguish from validation failure)

_To be finalized when creating the implementation issue._

### Error Message Format

Document concrete examples of error messages for common validation failures:

- Missing required field
- Invalid field type
- Cross-field rule violation (e.g., Grafana without Prometheus)
- Unknown provider type

_To be designed during implementation._

### Validation Rules Inventory

Catalog the existing validation rules by level:

| Level | Rule Type        | Examples                                 | Count |
| ----- | ---------------- | ---------------------------------------- | ----- |
| 1     | Syntactic        | Required fields, type checking           | TBD   |
| 2     | Config-intrinsic | Grafana requires Prometheus, port ranges | TBD   |
| 3     | State-dependent  | Name conflicts, file existence           | TBD   |

_To be discovered by analyzing `CreateCommandHandler` implementation._

### Code Reuse Strategy

How to share validation logic with `CreateCommandHandler`:

- **Option A**: Extract shared validation function that both commands call
- **Option B**: Refactor `CreateCommandHandler` to separate validation from persistence
- **Option C**: Call into domain type constructors (which already validate invariants)

_To be determined during implementation. May require refactoring to disable/extract level 3 rules for the `validate` command._

### Existing State-Dependent Checks

Identify any level 3 (state-dependent) validations currently mixed into the `create` flow:

- Name conflict checking
- Any other checks that read from data directory or external resources

_These will need to be bypassed or extracted for the `validate` command._

## �🗓️ Implementation Plan

### Phase 1: Core Validation Logic

- [ ] Extract validation logic from `create` command
- [ ] Create `EnvironmentConfigValidator` component
- [ ] Add unit tests for validation logic

### Phase 2: CLI Command

- [ ] Add `validate` subcommand to CLI
- [ ] Implement command handler
- [ ] Add integration tests

### Phase 3: Documentation and Polish

- [ ] Update user guide with validate command
- [ ] Update console-commands.md
- [ ] Add E2E test verifying no state changes
- [ ] Add verbose and JSON output modes (if approved)

## ✅ Definition of Done

- [ ] `validate` command exists and works
- [ ] Valid configurations return exit code 0 with success message
- [ ] Invalid configurations return non-zero exit code with clear errors
- [ ] No state changes occur during validation (verified by test)
- [ ] Command has proper help text
- [ ] User documentation updated
- [ ] Unit and integration tests pass
- [ ] E2E test confirms no side effects

## 🧪 Testing Strategy

### Unit Tests

- Validation logic correctly identifies valid configs
- Validation logic reports all errors in invalid configs
- Error messages are clear and actionable

### Integration Tests

- Command parses arguments correctly
- Command reads config file
- Command reports results appropriately

### E2E Tests

- Validate command with valid config: success, no state changes
- Validate command with invalid config: failure, no state changes
- Verify `data/` directory unchanged after validation
