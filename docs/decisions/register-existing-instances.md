# Decision: Register Command for Existing Instances

## Status

Accepted

## Date

2025-11-28

## Context

The Torrust Tracker Deployer was designed to provision infrastructure (VMs) and then deploy applications to them. However, several use cases require deploying to pre-existing infrastructure:

1. **End users** may have spare servers, infrastructure from unsupported cloud providers, or custom setups they want to use
2. **E2E testing** needed a way to test configuration without the overhead of provisioning real VMs, leading to the creation of `run_provision_simulation.rs` - a hack that directly manipulated environment state

The existing workflow required users to go through the `provision` command, which creates new infrastructure via OpenTofu/LXD. There was no way to skip this step and use existing infrastructure.

### The Problem with run_provision_simulation

The `run_provision_simulation.rs` hack was problematic because:

- It bypassed the application layer, directly manipulating domain state
- It was not a real command, so it couldn't be used by end users
- It created technical debt and made the codebase harder to understand
- It didn't properly test the state transitions and workflows

### Key Insight

The `create environment` command creates the environment _concept_ (SSH credentials, name, configuration) - not the actual infrastructure. The infrastructure is either:

- **Provisioned** via the `provision` command (creates new VMs)
- **Registered** via the `register` command (uses existing infrastructure)

Both paths lead to the same `Provisioned` state, and the only runtime output from provisioning stored is the instance IP. Therefore, `register` only needs the instance IP as input.

## Decision

Implement a `register` command as a first-class alternative to `provision`:

```bash
# Standard workflow (new infrastructure)
torrust-tracker-deployer create environment -f config.json
torrust-tracker-deployer provision my-env

# Register workflow (existing infrastructure)
torrust-tracker-deployer create environment -f config.json
torrust-tracker-deployer register my-env --instance-ip 192.168.1.100
```

### Key Design Decisions

1. **Command name**: `register` chosen based on industry precedent (GitHub/GitLab runners, Consul services, Vault)

2. **Minimal input**: Only `--instance-ip` required because SSH credentials and other configuration already exist from `create environment`

3. **State transition**: Transitions `Created` → `Provisioned` (same target state as `provision`)

4. **Provision method tracking**: Environments track how they were provisioned via `provision_method` field:

   - `Provisioned` - Infrastructure created by `provision` command (managed)
   - `Registered` - Infrastructure provided by user via `register` command (external)

5. **Destroy behavior**: The `provision_method` determines what happens on `destroy`:

   - Provisioned environments: `destroy` removes the infrastructure
   - Registered environments: `destroy` only removes deployer data, preserves the instance

6. **Minimal validation**: v1 only validates SSH connectivity. Advanced validation (OS version, disk space, etc.) deferred to v2

## Consequences

### Positive

- **Clean separation**: "Creating the concept" (environment) vs "materializing infrastructure" (provision/register) are clearly separated
- **No duplication**: SSH credentials, environment name, etc. are set once in `create environment`
- **Extensible**: If `RuntimeOutputs` grows, `register` parameters grow accordingly
- **E2E testing**: Replaced the `run_provision_simulation` hack with a proper command
- **User flexibility**: Users can deploy to any infrastructure they have SSH access to
- **Safety**: Registered instances are protected from accidental destruction

### Negative

- **Additional complexity**: Another command to maintain and document
- **Limited validation**: v1 only validates SSH connectivity, may fail later in workflow
- **Two-step process**: Users must run `create environment` before `register` (can't do it in one step)

### Risks

- Users may not understand the difference between `provision` and `register`
- Registered instances may not meet all requirements, leading to failures in `configure` or `test`

## Alternatives Considered

### Alternative 1: Single `create environment --instance-ip` Command

Combine environment creation and instance registration into a single command.

**Rejected because**:

- Conflates two distinct operations (creating the concept vs providing infrastructure)
- Would require duplicating SSH credential handling
- Less flexible for future enhancements

### Alternative 2: `import` Command Name

Use `import` instead of `register`.

**Rejected because**:

- `import` often implies moving/converting data
- `register` has stronger industry precedent (GitHub runners, Consul, etc.)
- `register` better conveys the action of "associating" rather than "importing"

### Alternative 3: Skip Environment Creation for Register

Allow `register` to create the environment if it doesn't exist.

**Rejected because**:

- Would require `register` to handle SSH credentials
- Breaks the clean separation of concerns
- Makes the command more complex and harder to understand

## Related Decisions

- [command-state-return-pattern.md](command-state-return-pattern.md) - Pattern for command state transitions
- [type-erasure-for-environment-states.md](type-erasure-for-environment-states.md) - How environment states are managed

## References

- [GitHub Issue #203](https://github.com/torrust/torrust-tracker-deployer/issues/203) - Original issue
- [Feature Specification](../features/import-existing-instances/specification.md) - Full feature specification
- [Register Command User Guide](../user-guide/commands/register.md) - User documentation
