# Command: configure

> **Status**: ✅ Configured successfully (2026-03-04, took ~103 s).

## What `configure` does

The `configure` command:

1. Renders Ansible playbook templates into `build/<env>/ansible/`.
2. Runs the Ansible playbook over SSH to set up the server:
   - Installs Docker and Docker Compose.
   - Creates the application user and directory structure under `/opt/torrust/`.
   - Writes the tracker configuration files (`.env`, `docker-compose.yml`,
     `tracker.toml`, Prometheus config, Grafana provisioning) to
     `/opt/torrust/` on the server.
3. Marks the environment as `Configured` on success.

It does **not** pull Docker images or start any services — that is done by `release` and `run`.

## Command

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  configure torrust-tracker-demo 2>&1 | tee -a data/logs/log.txt
```

## Output

```text
[2026-03-04 11:52:13] Starting Torrust Tracker Deployer container...
[2026-03-04 11:52:13] Verifying installed tools...
[2026-03-04 11:52:13] Tool versions:
[2026-03-04 11:52:13]   - OpenTofu: OpenTofu v1.11.5
[2026-03-04 11:52:13]   - Ansible: ansible [core 2.20.3]
[2026-03-04 11:52:13]   - SSH: OpenSSH_10.0p2 Debian-7, OpenSSL 3.5.4 30 Sep 2025
[2026-03-04 11:52:13]   - Git: git version 2.47.3
[2026-03-04 11:52:13] SSH directory found, checking permissions...
[2026-03-04 11:52:13] Container initialization complete. Executing command...
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: torrust-tracker-demo (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Configuring infrastructure...
⏳   ✓ Infrastructure configured (took 103.5s)
✅ Environment 'torrust-tracker-demo' configured successfully

{
  "environment_name": "torrust-tracker-demo",
  "instance_name": "torrust-tracker-vm-torrust-tracker-demo",
  "provider": "hetzner",
  "state": "Configured",
  "instance_ip": "46.225.234.201",
  "created_at": "2026-03-03T19:00:42.481676821Z"
}
```

## Problems

None.
