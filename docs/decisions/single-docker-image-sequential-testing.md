# Decision: Single Docker Image for Sequential E2E Command Testing

## Status

✅ Accepted

## Date

2025-12-10

## Context

When designing the E2E testing architecture for deployment workflow tests, we initially planned to create multiple Docker images representing different deployment phases:

- `provisioned-instance` - Post-provision state (base system ready)
- `configured-instance` - Post-configure state (dependencies installed)
- `released-instance` - Post-release state (applications deployed)
- `running-instance` - Post-run state (services started)

This multi-image approach would theoretically allow:

- **Isolated phase testing**: Test individual commands (configure, release, run, test) independently
- **Parallel test execution**: Run E2E tests for different commands in parallel
- **Clear phase boundaries**: Each image captures the exact state after a specific deployment phase

However, implementing and maintaining this architecture presented significant challenges:

1. **High Maintenance Overhead**: Every code change affecting any deployment phase requires updating multiple Docker images
2. **Image Synchronization**: Keeping all phase images in sync with code changes is error-prone and time-consuming
3. **Build Time**: Building multiple Docker images sequentially would be slower than running commands sequentially in a single container
4. **Parallel Execution Overhead**: Even with parallel tests, the Docker build and startup time for multiple images outweighs the benefits
5. **Complexity**: Managing multiple Dockerfiles, build dependencies, and test orchestration adds significant complexity
6. **Duplication**: Much of the image content would be duplicated across phases (base system, users, SSH setup)

The fundamental trade-off is between **test isolation/parallelism** (multiple images) versus **maintainability/simplicity** (single image).

## Decision

We will use a **single Docker image** (`provisioned-instance`) representing the pre-provisioned instance state, and run all deployment commands **sequentially** within that container during E2E tests.

### Implementation Details

**Single Image Approach**:

```text
docker/provisioned-instance/
├── Dockerfile              # Ubuntu 24.04 LTS + SSH + torrust user
├── supervisord.conf        # Process management
├── entrypoint.sh          # Container initialization
└── README.md              # Documentation
```

**Sequential Command Execution**:

```rust
// E2E test workflow (simplified)
async fn run_deployment_workflow_tests() -> Result<()> {
    // 1. Start single container (provisioned state)
    let container = start_provisioned_container().await?;

    // 2. Run commands sequentially
    run_create_command()?;
    run_register_command(container.ip())?;
    run_configure_command()?;    // Modifies container state
    run_release_command()?;      // Modifies container state
    run_run_command()?;          // Modifies container state
    run_test_command()?;         // Validates container state

    // 3. Cleanup
    container.stop().await?;
    Ok(())
}
```

### Trade-offs Accepted

**✅ Benefits**:

- **Low Maintenance**: Single Dockerfile to maintain - changes propagate automatically
- **Simpler Architecture**: Clear, understandable test flow
- **Faster Overall**: Sequential execution in one container is faster than building/starting multiple images
- **Easy Debugging**: Single container lifecycle to understand and inspect
- **Code Synchronization**: Image changes automatically reflect code changes via Ansible playbooks

**❌ Trade-offs**:

- **No Command Isolation**: Cannot test individual commands independently (must run full sequence)
- **No Test Parallelism**: Cannot run E2E tests for different commands in parallel
- **State Accumulation**: Later commands see state from earlier commands (intentional - tests real workflow)
- **Longer Test Runs**: If one command fails, must re-run entire sequence

## Consequences

### Positive

1. **Reduced Complexity**: Single Dockerfile, single container, single test flow
2. **Better Maintainability**: Code changes automatically tested via playbooks without image rebuilds
3. **Realistic Testing**: Sequential execution matches real deployment workflow exactly
4. **Faster Iteration**: No need to rebuild multiple images during development
5. **Lower CI Resources**: Single container uses fewer resources than multiple containers
6. **Simplified Debugging**: `--keep` flag allows inspection of final container state with all commands applied

### Negative

1. **Test Coupling**: Commands cannot be tested in isolation - must test full workflow
2. **Longer Feedback**: Must run entire sequence to test later commands
3. **No Parallel Speedup**: Cannot leverage parallel test execution for E2E workflow tests

### Risk Mitigation

The negative consequences are mitigated by:

- **Unit Tests**: Individual command logic is tested in isolation via unit tests
- **Integration Tests**: Command interfaces are tested without full E2E overhead
- **Fast Execution**: Sequential execution in Docker is still fast (~48 seconds total)
- **Split Test Suites**: Infrastructure tests run separately, allowing some parallelism at the suite level

## Alternatives Considered

### Alternative 1: Multi-Image Phase Architecture (Original Plan)

**Approach**: Build separate Docker images for each deployment phase (provisioned, configured, released, running).

**Pros**:

- Command isolation - test individual commands independently
- Parallel test execution possible
- Clear phase boundaries

**Cons**:

- High maintenance overhead - must update multiple images for code changes
- Slower build time - building 4 images takes longer than running 4 commands
- Complex orchestration - managing image dependencies and build order
- Image synchronization issues - keeping images in sync with code
- Higher CI resource usage

**Rejected Because**: Maintenance overhead outweighs benefits. Build time for multiple images exceeds sequential execution time.

### Alternative 2: Docker Compose Multi-Service Setup

**Approach**: Use Docker Compose to orchestrate multiple containers representing different phases.

**Pros**:

- Service isolation
- Declarative configuration
- Can leverage Docker Compose features

**Cons**:

- Even higher complexity than multi-image
- Still requires building/maintaining multiple images
- Orchestration overhead
- Harder to debug

**Rejected Because**: Adds orchestration complexity without solving the fundamental maintenance problem.

### Alternative 3: Container Snapshots Between Commands

**Approach**: Start with one image, create container snapshots after each command, test from snapshots.

**Pros**:

- Single base image
- Can jump to any phase via snapshot
- Some test isolation

**Cons**:

- Snapshot management complexity
- Storage overhead for snapshots
- Non-standard testing approach
- Still requires careful state management

**Rejected Because**: Complexity doesn't justify the limited benefits. Snapshots add non-standard workflow.

## Related Decisions

- [Docker Testing Evolution](./docker-testing-evolution.md) - Evolution from Docker rejection to hybrid approach for E2E testing
- [E2E Test Split Architecture](../e2e-testing.md#architecture) - Split between infrastructure and deployment workflow tests

## References

- [E2E Testing Guide - Docker Architecture](../e2e-testing.md#docker-architecture-for-e2e-testing)
- [Provisioned Instance Documentation](../../docker/provisioned-instance/README.md)
- GitHub Actions E2E Deployment Workflow: `.github/workflows/test-e2e-deployment.yml`
- E2E Deployment Workflow Tests: `src/bin/e2e_deployment_workflow_tests.rs`
