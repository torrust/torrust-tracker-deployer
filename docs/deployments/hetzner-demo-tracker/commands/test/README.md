# Command: test

> **Status**: ✅ Completed (2026-03-04) — result: `pass` (with DNS warnings)

## What `test` does

The `test` command runs smoke tests against a deployed environment to verify
that the infrastructure and services are reachable and responding correctly.
It is the recommended step to run immediately after `run`.

The test command:

1. Validates that the environment is in `Running` state.
2. Performs DNS resolution checks for each configured domain, comparing the
   resolved IP against the server's instance IP.
3. Reports a `pass` result if the infrastructure tests succeed, even if there
   are DNS warnings.

**Note**: These are infrastructure-level smoke tests. They do not test tracker
protocol logic (announce, scrape, etc.) — those are covered by deeper integration
tests.

## Command

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  test torrust-tracker-demo 2>&1 | tee -a data/logs/log.txt
```

## Output

```text
[2026-03-04 14:06:48] Starting Torrust Tracker Deployer container...
[2026-03-04 14:06:48] Verifying installed tools...
[2026-03-04 14:06:48] Tool versions:
[2026-03-04 14:06:48]   - OpenTofu: OpenTofu v1.11.5
[2026-03-04 14:06:48]   - Ansible: ansible [core 2.20.3]
[2026-03-04 14:06:48]   - SSH: OpenSSH_10.0p2 Debian-7, OpenSSL 3.5.4 30 Sep 2025
[2026-03-04 14:06:48]   - Git: git version 2.47.3
[2026-03-04 14:06:48] SSH directory found, checking permissions...
[2026-03-04 14:06:48] Container initialization complete. Executing command...
⏳ [1/3] Validating environment...
⏳   ✓ Environment name validated: torrust-tracker-demo (took 0ms)
⏳ [2/3] Creating command handler...
⏳   ✓ Done (took 0ms)
⏳ [3/3] Testing infrastructure...
⚠️  DNS check: api.torrust-tracker-demo.com resolves to [116.202.176.169] but expected 46.225.234.201
⚠️  DNS check: http1.torrust-tracker-demo.com resolves to [116.202.176.169] but expected 46.225.234.201
⚠️  DNS check: http2.torrust-tracker-demo.com resolves to [116.202.176.169] but expected 46.225.234.201
⚠️  DNS check: grafana.torrust-tracker-demo.com resolves to [116.202.176.169] but expected 46.225.234.201
⏳   ✓ Infrastructure tests passed (with DNS warnings) (took 1.0s)
{
  "environment_name": "torrust-tracker-demo",
  "instance_ip": "46.225.234.201",
  "result": "pass",
  "dns_warnings": [
    {
      "domain": "api.torrust-tracker-demo.com",
      "expected_ip": "46.225.234.201",
      "issue": "api.torrust-tracker-demo.com resolves to [116.202.176.169] but expected 46.225.234.201"
    },
    {
      "domain": "http1.torrust-tracker-demo.com",
      "expected_ip": "46.225.234.201",
      "issue": "http1.torrust-tracker-demo.com resolves to [116.202.176.169] but expected 46.225.234.201"
    },
    {
      "domain": "http2.torrust-tracker-demo.com",
      "expected_ip": "46.225.234.201",
      "issue": "http2.torrust-tracker-demo.com resolves to [116.202.176.169] but expected 46.225.234.201"
    },
    {
      "domain": "grafana.torrust-tracker-demo.com",
      "expected_ip": "46.225.234.201",
      "issue": "grafana.torrust-tracker-demo.com resolves to [116.202.176.169] but expected 46.225.234.201"
    }
  ]
}
```

### Duration

- Total: ~1.0s

### State Transition

No state transition — `test` is a read-only verification step. The environment
remains in `Running` state.

## DNS Warnings Explained

The test checks that each domain resolves to the server's **instance IP**
(`46.225.234.201`). All four domains instead resolve to `116.202.176.169`, which
is the **floating IP** assigned to this deployment.

This is expected and correct behavior for this setup — the DNS records
deliberately point to the floating IP, not the instance IP, so that traffic can
be rerouted to a different server instance by reassigning the floating IP without
changing DNS. See the provisioning phase documentation for the floating IP setup.

The test reports these as warnings (not failures) because the infrastructure
resolves DNS correctly; the resolved IP just differs from the raw instance IP.
A future improvement of the `test` command could be made aware of the floating IP
and treat a match against it as a pass rather than a warning.
See [improvements.md](../run/improvements.md) for related improvement notes.
