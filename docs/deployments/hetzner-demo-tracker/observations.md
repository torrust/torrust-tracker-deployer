# Deployment Observations

Cross-cutting learnings and insights gathered during this deployment that apply to the deployer
in general, not to any specific command or step.

## Deployer State and Recovery

### No built-in failure recovery (by design)

The deployer has no mechanism to recover from a failed state. If a command fails (e.g.
`provision` fails halfway through), the environment is left in a failed state
(`ProvisioningFailed`, `ConfigureFailed`, etc.) and the only supported path forward is to clean
up and restart from scratch.

This is intentional — see
[post-provision/README.md § Design Notes](post-provision/README.md#design-notes-tradeoffs-in-setup-sequencing)
for the rationale (recovery complexity vs. fast server recreation).

### Potential manual recovery via state snapshot (untested)

> ⚠️ **Warning**: This approach has not been tested or verified. It is a theoretical recovery
> path. Only attempt it if you understand exactly why the command failed and are confident the
> server is in a state that can be manually completed. Getting this wrong may leave the
> environment in a worse, harder-to-diagnose state.

The deployer stores the environment state in a JSON file at:

```text
data/<environment-name>/environment.json
```

For this deployment:

```text
data/torrust-tracker-demo/environment.json
```

This file tracks the current state (`Provisioned`, `Configured`, `Released`, `Running`, etc.)
and metadata like the server IP and creation timestamp.

**Theoretical recovery procedure:**

1. **Before running a command**, take a snapshot of the state file:

   ```bash
   cp data/torrust-tracker-demo/environment.json \
     data/torrust-tracker-demo/environment.json.bak-before-configure
   ```

2. **If the command fails**, identify exactly what the command did before failing. For example:
   - `configure` installs Docker, Docker Compose, and then writes application config files.
     If it failed after installing Docker but before writing config files, Docker is installed
     but the config is incomplete.
   - `release` pulls Docker images and stages release artifacts. If it failed midway, some
     images may be present, others not.

3. **Manually complete or undo the partial work** on the server via SSH so the server is in a
   consistent state matching the target state of the command.

4. **Restore the pre-command snapshot**:

   ```bash
   cp data/torrust-tracker-demo/environment.json.bak-before-configure \
     data/torrust-tracker-demo/environment.json
   ```

5. **Retry the command.**

**When this is safe to attempt:**

- The failure reason is clear and the root cause is understood.
- The partial work done by the command is reversible or completable manually.
- You can verify the server state via SSH before retrying.

**When NOT to attempt this:**

- The failure cause is unknown.
- Infrastructure state (e.g. Hetzner resources created by OpenTofu) is out of sync with
  the local Tofu state files — in that case, restoring the environment JSON will not fix the
  inconsistency and may cause further problems.
- The command involved OpenTofu (`provision`, `destroy`) — Tofu manages its own state in
  `build/<env>/tofu/` and that state must also be consistent.
