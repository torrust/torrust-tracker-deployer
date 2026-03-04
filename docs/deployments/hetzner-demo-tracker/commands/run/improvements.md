# Run Command — Improvements

## Improvement: `Running` state does not guarantee services are healthy

**Status**: Documented — existing `test` command is the recommended solution
**Type**: Clarification / UX

### Observation

After the `run` command completes and the environment transitions to `Running`,
the individual containers are not necessarily healthy. In this deployment the
state became `Running` while the tracker container was stuck in a restart loop
due to [Bug 3](bugs.md#bug-3-mysql-password-is-not-url-encoded-in-the-tracker-connection-string).
The deployer gave no indication that anything was wrong.

The `Running` state only means that `docker compose up -d` exited with code 0 —
i.e. Docker accepted the request to start the stack. It says nothing about
whether each service is actually reachable and functioning.

### Why the `run` Command Does Not Wait for Health

Waiting inside `run` for all containers to become healthy is difficult in
practice because:

- Health checks vary per service and are defined in `docker-compose.yml` with
  service-specific commands and timeouts.
- Some services (e.g. Caddy obtaining TLS certificates) can take tens of seconds
  or minutes to become fully operational depending on DNS propagation and the
  ACME provider.
- The deployer has no deep knowledge of what "healthy" means for each service
  beyond what Docker's own health-check reports, and Docker's health-check for
  the tracker does not distinguish between "still starting" and "crashed".

Blocking the `run` command until everything is provably healthy would require
duplicating the logic that is already expressed in the `test` command (smoke
tests).

### Recommended Approach

The deployer already provides a separate `test` command that runs smoke tests
against the deployed stack. That is the right tool for verifying that services
are actually reachable and responding correctly after `run` completes.

The intended workflow is:

```text
release  →  run  →  test
```

`run` starts the stack; `test` confirms it works. Operators should always run
`test` after `run` and treat a passing `test` as the definitive signal that the
deployment is functional.

### Possible Future Improvement

A lightweight post-start check could be added to `run` that waits for Docker's
own health status (not full smoke-test level) to settle — for example polling
`docker compose ps` until no container is in `starting` or `restarting` state,
with a configurable timeout. This would catch fast-failing containers like the
tracker URL-encoding crash without requiring the full `test` logic inside `run`.

This should be considered only if the cost of running `test` immediately after
`run` becomes a significant friction point for operators.
