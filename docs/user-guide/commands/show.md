# `show` - Display Environment Information

Display detailed information about an environment's current state.

## Purpose

Provides a quick, read-only view of environment details including state, infrastructure configuration, and service information. This command reads stored data without making remote connections, making it fast and reliable.

## Command Syntax

```bash
torrust-tracker-deployer show <ENVIRONMENT>
```

## Arguments

- `<ENVIRONMENT>` (required) - Name of the environment to display

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
