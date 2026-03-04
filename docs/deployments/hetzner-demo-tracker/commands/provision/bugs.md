# Bugs: provision

Issues found in the `provision` command output during the Hetzner demo tracker deployment.

> This is a living document — bugs are added as they are discovered.

<!--
Template for each bug:

## Bug: [Short description]

**Command**: `provision`
**Severity**: [Critical / Major / Minor]

### Symptom

What was observed (wrong output, missing data, unexpected behavior).

### Expected Behavior

What should have happened.

### Root Cause

Why it happens (if known).

### Workaround

How to get the missing information until the bug is fixed.

### Fix

Where the fix should be applied (if known).
-->

## Bug: UDP tracker domains missing from `provision` output

**Command**: `provision`
**Severity**: Minor

### Symptom

The `domains` array in the successful `provision` output only contains HTTP-based domains.
UDP tracker domains are not included:

```json
{
  "domains": [
    "http1.torrust-tracker-demo.com",
    "http2.torrust-tracker-demo.com",
    "api.torrust-tracker-demo.com",
    "grafana.torrust-tracker-demo.com"
  ]
}
```

The two UDP tracker domains configured in `envs/torrust-tracker-demo.json` are absent:

```text
udp1.torrust-tracker-demo.com   (port 6969)
udp2.torrust-tracker-demo.com   (port 6868)
```

### Expected Behavior

All service domains — HTTP and UDP — should appear in the `domains` array:

```json
{
  "domains": [
    "http1.torrust-tracker-demo.com",
    "http2.torrust-tracker-demo.com",
    "udp1.torrust-tracker-demo.com",
    "udp2.torrust-tracker-demo.com",
    "api.torrust-tracker-demo.com",
    "grafana.torrust-tracker-demo.com"
  ]
}
```

These domains are defined in `envs/torrust-tracker-demo.json` under `tracker.udp_trackers`:

```json
"udp_trackers": [
  { "bind_address": "[::]:6969", "domain": "udp1.torrust-tracker-demo.com" },
  { "bind_address": "[::]:6868", "domain": "udp2.torrust-tracker-demo.com" }
]
```

And in the deployment spec at
[`docs/deployments/hetzner-demo-tracker/deployment-spec.md`](../../deployment-spec.md):

```text
UDP Tracker 1: udp://udp1.torrust-tracker-demo.com:6969/announce
UDP Tracker 2: udp://udp2.torrust-tracker-demo.com:6868/announce
```

### Root Cause

The `provision` command output is built from the environment configuration. It likely only
collects domains from HTTP-based tracker configs (`http_trackers`, `http_api`, `grafana`) and
does not iterate over `udp_trackers` when assembling the `domains` list.

### Workaround

The UDP tracker domains are defined in the environment config file and can be read directly:

```bash
cat envs/torrust-tracker-demo.json \
  | python3 -c "import json,sys; d=json.load(sys.stdin); \
    [print(t['domain']) for t in d['tracker']['udp_trackers']]"
```

Expected output:

```text
udp1.torrust-tracker-demo.com
udp2.torrust-tracker-demo.com
```

### Fix

The domain collection logic in the `provision` output builder should include domains from all
service types. Specifically, `udp_trackers[].domain` entries should be added to the `domains`
list alongside the HTTP tracker and API domains.

**Likely location**: the `ProvisionOutput` struct and the code that populates its `domains`
field, in `src/application/commands/provision/`.
