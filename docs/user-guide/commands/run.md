# `run` - Start Tracker Services

Start the Torrust Tracker application services on a released environment.

## Purpose

Starts the Docker Compose services for the Torrust Tracker, bringing the application online. This command takes an environment from the "Released" state to the "Running" state with active tracker services.

The run command starts services using `docker compose up -d` and verifies they are running and accessible.

## Command Syntax

```bash
torrust-tracker-deployer run <ENVIRONMENT>
```

## Arguments

- `<ENVIRONMENT>` (required) - Name of the environment to start

## Prerequisites

1. **Environment released** - Must run `release` command first
2. **Docker Compose files deployed** - Application configuration must be on VM
3. **Firewall configured** - Tracker ports must be open (done by `configure`)

## State Transition

```text
[Released] --run--> [Running]
```

## What Happens

When you run an environment:

1. **Starts Docker Compose services** - Brings up tracker container (`docker compose up -d`)
2. **Validates services are running** - Checks Docker Compose status
3. **Validates external accessibility** - Verifies tracker services respond from outside VM
   - Tracker API health check (port 1212) - **required**
   - HTTP Tracker health checks (all configured HTTP tracker ports) - **optional**

**Note**: All tracker ports must be explicitly configured (port 0 for dynamic assignment is not supported). See [ADR: Port Zero Not Supported](../../decisions/port-zero-not-supported.md) for details.

### Backup Setup

**Important**: Initial backup creation is not yet automatically triggered during the `run` command. This is a planned enhancement (Phase 4 Part 2.2).

Currently, after the `run` command completes:

1. The backup service is configured and the crontab entry is installed
2. Scheduled backups will run automatically on your configured schedule via crontab
3. You can manually trigger an initial backup using:

   ```bash
   ssh -i <key> user@<instance-ip>
   cd /opt/torrust
   docker compose --profile backup run --rm backup
   ```

