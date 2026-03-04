# Command: release

> **Status**: ✅ Done (2026-03-04)

## What `release` does

The `release` command:

1. Pulls the latest Docker images for the tracker and monitoring stack on the server.
2. Stages the release artifacts.
3. Marks the environment as `Released` on success.

It does **not** start the services — that is done by the `run` command.

## Command

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  release torrust-tracker-demo 2>&1 | tee -a data/logs/log.txt
```

## Output

```text
[2026-03-04 12:35:20] Starting Torrust Tracker Deployer container...
[2026-03-04 12:35:20] Verifying installed tools...
[2026-03-04 12:35:20] Tool versions:
[2026-03-04 12:35:20]   - OpenTofu: OpenTofu v1.11.5
[2026-03-04 12:35:20]   - Ansible: ansible [core 2.20.3]
[2026-03-04 12:35:20]   - SSH: OpenSSH_10.0p2 Debian-7, OpenSSL 3.5.4 30 Sep 2025
[2026-03-04 12:35:20]   - Git: git version 2.47.3
[2026-03-04 12:35:20] SSH directory found, checking permissions...
[2026-03-04 12:35:20] Container initialization complete. Executing command...
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: torrust-tracker-demo (took 0ms)
⏳ [2/2] Releasing application...
⏳   ✓ Application released successfully (took 133.5s)
{
  "environment_name": "torrust-tracker-demo",
  "instance_name": "torrust-tracker-vm-torrust-tracker-demo",
  "provider": "hetzner",
  "state": "Released",
  "instance_ip": "46.225.234.201",
  "created_at": "2026-03-03T19:00:42.481676821Z"
}
```

**Duration**: ~134 seconds (image pulls dominate)

## Problems

See [bugs.md](bugs.md) for the issue encountered on the first attempt:
`release` failed with `docker not in PATH` when running via the deployer container.
The fix was applied and the image was rebuilt before the successful run above.
