# Decision: Test Command as Smoke Test for Running Services

## Status

Accepted

## Date

2025-11-19

## Context

The `test` command was initially implemented with validation steps that check individual infrastructure components (cloud-init completion, Docker installation, Docker Compose installation). This led to ambiguity about the command's true purpose:

1. **Infrastructure Validation Tool**: Check that each deployment step completed successfully by validating individual components
2. **Smoke Test**: Verify that the deployed Tracker application is running and accessible (the real goal)

The current implementation validates infrastructure components rather than the deployed application itself. This creates confusion because:

- It tests infrastructure prerequisites, not the actual application
- It doesn't align with typical smoke test patterns (quick health checks after deployment)
- It's not what a human would do to verify a deployment (they'd check if the service responds to requests)
- The validation steps exist because the full deployment workflow isn't implemented yet

## Decision

**The `test` command is a smoke test for running Tracker services, not an infrastructure validation tool.**

The command's purpose is to verify that the deployed Tracker application is running and accessible after a complete deployment. Once the full deployment workflow is implemented (`Released` and `Running` states), the test command will:

1. Make HTTP requests to the publicly exposed Tracker services
2. Verify services respond correctly (health check endpoints, basic API calls)
3. Confirm the deployment is production-ready from an end-user perspective

**Current Implementation Status**: Work in Progress

The current validation steps (cloud-init, Docker, Docker Compose) are **temporary scaffolding** that exist only because:

- The complete deployment workflow is not yet implemented
- The `Running` state doesn't exist yet
- The actual Tracker application deployment hasn't been implemented

These steps will be **removed** when the full deployment is implemented and replaced with actual smoke tests of the running services.

## Consequences

### Positive

- **Clear Purpose**: Command has a well-defined role as a post-deployment smoke test
- **User-Focused**: Tests what users care about (is the service working?) not implementation details
- **Production-Ready Pattern**: Aligns with industry standard smoke testing practices
- **Simplified Debugging**: If smoke tests fail, users know the deployment failed (not which specific step)

### Negative

- **Limited Utility Now**: Current temporary implementation doesn't match the intended purpose
- **Missing Diagnostic Value**: When removed, won't help diagnose which deployment step failed
- **Requires Full Implementation**: True value only realized when complete deployment workflow exists

### Implementation Impact

**Now (Temporary)**:

```rust
// Current scaffolding - validates infrastructure prerequisites
ValidateCloudInitCompletionStep::new(ssh_config.clone()).execute().await?;
ValidateDockerInstallationStep::new(ssh_config.clone()).execute().await?;
ValidateDockerComposeInstallationStep::new(ssh_config).execute().await?;
```

**Future (Target)**:

```rust
// Real smoke tests - validates running application
HealthCheckStep::new(tracker_url).execute().await?;
BasicApiTestStep::new(tracker_url).execute().await?;
MetricsEndpointCheckStep::new(tracker_url).execute().await?;
```

**State Requirements**:

- **Current**: Requires `Configured` state (minimum) - because we validate Docker/Compose installation
- **Future**: Requires `Running` state - because we validate the running application

## Alternatives Considered

### Alternative 1: Infrastructure Validation Tool

Make the command validate infrastructure components at each deployment stage:

- Validate what's possible based on current state
- Help diagnose deployment failures
- Provide detailed infrastructure health checks

**Rejected because**:

- Creates tool overlap with debugging/diagnostic commands
- Not a smoke test - it's a diagnostic tool
- Adds complexity to determine what to test based on state
- Users want to know "is it working?" not "which step completed?"

### Alternative 2: State-Aware Flexible Testing

Run different validation sets based on current environment state:

- `Provisioned`: Test cloud-init, SSH connectivity
- `Configured`: Test Docker, Docker Compose
- `Running`: Test application endpoints

**Rejected because**:

- Dilutes command purpose - does too many things
- Complex state-based branching logic
- Confuses users about when to run tests
- Better handled by separate diagnostic commands if needed

## Related Decisions

- [Command State Return Pattern](./command-state-return-pattern.md) - Type-safe state transitions
- [Actionable Error Messages](./actionable-error-messages.md) - Clear user guidance on failures
- [Type Erasure for Environment States](./type-erasure-for-environment-states.md) - Runtime state handling

## References

- Issue [#184](https://github.com/torrust/torrust-tracker-deployer/issues/184) - Test Command Handler Refactoring
- `src/application/command_handlers/test/` - Current implementation
- Smoke Testing Best Practices: Quick post-deployment sanity checks to verify basic functionality
- Industry Pattern: Smoke tests validate the system works end-to-end, not individual components