For more information on manual backup procedures, see [Backup Management](../backup.md#triggering-manual-backups).

## Services Started

### Tracker Service

The tracker container provides:

- **UDP Tracker** - BitTorrent announce endpoints (default ports: 6868, 6969)
- **HTTP Tracker** - HTTP-based announce endpoint (default port: 7070)
- **HTTP API** - RESTful API for tracker management (default port: 1212)

All services run inside a single `torrust/tracker:develop` Docker container.

## Command Output

When the run command completes successfully, it displays service URLs for easy access:

```text
✓ Run command completed for 'my-environment'

Service URLs:
  API:             http://192.168.1.100:1212
  HTTP Tracker:    http://192.168.1.100:7070
  Health Check:    http://192.168.1.100:1212/api/health_check

Tip: Run 'torrust-tracker-deployer show my-environment' for full details
```

**Notes**:

- Only publicly accessible services are shown (localhost-only services are excluded)
- UDP trackers are not shown (no web-accessible endpoint)
- Prometheus is internal-only and not displayed
- For HTTPS/TLS environments, you'll also see a DNS configuration hint

### HTTPS/TLS Environment Output

For environments with TLS configured, you'll see additional DNS configuration guidance:

```text
✓ Run command completed for 'my-tls-env'

Service URLs:
  API:             https://tracker.example.com:1212
  HTTP Tracker:    https://tracker.example.com:7070
  Health Check:    https://tracker.example.com:1212/api/health_check

⚠️  DNS Configuration Required:
    Configure these domains to point to 192.168.1.100:
    - tracker.example.com

Tip: Run 'torrust-tracker-deployer show my-tls-env' for full details
```

## JSON Output

The `run` command supports JSON output for automation workflows using the `--output-format json` or `-o json` flag.

### Command Syntax

```bash
torrust-tracker-deployer run <ENVIRONMENT> --output-format json
# Or use the short form:
torrust-tracker-deployer run <ENVIRONMENT> -o json
```

### JSON Output Structure

```json
{
  "environment_name": "my-environment",
  "state": "Running",
  "services": {
    "udp_trackers": ["udp://udp.tracker.local:6969/announce"],
    "https_http_trackers": [],
    "direct_http_trackers": ["http://10.140.190.133:7070/announce"],
    "localhost_http_trackers": [],
    "api_endpoint": "http://10.140.190.133:1212/api",
    "api_uses_https": false,
    "api_is_localhost_only": false,
    "health_check_url": "http://10.140.190.133:1313/health_check",
    "health_check_uses_https": false,
    "health_check_is_localhost_only": false,
    "tls_domains": []
  },
  "grafana": {
    "url": "http://10.140.190.133:3000/",
    "uses_https": false
  }
}
```

### JSON Output with HTTPS/TLS

When TLS is configured, the output includes HTTPS URLs and domain information:

```json
{
  "environment_name": "my-tls-env",
  "state": "Running",
  "services": {
    "udp_trackers": ["udp://udp.tracker.example.com:6969/announce"],
    "https_http_trackers": ["https://tracker.example.com:7070/announce"],
    "direct_http_trackers": [],
    "localhost_http_trackers": [],
    "api_endpoint": "https://tracker.example.com:1212/api",
    "api_uses_https": true,
    "api_is_localhost_only": false,
    "health_check_url": "https://tracker.example.com:1313/health_check",
    "health_check_uses_https": true,
    "health_check_is_localhost_only": false,
    "tls_domains": ["tracker.example.com"]
  },
  "grafana": {
    "url": "https://tracker.example.com:3000/",
    "uses_https": true
  }
}
```

### Automation Use Cases

#### Extract API Endpoint

```bash
# Get API endpoint for automated testing
API_ENDPOINT=$(torrust-tracker-deployer run my-env -o json | jq -r '.services.api_endpoint')
curl "$API_ENDPOINT/health_check"
```

#### Check if HTTPS is Required

```bash
# Determine if API uses HTTPS
USES_HTTPS=$(torrust-tracker-deployer run my-env -o json | jq -r '.services.api_uses_https')
if [ "$USES_HTTPS" = "true" ]; then
    echo "HTTPS is enabled"
else
    echo "Using HTTP only"
fi
```

#### Extract All HTTP Tracker URLs

```bash
# Get all HTTP/HTTPS tracker announce URLs for testing
torrust-tracker-deployer run my-env -o json | \
  jq -r '.services | (.direct_http_trackers + .https_http_trackers)[]'
```

#### Parse TLS Domains for DNS Configuration

```bash
# Extract domains that need DNS configuration
DOMAINS=$(torrust-tracker-deployer run my-env -o json | jq -r '.services.tls_domains[]')
for domain in $DOMAINS; do
    echo "Configure DNS A record: $domain → $(jq -r '.services.api_endpoint' <<< "$JSON" | cut -d: -f2 | tr -d '/')"
done
```

#### Monitor Service Status in CI/CD

```bash
# Save output for later analysis
torrust-tracker-deployer run production-env -o json > run-output.json

# Parse for verification
jq '.services.api_endpoint' run-output.json
jq '.services.udp_trackers[]' run-output.json
jq '.grafana.url' run-output.json
```

### JSON Output Fields

| Field                                     | Type     | Description                                       |
| ----------------------------------------- | -------- | ------------------------------------------------- |
| `environment_name`                        | string   | Name of the environment                           |
| `state`                                   | string   | Always "Running" after successful run             |
| `services.udp_trackers`                   | string[] | UDP tracker announce URLs                         |
| `services.https_http_trackers`            | string[] | HTTPS HTTP tracker announce URLs (TLS configured) |
| `services.direct_http_trackers`           | string[] | HTTP tracker announce URLs (no TLS)               |
| `services.localhost_http_trackers`        | string[] | Localhost-only HTTP trackers (for testing)        |
| `services.api_endpoint`                   | string   | Tracker API base URL                              |
| `services.api_uses_https`                 | boolean  | True if API uses HTTPS                            |
| `services.api_is_localhost_only`          | boolean  | True if API only bound to localhost               |
| `services.health_check_url`               | string   | Health check endpoint URL                         |
| `services.health_check_uses_https`        | boolean  | True if health check uses HTTPS                   |
| `services.health_check_is_localhost_only` | boolean  | True if health check only on localhost            |
| `services.tls_domains`                    | string[] | Domains requiring DNS configuration for TLS       |
| `grafana.url`                             | string   | Grafana dashboard URL (if enabled)                |
| `grafana.uses_https`                      | boolean  | True if Grafana uses HTTPS                        |

**Note**: `grafana` will be `null` if monitoring is not enabled in the environment configuration.

## Example Usage

### Basic Run

```bash
# Start tracker services
torrust-tracker-deployer run my-environment
```

### Complete Workflow

```bash
# 1. Create environment
torrust-tracker-deployer create template --provider lxd > my-env.json
# Edit my-env.json with your settings
torrust-tracker-deployer create environment --env-file my-env.json

# 2. Provision infrastructure
torrust-tracker-deployer provision my-environment

# 3. Configure system
torrust-tracker-deployer configure my-environment

# 4. Release application
torrust-tracker-deployer release my-environment

# 5. Start services
torrust-tracker-deployer run my-environment

# Tracker is now running!
```

## Verification

After running, you can verify the tracker is working:

```bash
# Get VM IP address
VM_IP=$(torrust-tracker-deployer show my-environment | grep 'IP Address' | awk '{print $3}')

# Check tracker API health
curl http://$VM_IP:1212/api/health_check

# Expected: {"status":"ok"} or similar health response

# Check tracker stats
curl http://$VM_IP:1212/api/v1/stats

# Expected: JSON with tracker statistics (torrents, seeders, leechers, etc.)

# Check HTTP tracker health
curl http://$VM_IP:7070/api/health_check

# Expected: {"status":"ok"} or similar health response
```

### Check Service Status via SSH

```bash
# SSH into VM
ssh -i ~/.ssh/your-key user@$VM_IP

# Check Docker Compose services
cd /opt/torrust
docker compose ps

# Expected output shows "tracker" service with status "Up"

# View tracker logs
docker compose logs tracker

# Follow tracker logs in real-time
docker compose logs -f tracker
```

### Verify Backup (if enabled)

If you enabled backup in your environment configuration:

```bash
# Check if backup files were created
ssh -i ~/.ssh/your-key user@$VM_IP "ls -lh /opt/torrust/storage/backup/sqlite/"
ssh -i ~/.ssh/your-key user@$VM_IP "ls -lh /opt/torrust/storage/backup/config/"

# Expected: Files like sqlite_20260203_030000.db.gz and config_20260203_030000.tar.gz

# Check crontab for scheduled backups
ssh -i ~/.ssh/your-key user@$VM_IP "crontab -l"

# Expected: Backup cron job with your configured schedule

# View backup logs
ssh -i ~/.ssh/your-key user@$VM_IP "tail -20 /var/log/torrust-backup.log"

# Expected: Messages showing backup cycle completed successfully
```

## Service Ports

The tracker exposes these ports (configurable in environment JSON):

| Port | Protocol | Service      | Purpose                    |
| ---- | -------- | ------------ | -------------------------- |
| 6868 | UDP      | UDP Tracker  | BitTorrent announce (UDP)  |
| 6969 | UDP      | UDP Tracker  | BitTorrent announce (UDP)  |
| 7070 | TCP      | HTTP Tracker | BitTorrent announce (HTTP) |
| 1212 | TCP      | HTTP API     | Tracker management API     |

All ports are accessible externally if firewall is configured correctly.

## Troubleshooting

### Run Fails with "Environment not released"

**Problem**: Trying to run before releasing application files.

**Solution**:

```bash
# Run release first
torrust-tracker-deployer release my-environment
# Then try run again
torrust-tracker-deployer run my-environment
```

### Services Start But Health Check Fails

**Problem**: Docker shows services running but API not responding.

**Solution**:

```bash
# Get VM IP
VM_IP=$(torrust-tracker-deployer show my-environment | grep 'IP Address' | awk '{print $3}')

# Check if service is listening internally
ssh -i ~/.ssh/your-key user@$VM_IP "curl http://localhost:1212/api/health_check"

# If this works, it's a firewall issue - check UFW rules
ssh -i ~/.ssh/your-key user@$VM_IP "sudo ufw status numbered"

# Verify tracker ports are allowed (6868/udp, 6969/udp, 7070/tcp, 1212/tcp)
```

### Tracker Container Crashes on Startup

**Problem**: Container starts but immediately exits.

**Solution**:

```bash
# SSH into VM and check logs
ssh -i ~/.ssh/your-key user@$VM_IP "cd /opt/torrust && docker compose logs tracker"

# Common issues:
# 1. Configuration error in tracker.toml
# 2. Database file permissions
# 3. Port already in use

# Check tracker configuration
ssh -i ~/.ssh/your-key user@$VM_IP "cat /opt/torrust/storage/tracker/etc/tracker.toml"

# Check database file exists and has correct permissions
ssh -i ~/.ssh/your-key user@$VM_IP "ls -la /opt/torrust/storage/tracker/lib/database/"
```

### External Connectivity Issues

**Problem**: Services running internally but not accessible from outside.

**Solution**:

```bash
# Verify firewall rules
ssh -i ~/.ssh/your-key user@$VM_IP "sudo ufw status numbered"

# Check if ports are listening
ssh -i ~/.ssh/your-key user@$VM_IP "sudo netstat -tulnp | grep -E '6868|6969|7070|1212'"

# Test connectivity from host
nc -zv $VM_IP 7070  # HTTP Tracker
nc -zv $VM_IP 1212  # HTTP API

# For UDP (may timeout but verifies firewall)
nc -zvu $VM_IP 6868  # UDP Tracker
```

## Stopping Services

To stop tracker services:

```bash
# SSH into VM
ssh -i ~/.ssh/your-key user@$VM_IP

# Stop services
cd /opt/torrust
docker compose down

# Or stop without removing containers
docker compose stop
```

To restart after stopping:

```bash
# Re-run the run command
torrust-tracker-deployer run my-environment

# Or SSH and start manually
ssh -i ~/.ssh/your-key user@$VM_IP "cd /opt/torrust && docker compose up -d"
```

## Health Check Details

The `run` command performs external health checks to validate deployment:

1. **Docker Compose Status Check** (internal, via SSH)
   - Verifies tracker container is in "running" state
   - Checks via `docker compose ps`

2. **Tracker API Health Check** (external, direct HTTP)
   - Tests `http://<vm-ip>:1212/api/health_check`
   - **Required check** - deployment fails if not accessible
   - Validates both service functionality AND firewall rules

3. **HTTP Tracker Health Checks** (external, direct HTTP)
   - Tests `http://<vm-ip>:<port>/health_check` for **all configured HTTP trackers**
   - **Required checks** - deployment fails if not accessible
   - If you configure multiple HTTP trackers (e.g., ports 7070, 7071, 7072), all will be validated

If external checks fail but Docker shows services running, it indicates a firewall or network configuration issue.

## Using the Tracker

Once running, the tracker can be used by BitTorrent clients:

### UDP Announce URLs

```text
udp://<vm-ip>:6868/announce
udp://<vm-ip>:6969/announce
```

### HTTP Announce URLs

```text
http://<vm-ip>:7070/announce
```

### API Access

```bash
# Get tracker statistics
curl http://$VM_IP:1212/api/v1/stats

# Authenticate with admin token (from environment config)
curl -H "Authorization: Bearer MyAccessToken" \
     http://$VM_IP:1212/api/v1/stats
```

## Next Steps

After starting services:

1. **Test announce** - Configure a BitTorrent client to use your tracker
2. **Monitor logs** - Watch tracker activity via Docker logs
3. **Test API** - Explore tracker management API endpoints

When finished:

- **Stop services** - Use `docker compose down` on VM
- **Destroy environment** - Use `destroy` command to clean up infrastructure

## Related Commands

- [`release`](release.md) - Deploy application configuration (required before run)
- [`configure`](configure.md) - Configure system infrastructure
- [`test`](test.md) - Verify infrastructure readiness
- [`destroy`](destroy.md) - Clean up deployment

## Technical Details

The run command executes these steps in order:

1. **Start services** (`StartServicesStep`) - Runs `docker compose up -d` via Ansible
2. **Validate running services** (`RunningServicesValidator`)
   - Checks Docker Compose status (via SSH)
   - Checks external tracker API accessibility (direct HTTP - **required**)
   - Checks external HTTP tracker accessibility for **all configured HTTP trackers** (direct HTTP - **optional**)

The validation ensures:

- Services are actually running inside the VM
- Firewall rules allow external access
- Tracker API responds to health checks
- All HTTP tracker instances (if configured) are accessible externally

**Port Configuration Note**: Dynamic port assignment (port 0) is not supported. All tracker ports must be explicitly specified in the environment configuration. This ensures deterministic deployment and reliable firewall configuration.
