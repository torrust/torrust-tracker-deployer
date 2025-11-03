# Decision: Environment Variable Prefix Convention

## Status

Accepted

## Date

2025-11-03

## Context

The Torrust Tracker Deployer application needs a consistent naming convention for environment variables. This is important for:

- **Project identification**: Making it clear which project's variables are being used in shared environments
- **Namespace collision avoidance**: Preventing conflicts with system variables or other applications
- **Ecosystem consistency**: Aligning with other Torrust projects for a unified developer experience
- **Discoverability**: Making it easy to find all project-related environment variables (e.g., using `env | grep TORRUST_TD_`)

Without a standardized prefix, environment variables could conflict with other applications or be difficult to identify as belonging to the Torrust Tracker Deployer project.

## Decision

All environment variables used by the Torrust Tracker Deployer application will use the prefix **`TORRUST_TD_`**.

**Prefix breakdown:**

- `TORRUST_` - Identifies variables belonging to the Torrust ecosystem
- `TD_` - Stands for "Tracker Deployer", identifying this specific project within the ecosystem

**Example:**

```bash
TORRUST_TD_SKIP_FIREWALL_IN_CONTAINER=true
```

This is currently the only environment variable used by the application. Future environment variables will follow this same prefix convention.

## Consequences

### Positive

- **Clear project identification**: Anyone seeing these variables immediately knows they belong to Torrust Tracker Deployer
- **Ecosystem consistency**: All Torrust projects share the `TORRUST_` prefix, creating a unified namespace
- **No namespace collisions**: The specific `TORRUST_TD_` prefix prevents conflicts with system variables or other applications
- **Easy filtering**: Variables can be easily listed with `env | grep TORRUST_TD_` or similar commands
- **Reasonable length**: The prefix is concise enough to avoid excessive verbosity while remaining descriptive

### Negative

- **Slightly verbose**: The prefix adds 11 characters to every environment variable name
- **Potential for confusion**: New contributors might initially be unclear what "TD" stands for (mitigated by documentation)
- **Migration effort**: Existing code using different variable names will need to be updated

## Alternatives Considered

### Option 1: `TORRUST_TRACKER_DEPLOYER_` (Full Project Name)

**Pros:**

- Maximally explicit - no ambiguity about what "TD" means
- Self-documenting

**Cons:**

- **Too verbose**: 25 characters for the prefix alone
- Results in very long variable names (e.g., `TORRUST_TRACKER_DEPLOYER_CONFIG_PATH`)
- Cumbersome to type and read

**Decision:** Rejected due to excessive length

### Option 2: `TRACKER_DEPLOYER_` (No Ecosystem Prefix)

**Pros:**

- Shorter than full prefix
- Still identifies the project

**Cons:**

- **Lacks ecosystem consistency**: Doesn't align with other Torrust projects
- **Less distinctive**: "TRACKER_DEPLOYER" is more generic than "TORRUST_TD"
- **Harder to identify**: Not immediately clear it's part of the Torrust ecosystem

**Decision:** Rejected due to lack of ecosystem alignment

### Option 3: `TT_DEPLOYER_` (Abbreviated Both Parts)

**Pros:**

- Very short (11 characters)
- Still somewhat identifiable

**Cons:**

- **Ambiguous abbreviation**: "TT" could mean many things (not obviously "Torrust Tracker")
- **Inconsistent with ecosystem**: Other Torrust projects may not use similar abbreviations
- **Less discoverable**: Harder to guess or search for

**Decision:** Rejected due to ambiguity

## Related Decisions

- None yet - this is the first ADR establishing environment variable conventions

## References

- [The Twelve-Factor App - Config](https://12factor.net/config) - Best practices for configuration management
- [Environment Variable Naming Conventions](https://systemd.io/ENVIRONMENT/) - System-wide conventions
- Torrust ecosystem conventions (to be documented)
