# ADR: Docker-in-Docker Support for E2E Tests

## Status

Accepted (Implemented)

## Context

The `run` command starts Docker Compose services on the target VM. To test this in the
E2E configuration tests (`e2e_config_and_release_tests.rs`), we need Docker to be
available inside the test container that simulates a provisioned VM.

Previously, the `provisioned-instance` Docker container did not have Docker installed
or available, so the `run` command could not be tested. We had a workaround
(`TORRUST_TD_SKIP_RUN_IN_CONTAINER`) to skip the `run` command in container-based tests,
but this meant we could not verify the complete deployment workflow in E2E tests.

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

### Phase 1: Update Dockerfile (Completed)

Add Docker installation to `docker/provisioned-instance/Dockerfile`:

```dockerfile
# Install iptables for Docker networking
RUN apt-get update && apt-get install -y iptables

# Install Docker using official installation script
# This installs both Docker daemon and Docker Compose plugin
RUN curl -fsSL https://get.docker.com | sh \
    && rm -rf /var/lib/apt/lists/*

# Create torrust user with docker group membership
RUN useradd -m -s /bin/bash -G sudo,docker torrust
```

### Phase 2: Update Supervisor Configuration (Completed)

Add Docker daemon to `supervisord.conf` with `vfs` storage driver:

```ini
[program:dockerd]
command=/usr/bin/dockerd --host=unix:///var/run/docker.sock --storage-driver=vfs
stdout_logfile=/var/log/supervisor/dockerd.log
stderr_logfile=/var/log/supervisor/dockerd.log
autorestart=true
startretries=3
priority=5
```

**Important**: The `vfs` storage driver is required because the default `overlay2` driver
does not work in nested Docker environments (fails with "invalid argument" error).
The `vfs` driver is slower but compatible with Docker-in-Docker scenarios.

### Phase 3: Update Container Startup (Completed)

Modify testcontainers configuration in `src/testing/e2e/containers/provisioned.rs`:

1. Use `with_privileged(true)` from `testcontainers::ImageExt` trait
2. Wait for Docker daemon to be ready by checking for `dockerd entered RUNNING state`
   in supervisor logs (instead of waiting for sshd)

```rust
// Wait for Docker daemon to be ready (not just SSH)
.with_wait_condition(WaitFor::message_on_stdout("dockerd entered RUNNING state"))

// Start with privileged mode for Docker-in-Docker
image.with_privileged(true).start().await
```

### Phase 4: Handle Docker Package Conflicts (Completed)

A challenge discovered during implementation: Ansible's Docker installation playbook
installs `docker.io` package, which conflicts with Docker CE (`containerd.io`) installed
via `get.docker.com` script in the Dockerfile.

**Solution**: Add environment variable `TORRUST_TD_SKIP_DOCKER_INSTALL_IN_CONTAINER`
to skip Docker/Docker Compose installation via Ansible when Docker is already
pre-installed in the container.

In `src/bin/e2e_config_and_release_tests.rs`:

```rust
// SAFETY: Set before async runtime starts
unsafe {
    std::env::set_var("TORRUST_TD_SKIP_DOCKER_INSTALL_IN_CONTAINER", "true");
}
```

In `src/application/command_handlers/configure/handler.rs`:

```rust
let skip_docker = std::env::var("TORRUST_TD_SKIP_DOCKER_INSTALL_IN_CONTAINER")
    .map(|v| v == "true")
    .unwrap_or(false);

if skip_docker {
    // Skip Docker and Docker Compose installation steps
}
```

### Phase 5: Remove Skip Flag (Completed)

Removed `TORRUST_TD_SKIP_RUN_IN_CONTAINER` environment variable since the `run` command
now works in container-based E2E tests.

## Consequences

### Positive

- Complete E2E test coverage for all commands including `run`
- Tests verify real Docker Compose behavior
- CI/CD validates complete deployment workflow

### Negative

- Test containers require `--privileged` mode
- Slightly longer test startup time (~2-3 seconds for Docker daemon)
- More complex container configuration
- Must use `vfs` storage driver (slower than `overlay2`)

### Neutral

- Need to document privileged mode requirement
- Docker installation skipped via environment variable to avoid package conflicts

## Lessons Learned

1. **Storage Driver Compatibility**: The `overlay2` storage driver does not work in
   nested Docker environments. The `vfs` driver must be used, which is slower but
   compatible.

2. **Package Conflicts**: Docker CE (`containerd.io` from `get.docker.com`) conflicts
   with `docker.io` package. When Docker is pre-installed in the container, Ansible's
   Docker installation must be skipped.

3. **Rust 1.81+ Safety**: `std::env::set_var` is now `unsafe` in Rust 1.81+. Environment
   variables must be set in an `unsafe` block with a safety comment explaining why
   concurrent access is not possible.

4. **Wait Conditions**: Waiting for `dockerd entered RUNNING state` in supervisor logs
   is more reliable than waiting for SSH, as it ensures Docker is ready before tests run.

## References

- [Docker-in-Docker Documentation](https://hub.docker.com/_/docker)
- [GitHub Actions Container Support](https://docs.github.com/en/actions/using-containerized-services/about-service-containers)
- [testcontainers-rs Privileged Mode](https://docs.rs/testcontainers/latest/testcontainers/)
- [Docker Storage Drivers](https://docs.docker.com/storage/storagedriver/select-storage-driver/)
- [Rust 1.81 set_var Safety](https://doc.rust-lang.org/stable/std/env/fn.set_var.html)
