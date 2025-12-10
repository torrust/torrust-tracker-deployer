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
3. **Validates external accessibility** - Verifies tracker API responds from outside VM
   - Tracker API health check (port 1212)
   - HTTP Tracker health check (port 7070) - optional

## Services Started

### Tracker Service

The tracker container provides:

- **UDP Tracker** - BitTorrent announce endpoints (default ports: 6868, 6969)
- **HTTP Tracker** - HTTP-based announce endpoint (default port: 7070)
- **HTTP API** - RESTful API for tracker management (default port: 1212)

All services run inside a single `torrust/tracker:develop` Docker container.

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
ssh -i ~/.ssh/your-key user@$VM_IP "sudo netstat -tulpn | grep -E '6868|6969|7070|1212'"

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
   - **Required check** - fails if not accessible
   - Validates both service functionality AND firewall rules

3. **HTTP Tracker Health Check** (external, direct HTTP)
   - Tests `http://<vm-ip>:7070/api/health_check`
   - **Optional check** - warns if not accessible
   - Some tracker versions may not have health endpoint

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
   - Checks external tracker API accessibility (direct HTTP)
   - Checks external HTTP tracker accessibility (direct HTTP, optional)

The validation ensures:

- Services are actually running inside the VM
- Firewall rules allow external access
- Tracker API responds to health checks
