# Decision: Handle Concurrent Docker Image Builds in Tests Gracefully

## Status

✅ Accepted

## Date

2026-02-13

## Context

Several integration tests require Docker images that are built on-demand
before the test runs. The image builders (`ImageBuilder::build_if_missing()`
and `ContainerImageBuilder::build()`) follow a check-then-build pattern:

1. Check if the image exists (`docker image inspect`)
2. If it does not exist, build it (`docker build -t <name>:<tag> ...`)

Since `cargo test` runs tests in parallel by default, multiple tests that
depend on the same Docker image can trigger the build simultaneously. This
creates a race condition:

1. **Test A** and **Test B** both call `build_if_missing()` concurrently
2. Both call `image_exists()` → both get `false` (no image yet)
3. Both start `docker build` in parallel (~60 seconds each)
4. **Test A** finishes first and tags the image → success
5. **Test B** finishes all build steps successfully, but the final
   export/tagging step fails because the tag already exists:

```text
#8 exporting to image
#8 exporting layers done
#8 naming to docker.io/library/dependency-installer-test:ubuntu-24.04 done
#8 ERROR: image "docker.io/library/dependency-installer-test:ubuntu-24.04": already exists
```

This error is misleading: all Docker build steps completed successfully.
The failure occurs only at the final naming/tagging step because another
concurrent build already claimed the tag. The image is valid and available
for use.

This manifested as flaky CI failures in GitHub Actions where
`dependency-installer` integration tests would intermittently fail with
`BuildFailed` errors despite the Docker image being correctly built. See
[issue 342](https://github.com/torrust/torrust-tracker-deployer/issues/342)
for the full debugging history.

### Additional caveat: image staleness in development

All tests that need the same Docker image share a single tagged image. This
works well in CI, where every workflow run starts with a clean Docker state
and builds a fresh image. However, during local development, if a developer
modifies the Dockerfile or its build context, the existing cached image
will not be rebuilt because `build_if_missing()` skips the build when the
tag already exists. Developers must manually delete the old image
(`docker rmi <name>:<tag>`) before running tests to pick up their changes.

## Decision

When a Docker build fails and the error output contains the string
`"already exists"`, treat it as **success** instead of propagating the
error. The image was built by a concurrent process and is available for
use.

### Implementation

In both image builders, after detecting a non-zero exit code from
`docker build`, check the error output before returning an error:

```rust
if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Concurrent build race: another test already tagged this image.
    // The image is available for use — this is not a real failure.
    if stderr.contains("already exists") {
        info!(
            image = full_image_name,
            "Docker image was built by a concurrent process, treating as success"
        );
        return Ok(());
    }

    // ... propagate real build errors as before
}
```

### Why this is correct

- The `"already exists"` error only occurs at the export/tagging step,
  **after** all build steps complete successfully
- It means the exact same image (same Dockerfile, same tag) was already
  built and tagged by a concurrent process
- The image is immediately available for container creation
- Docker tags are atomic pointers — no data loss or corruption is possible

### Affected files

- `packages/dependency-installer/tests/containers/image_builder.rs` —
  `build_if_missing()` method
- `src/testing/e2e/containers/image_builder.rs` —
  `build()` method

## Consequences

### Positive

- **CI stability**: Parallel tests no longer fail due to concurrent image
  builds — the flaky CI failure is eliminated
- **Simplicity**: Minimal code change (string check on error output) with
  no new dependencies or synchronization primitives
- **No performance impact**: No locks, no serialization of builds, no
  additional Docker commands
- **Preserves parallelism**: Tests continue to run in parallel without
  any coordination overhead

### Negative

- **String-based error detection**: The fix relies on matching the string
  `"already exists"` in Docker's error output. If Docker changes this
  message in a future version, the detection would stop working and the
  original error would resurface (but would not silently break — tests
  would fail visibly)
- **Redundant builds**: When the race occurs, both tests perform the full
  build (~60 seconds each), wasting CI time. Only the tagging of the
  second build is skipped. This is acceptable because the race is
  infrequent and the alternative solutions add complexity
- **Image staleness in development**: Developers who modify Dockerfiles
  or build contexts must manually remove old images before running tests.
  The `build_if_missing()` pattern does not detect changes to the build
  inputs

## Alternatives Considered

### 1. Tag images uniquely per test

Give each test a unique image tag (e.g., `test-image:test-<uuid>`) so
concurrent builds never collide on the same tag.

**Rejected because:**

- Pollutes the Docker tag namespace with many test-specific tags
- Requires cleanup logic to remove stale test images
- Each test would build its own image from scratch, significantly
  increasing total CI time (no sharing of built images between tests)
- Adds complexity to the test infrastructure for a problem that occurs
  only during the tagging step

### 2. Use a file-based lock to serialize builds

Use a lock file or `flock` to ensure only one test builds the image at a
time. Other tests wait for the lock, then find the image already exists.

**Rejected because:**

- Introduces a new synchronization primitive into the test infrastructure
- Cross-process file locks have portability concerns across operating
  systems
- Tests may have execution timeouts that could be exceeded while waiting
  for the lock, especially if the build takes a long time
- Adds complexity for a problem that has a simpler solution

### 3. Pre-build images before running tests

Add a build step (e.g., in CI workflow or a setup script) that builds all
required Docker images before `cargo test` runs.

**Rejected because:**

- Adds a mandatory setup step that developers must remember to run
- Couples the test execution to an external build step, making it harder
  to run tests in isolation
- Does not fully eliminate the race if a developer runs `cargo test`
  without the pre-build step
- The current on-demand build pattern is more ergonomic for development

### 4. Remove existing image before building (`docker rmi -f`)

Force-remove the tagged image before starting the build to ensure a
clean slate.

**Tried and rejected because:**

- Creates **worse** race conditions: Test A removes the image while
  Test B is using a container based on it
- Does not solve the fundamental concurrency problem — just shifts the
  race window
- Was the first approach attempted during the debugging of issue 342
  and was proven to make CI failures more frequent

## Related Decisions

- [Single Docker Image for Sequential E2E Command Testing](./single-docker-image-sequential-testing.md) —
  related decision about Docker image strategy for E2E tests
- [Docker Testing Evolution](./docker-testing-evolution.md) —
  broader context on Docker usage in the test infrastructure

## References

- [Issue 342: Fix Docker BuildKit "image already exists" error](https://github.com/torrust/torrust-tracker-deployer/issues/342)
- [PR 344: fix: resolve Docker BuildKit "image already exists" error in CI](https://github.com/torrust/torrust-tracker-deployer/pull/344)
