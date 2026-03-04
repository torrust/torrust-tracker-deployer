# Command: run

> **Status**: ✅ Completed (2026-03-04) — state: `Running`
>
> **Note**: The `run` command succeeded and the environment transitioned to
> `Running`, but the tracker container was in a restart loop due to
> [Bug 3](bugs.md#bug-3-mysql-password-is-not-url-encoded-in-the-tracker-connection-string)
> (URL encoding). A manual fix was applied before the `test` command was run.
> See [bugs.md](bugs.md) for details.

## What `run` does

The `run` command:

1. Validates that the environment is in `Released` state.
2. Runs the `run-compose-services` Ansible playbook on the remote server.
3. The playbook pulls any updated Docker images, then runs `docker compose up -d`.
4. Transitions the environment state to `Running` on success.

It requires the environment to already be in a `Released` state (i.e., `release`
must have been run first).

**Important**: `Running` state only guarantees that `docker compose up -d`
returned successfully — not that all services are healthy and reachable. Use the
`test` command immediately after `run` to verify the stack.
See [improvements.md](improvements.md) for more context.

## Command

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  run torrust-tracker-demo 2>&1 | tee -a data/logs/log.txt
```

## Output

```text
[2026-03-04 13:19:16] Starting Torrust Tracker Deployer container...
[2026-03-04 13:19:16] Verifying installed tools...
[2026-03-04 13:19:16] Tool versions:
[2026-03-04 13:19:16]   - OpenTofu: OpenTofu v1.11.5
[2026-03-04 13:19:16]   - Ansible: ansible [core 2.20.3]
[2026-03-04 13:19:16]   - SSH: OpenSSH_10.0p2 Debian-7, OpenSSL 3.5.4 30 Sep 2025
[2026-03-04 13:19:16]   - Git: git version 2.47.3
[2026-03-04 13:19:16] SSH directory found, checking permissions...
[2026-03-04 13:19:16] Container initialization complete. Executing command...
⏳ [1/2] Validating environment...
⏳   ✓ Environment name validated: torrust-tracker-demo (took 0ms)
⏳ [2/2] Running application services...
⏳   ✓ Services started (took 31.6s)
✅ Run command completed for 'torrust-tracker-demo'
{
  "environment_name": "torrust-tracker-demo",
  "state": "Running",
  "services": {
    "udp_trackers": [
      "udp://udp1.torrust-tracker-demo.com:6969/announce",
      "udp://udp2.torrust-tracker-demo.com:6868/announce"
    ],
    "https_http_trackers": [
      "https://http1.torrust-tracker-demo.com/announce",
      "https://http2.torrust-tracker-demo.com/announce"
    ],
    "direct_http_trackers": [],
    "localhost_http_trackers": [],
    "api_endpoint": "https://api.torrust-tracker-demo.com/api",
    "api_uses_https": true,
    "api_is_localhost_only": false,
    "health_check_url": "http://46.225.234.201:1313/health_check",
    "health_check_uses_https": false,
    "health_check_is_localhost_only": true,
    "tls_domains": [
      {
        "domain": "http1.torrust-tracker-demo.com",
        "internal_port": 7070
      },
      {
        "domain": "http2.torrust-tracker-demo.com",
        "internal_port": 7071
      },
      {
        "domain": "api.torrust-tracker-demo.com",
        "internal_port": 1212
      },
      {
        "domain": "grafana.torrust-tracker-demo.com",
        "internal_port": 3000
      }
    ]
  },
  "grafana": {
    "url": "https://grafana.torrust-tracker-demo.com/",
    "uses_https": true
  }
}
```

### Duration

- Total: ~31.6s (Ansible `docker compose up -d` + container startup)

### State Transition

`Released` → `Running`
