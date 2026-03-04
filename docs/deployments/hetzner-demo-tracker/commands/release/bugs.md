# Release Command — Bugs

## Bug: `release` fails when deployer runs inside Docker (docker not in PATH)

**Status**: Fixed in this branch (pending merge)
**Related PR**: [#384](https://github.com/torrust/torrust-tracker-deployer/pull/384) introduced the local validator
**Severity**: High — `release` is completely broken when running the deployer via Docker container

### Symptom

Running the `release` command via the Docker container fails immediately with:

```text
❌ Release command failed: Release command failed: Template rendering failed:
Docker Compose template rendering failed: Rendered docker-compose.yml failed
local validation: Failed to run 'docker compose config --quiet' — is Docker
installed and in PATH?
Source: No such file or directory (os error 2)
```

The environment transitions to `ReleaseFailed` state and the deployment cannot continue.

### Root Cause

PR [#384](https://github.com/torrust/torrust-tracker-deployer/pull/384) added a
local validation step that runs `docker compose config --quiet` against the rendered
`docker-compose.yml` before uploading it to the remote server. This is a useful check
when the deployer runs natively on a machine where Docker is installed.

However, the standard production usage is to run the deployer **inside a Docker
container** (`torrust/tracker-deployer:latest`). The deployer container does not
contain a `docker` binary — it has no need to run Docker locally. When `docker` is not
in PATH, the OS returns `ENOENT` (error 2, "No such file or directory"), which the
validator treated as a hard failure.

### Fix Applied

The validator in
`src/infrastructure/templating/docker_compose/local_validator.rs` was updated to
handle `io::ErrorKind::NotFound` gracefully: when `docker` is not in PATH, validation
is **skipped** with a warning log rather than failing the command.

The warning logged when running inside a container:

```text
WARN Skipping local docker-compose.yml validation: 'docker' is not available
in PATH (deployer may be running inside a container). The rendered file will be
validated by Docker Compose on the remote host.
```

Any other OS error (e.g. `PermissionDenied`) is still treated as a hard failure,
since that indicates a real system problem.

### State Recovery Applied

After the failed `release`, the environment state was manually reset from
`ReleaseFailed` back to `Configured` so that `release` could be retried:

```bash
# Before resetting: back up the failed state
cp data/torrust-tracker-demo/environment.json \
  data/torrust-tracker-demo/environment.json.release-failed-bak

# Reset to Configured (the state serialized as {"Configured": {"context": ..., "state": null}})
python3 -c "
import json
with open('data/torrust-tracker-demo/environment.json') as f:
    data = json.load(f)
context = data['ReleaseFailed']['context']
new_data = {'Configured': {'context': context, 'state': None}}
with open('data/torrust-tracker-demo/environment.json', 'w') as f:
    json.dump(new_data, f, indent=2)
"
```

This is the manual state recovery approach described in
[observations.md](../../observations.md#potential-manual-recovery-via-state-snapshot-untested).
It was safe here because:

- The failure happened at template rendering, **before** any remote action was taken
- The server state was not modified at all during the failed `release` attempt
