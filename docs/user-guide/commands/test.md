# `test` - Verify Deployed Services

Verify that deployed Torrust Tracker services are running and accessible from
external clients.

## Purpose

Performs smoke tests against a deployed environment to confirm that the Tracker
API, HTTP Tracker, and any other configured services respond correctly. Also runs
advisory DNS resolution checks for configured domains.

## Command Syntax

```bash
torrust-tracker-deployer test <ENVIRONMENT> [OPTIONS]
```

## Arguments

- `<ENVIRONMENT>` (required) - Name of the environment to test

## Options

| Option            | Values         | Default           | Description                              |
| ----------------- | -------------- | ----------------- | ---------------------------------------- |
| `--output-format` | `text`, `json` | `text`            | Output format for test results           |
| `--working-dir`   | path           | current dir       | Working directory containing data folder |
| `--log-dir`       | path           | (default log dir) | Directory for log files                  |

## Prerequisites

1. **Environment exists** - Must have run `create environment` first
2. **Instance reachable** - VM must be accessible from the test runner
3. **Services running** - The `run` command must have been executed (or services started manually)

## What Happens

When you test an environment:

1. **Validates environment name** - Confirms the name format is valid
2. **Creates command handler** - Prepares the application layer handler
3. **Tests infrastructure** - Performs external health checks against deployed services:
   - Tracker API health endpoint (required)
   - HTTP Tracker health endpoint (required)
   - Advisory DNS resolution checks for all configured domains

> **Note**: The test command loads the environment in **any state** — it does
> not require a specific state like "Configured" or "Released". As long as the
> environment has an instance IP set and services are accessible, the tests will
> pass.

## Output Formats

### Text Output (default)

```bash
torrust-tracker-deployer test my-environment
```

```text
Test Results:
  Environment:       my-environment
  Instance IP:       10.140.190.39
  Result:            pass
```

With DNS warnings:

```text
Test Results:
  Environment:       my-environment
  Instance IP:       10.140.190.39
  Result:            pass

DNS Warnings:
  - tracker.local: tracker.local does not resolve (expected: 10.140.190.39): name resolution failed
  - api.tracker.local: api.tracker.local resolves to [192.168.1.1] but expected 10.140.190.39
```

### JSON Output

```bash
torrust-tracker-deployer test my-environment --output-format json
```

```json
{
  "environment_name": "my-environment",
  "instance_ip": "10.140.190.39",
  "result": "pass",
  "dns_warnings": []
}
```

With DNS warnings:

```json
{
  "environment_name": "my-environment",
  "instance_ip": "10.140.190.39",
  "result": "pass",
  "dns_warnings": [
    {
      "domain": "tracker.local",
      "expected_ip": "10.140.190.39",
      "issue": "tracker.local does not resolve (expected: 10.140.190.39): name resolution failed"
    },
    {
      "domain": "api.tracker.local",
      "expected_ip": "10.140.190.39",
      "issue": "api.tracker.local resolves to [192.168.1.1] but expected 10.140.190.39"
    }
  ]
}
```

#### JSON Fields

| Field              | Type   | Description                                           |
| ------------------ | ------ | ----------------------------------------------------- |
| `environment_name` | string | Name of the environment tested                        |
| `instance_ip`      | string | IP address of the tested instance                     |
| `result`           | string | Always `"pass"` — failures produce an error, not JSON |
| `dns_warnings`     | array  | Advisory DNS warnings (may be empty)                  |

DNS warning fields:

| Field         | Type   | Description                                 |
| ------------- | ------ | ------------------------------------------- |
| `domain`      | string | The domain that was checked                 |
| `expected_ip` | string | The expected IP address (instance IP)       |
| `issue`       | string | Human-readable description of the DNS issue |

## Validation Details

### External Health Checks

The test command validates deployed services through **external accessibility
checks** — HTTP(S) requests from the test runner to the VM:

- **Tracker API** — Health endpoint (`/api/health_check`)
- **HTTP Tracker** — Health endpoint (`/health_check`)

External checks are preferred because they are a superset of internal checks: if
services are accessible externally, they must be running internally, and firewall
rules are validated automatically.

### HTTPS Support

When services have TLS enabled via Caddy reverse proxy:

- Uses HTTPS URLs with the configured domain
- Resolves domains locally to the VM IP (no DNS dependency for testing)
- Accepts self-signed certificates for `.local` domains

