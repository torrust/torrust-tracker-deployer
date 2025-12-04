# ADR: Docker-in-Docker Support for E2E Tests

## Status

Proposed

## Context

The `run` command starts Docker Compose services on the target VM. To test this in the
E2E configuration tests (`e2e_config_and_release_tests.rs`), we need Docker to be
available inside the test container that simulates a provisioned VM.

Currently, the `provisioned-instance` Docker container does not have Docker installed
or available, so the `run` command cannot be tested. We added a workaround
(`TORRUST_TD_SKIP_RUN_IN_CONTAINER`) to skip the `run` command in container-based tests,
but this means we cannot verify the complete deployment workflow in E2E tests.

### Requirements

1. Docker Compose must be able to run inside the test container
2. The solution must work on GitHub Actions CI/CD
3. The solution should not significantly increase test time
4. Security considerations should be documented

## Decision Drivers

- **Test Coverage**: We want to test the complete deployment workflow including `run`
- **CI Compatibility**: Must work on GitHub Actions runners
- **Simplicity**: Prefer simpler solutions that are easier to maintain
- **Security**: Understand and document security implications

## Considered Options

### Option 1: Docker-in-Docker (DinD) with Privileged Mode

Run a full Docker daemon inside the container using `--privileged` mode.

**Changes Required**:

1. Install Docker daemon in `docker/provisioned-instance/Dockerfile`
2. Add Docker daemon to supervisor configuration
3. Run container with `--privileged` flag in testcontainers
4. Wait for Docker daemon to be ready before running tests

**Pros**:

- Complete isolation - containers created inside are truly nested
- Realistic simulation of a VM with Docker installed
- No side effects on host Docker

**Cons**:

- Requires `--privileged` flag (security concern)
- Docker daemon startup adds ~5-10 seconds to test time
- More complex supervisor configuration
- May have issues on some CI environments

### Option 2: Docker Socket Mounting (DooD)

Mount the host's Docker socket (`/var/run/docker.sock`) into the container.

**Changes Required**:

1. Install Docker CLI (not daemon) in `docker/provisioned-instance/Dockerfile`
2. Mount Docker socket when starting container in testcontainers
3. Add `torrust` user to `docker` group

**Pros**:

- Simpler setup - no daemon to manage
- Faster - no Docker daemon startup time
- No `--privileged` flag needed
- Works reliably on most CI environments

**Cons**:

- Containers created are siblings, not children
- Shares host Docker daemon (potential resource conflicts)
- Container names may conflict with host containers
- Less isolation

### Option 3: Use Real VMs for Complete Testing (Current Approach)

Keep the skip flag and only test `run` command with real LXD VMs.

**Changes Required**:

- None - keep current implementation
- Document that `run` is only tested in `e2e_tests_full.rs` and manual tests

**Pros**:

- No additional complexity
- Tests real VM behavior
- No security concerns

**Cons**:

- Incomplete E2E test coverage in container-based tests
- `run` command not tested in CI

## Decision

### Chosen Option: Option 1 - Docker-in-Docker (DinD) with Privileged Mode

We choose DinD because:

1. **Realistic Testing**: The container accurately simulates a VM with Docker installed
2. **Complete Isolation**: No interference with host Docker or other tests
3. **CI Compatibility**: GitHub Actions supports privileged containers
4. **Consistent Behavior**: Same Docker version inside container regardless of host

The `--privileged` flag is acceptable for test containers because:

- Test containers are ephemeral and isolated
- No untrusted code runs inside
- GitHub Actions already runs with elevated privileges

## Implementation Plan

### Phase 1: Update Dockerfile

Add Docker installation to `docker/provisioned-instance/Dockerfile`:

```dockerfile
# Install Docker
RUN curl -fsSL https://get.docker.com | sh \
    && usermod -aG docker torrust
```

### Phase 2: Update Supervisor Configuration

Add Docker daemon to `supervisord.conf`:

```ini
[program:dockerd]
command=/usr/bin/dockerd
priority=10
autostart=true
autorestart=true
```

### Phase 3: Update Container Startup

Modify testcontainers configuration to:

1. Use `--privileged` flag
2. Wait for Docker daemon to be ready (check `docker info`)

### Phase 4: Remove Skip Flag

Remove `TORRUST_TD_SKIP_RUN_IN_CONTAINER` environment variable and enable `run` command
testing in `e2e_config_and_release_tests.rs`.

## Consequences

### Positive

- Complete E2E test coverage for all commands including `run`
- Tests verify real Docker Compose behavior
- CI/CD validates complete deployment workflow

### Negative

- Test containers require `--privileged` mode
- Slightly longer test startup time (~5-10 seconds)
- More complex container configuration

### Neutral

- Need to document privileged mode requirement
- May need to handle Docker daemon startup failures gracefully

## References

- [Docker-in-Docker Documentation](https://hub.docker.com/_/docker)
- [GitHub Actions Container Support](https://docs.github.com/en/actions/using-containerized-services/about-service-containers)
- [testcontainers-rs Privileged Mode](https://docs.rs/testcontainers/latest/testcontainers/)
