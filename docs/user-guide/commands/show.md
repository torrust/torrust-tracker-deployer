# `show` - Display Environment Information

Display detailed information about an environment's current state.

## Purpose

Provides a quick, read-only view of environment details including state, infrastructure configuration, and service information. This command reads stored data without making remote connections, making it fast and reliable.

## Command Syntax

```bash
torrust-tracker-deployer show <ENVIRONMENT> [OPTIONS]
```

## Arguments

- `<ENVIRONMENT>` (required) - Name of the environment to display

## Options

- `-o, --output-format <FORMAT>` (optional) - Output format: `text` (default) or `json`

## Prerequisites

1. **Environment exists** - Must create environment first with `create environment`

## What Information Is Displayed

The information displayed depends on the environment's current state:

### Created State

When an environment has been created but not provisioned:

```text
Environment: my-environment
State: Created
Provider: LXD
Created: 2025-01-07 14:30:00 UTC

Next: Run 'provision my-environment' to create infrastructure
```

### Provisioned/Configured State

When infrastructure has been created:

```text
Environment: my-environment
State: Provisioned
Provider: LXD
Created: 2025-01-07 14:30:00 UTC

Infrastructure:
  Instance IP: 10.140.190.171
  SSH Port: 22
  SSH User: torrust
  SSH Key: ~/.ssh/torrust_deployer_key

Connection:
  ssh -i ~/.ssh/torrust_deployer_key torrust@10.140.190.171

Next: Run 'configure my-environment' to install software
```

### Released/Running State

When services have been deployed:

```text
Environment: my-environment
State: Running
Provider: LXD
Created: 2025-01-07 14:30:00 UTC

Infrastructure:
  Instance IP: 10.140.190.171
  SSH Port: 22
  SSH User: torrust
  SSH Key: ~/.ssh/torrust_deployer_key

Connection:
  ssh -i ~/.ssh/torrust_deployer_key torrust@10.140.190.171

Tracker Services:
  UDP Trackers:
    - udp://10.140.190.171:6969/announce
  HTTP Trackers:
    - http://10.140.190.171:7070/announce
  API Endpoint:
    - http://10.140.190.171:1212/api
  Health Check:
    - http://10.140.190.171:1313/health_check

Tracker is running! Use the URLs above to connect.
```

## Output Formats

The `show` command supports two output formats:

### Text Format (Default)

Human-readable format suitable for terminal viewing:

```bash
torrust-tracker-deployer show my-environment
# or explicitly:
torrust-tracker-deployer show my-environment --output-format text
```

### JSON Format

Machine-readable format for automation and scripting:

```bash
torrust-tracker-deployer show my-environment --output-format json
```

#### JSON Output for Provisioned State

```json
{
  "name": "my-environment",
  "state": "Provisioned",
  "provider": "LXD",
  "created_at": "2026-02-16T17:56:43.788700279Z",
  "infrastructure": {
    "instance_ip": "10.140.190.85",
    "ssh_port": 22,
    "ssh_user": "torrust",
    "ssh_key_path": "/home/user/.ssh/torrust_key"
  },
  "services": null,
  "prometheus": null,
  "grafana": null,
  "state_name": "provisioned"
}
```

#### JSON Output for Running State

```json
{
  "name": "my-environment",
  "state": "Running",
  "provider": "LXD",
  "created_at": "2026-02-11T09:52:28.800407753Z",
  "infrastructure": {
    "instance_ip": "10.140.190.36",
    "ssh_port": 22,
    "ssh_user": "torrust",
    "ssh_key_path": "/home/user/.ssh/torrust_key"
  },
  "services": {
    "udp_trackers": ["udp://udp.tracker.local:6969/announce"],
    "https_http_trackers": ["https://http.tracker.local/announce"],
    "direct_http_trackers": [],
    "localhost_http_trackers": [],
    "api_endpoint": "https://api.tracker.local/api",
    "api_uses_https": true,
    "api_is_localhost_only": false,
    "health_check_url": "https://health.tracker.local/health_check",
    "health_check_uses_https": true,
    "health_check_is_localhost_only": false,
    "tls_domains": [
      {
        "domain": "http.tracker.local",
        "internal_port": 7070
      },
      {
        "domain": "api.tracker.local",
        "internal_port": 1212
      }
    ]
  },
  "prometheus": {
    "access_note": "Internal only (localhost:9090) - not exposed externally"
  },
  "grafana": {
    "url": "https://grafana.tracker.local/",
    "uses_https": true
  },
  "state_name": "running"
}
```

## Examples

### Basic usage

```bash
# Show environment information
torrust-tracker-deployer show my-environment
```

### Check environment state in scripts

```bash
#!/bin/bash
# Check if environment exists before operations
if torrust-tracker-deployer show my-environment 2>/dev/null; then
    echo "Environment found"
else
    echo "Environment not found - creating..."
    torrust-tracker-deployer create environment -f config.json
fi
```

### Parse JSON output for automation

```bash
#!/bin/bash
# Extract tracker URL from environment
API_URL=$(torrust-tracker-deployer show my-env -o json | \
    jq -r '.services.api_endpoint // empty')

if [ -n "$API_URL" ]; then
    echo "API available at: $API_URL"
    curl "$API_URL/stats"
else
    echo "Service not yet running"
fi
```

### Monitor environment state

```bash
#!/bin/bash
# Check if environment is fully running
STATE=$(torrust-tracker-deployer show my-env -o json | \
    jq -r '.state_name')

if [ "$STATE" = "running" ]; then
    echo "✓ Environment is fully operational"
else
    echo "⚠ Environment is in '$STATE' state"
fi
```

### Quick reference for SSH connection

```bash
# Show environment to get SSH command
torrust-tracker-deployer show my-environment

# The output includes a ready-to-use SSH command:
# Connection:
#   ssh -i ~/.ssh/key user@10.140.190.171
```

## Command Comparison

| Command | Purpose               | Network Access |
| ------- | --------------------- | -------------- |
| `show`  | Display stored state  | No (fast)      |
| `test`  | Verify infrastructure | Yes (SSH)      |
| `list`  | List all environments | No (fast)      |

## Error Handling

### Environment Not Found

If the specified environment doesn't exist:

```bash
torrust-tracker-deployer show nonexistent
# Error: Environment 'nonexistent' not found
#
# Use 'list' to see available environments
```

## Performance

The `show` command is designed to be fast:

- **No network connections** - Reads only from local storage
- **Typical execution** - Under 100ms
- **No remote verification** - Use `test` command for that

## Related Commands

- [`create environment`](create.md) - Create a new environment
- [`list`](README.md) - List all environments
- [`test`](test.md) - Verify infrastructure is working
- [`provision`](provision.md) - Create infrastructure
- [`configure`](configure.md) - Install software
- [`release`](release.md) - Deploy tracker services
- [`run`](run.md) - Start tracker services