### DNS Resolution Checks

DNS checks are **advisory only** — they do not affect the pass/fail result:

- Checks resolution for all configured service domains (API, HTTP trackers,
  health check API, Grafana)
- Warns when domains don't resolve or resolve to unexpected IPs
- DNS warnings appear in both text and JSON output formats

DNS failures are advisory because:

- DNS propagation can take time
- Local `.local` domains use `/etc/hosts` which may not be configured
- Users may intentionally test without DNS

## Examples

### Basic testing

```bash
# Test the environment
torrust-tracker-deployer test full-stack-docs
```

### JSON output for automation

```bash
# Get machine-readable results
torrust-tracker-deployer test my-env --output-format json | jq '.result'
```

### CI/CD verification

```bash
#!/bin/bash
set -e

ENV_NAME="ci-${BUILD_ID}"

# Setup
torrust-tracker-deployer create environment -f ci.json
torrust-tracker-deployer provision ${ENV_NAME}
torrust-tracker-deployer configure ${ENV_NAME}
torrust-tracker-deployer release ${ENV_NAME}
torrust-tracker-deployer run ${ENV_NAME}

# Verify infrastructure readiness
if torrust-tracker-deployer test ${ENV_NAME}; then
    echo "Infrastructure validated successfully"
else
    echo "Infrastructure tests failed"
    exit 1
fi

# Cleanup
torrust-tracker-deployer destroy ${ENV_NAME}
```

### Complete workflow

```bash
# Full setup and verification
torrust-tracker-deployer create environment -f config.json
torrust-tracker-deployer provision my-environment
torrust-tracker-deployer configure my-environment
torrust-tracker-deployer release my-environment
torrust-tracker-deployer run my-environment
torrust-tracker-deployer test my-environment

# If all tests pass, environment is ready
```

## Troubleshooting

### Services not accessible

**Problem**: Health check endpoints cannot be reached.

**Solution**: Verify services are running on the VM.

```bash
# SSH into the VM
ssh -i <private-key> torrust@<vm-ip>

# Check running services
cd /opt/torrust
docker compose ps

# Check logs
docker compose logs tracker
```

### Health check fails

**Problem**: Services are running but the health endpoint returns an error.

**Solution**: Check service configuration and logs.

```bash
# SSH into the VM
ssh -i <private-key> torrust@<vm-ip>

# Test health endpoint locally
curl http://localhost:1212/api/health_check

# Check tracker logs
cd /opt/torrust
docker compose logs tracker
```

### DNS warnings

**Problem**: DNS warnings appear but all tests pass.

**Solution**: DNS warnings are advisory. To resolve them:

```bash
# For local domains, add entries to /etc/hosts
echo "10.140.190.39 tracker.local" | sudo tee -a /etc/hosts

# For public domains, configure DNS A record to point to the instance IP
# This may take time to propagate
```

### Missing instance IP

**Problem**: Environment does not have an instance IP set.

**Solution**: The environment must have been provisioned (or registered) to have
an IP address.

```bash
# Check environment state
torrust-tracker-deployer show my-environment

# If needed, provision or register the environment
torrust-tracker-deployer provision my-environment
```

## Technical Details

### Test Implementation

The test command uses:

- **External HTTP(S) requests** — validates service accessibility from outside
  the VM
- **DNS resolver** — performs advisory domain resolution checks
- **Environment repository** — loads the environment in any state

### Test Execution Flow

1. **Environment Loading** — Loads environment from repository (any state)
2. **IP Extraction** — Gets instance IP from environment data
3. **Endpoint Building** — Constructs service URLs from tracker configuration
4. **Service Validation** — Performs external health checks against all endpoints
5. **DNS Checks** — Advisory resolution checks for configured domains
6. **Result Reporting** — Renders structured results (text or JSON)

### Port Discovery

The test command extracts tracker ports from the environment's tracker
configuration:

- HTTP API port from `tracker.http_api.bind_address`
- HTTP Tracker port from `tracker.http_trackers[0].bind_address`

## See Also

- [run](run.md) - Run deployed services (prerequisite for testing)
- [configure](configure.md) - Configure infrastructure
- [provision](provision.md) - Provision infrastructure
- [show](show.md) - Show environment details
- [destroy](destroy.md) - Clean up infrastructure
