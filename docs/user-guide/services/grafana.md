# Grafana Visualization Service

This guide covers the Grafana visualization service integration in the Torrust Tracker Deployer.

## Overview

The deployer includes Grafana for metrics visualization by default. Grafana provides a web-based interface for creating dashboards and visualizing metrics collected by Prometheus, making it easy to monitor tracker performance and behavior.

## Default Behavior

- **Enabled by default** in generated environment templates
- **Requires Prometheus** to be enabled (hard dependency)
- Web UI accessible on port `3100`
- Default admin credentials: `admin` / `admin` (should be changed)
- Uses Docker named volume for persistent storage

## Configuration

### Basic Configuration

Add the `grafana` section to your environment configuration file:

```json
{
  // ... environment, provider, ssh_credentials, tracker config ...
  "prometheus": {
    "scrape_interval_in_secs": 15
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "SecurePassword123!"
  }
}
```

**Note**: Only relevant sections shown. See [`envs/manual-test-grafana.json`](../../../envs/manual-test-grafana.json) for a complete example.

### Configuration Fields

**grafana.admin_user** (required):

- Administrator username for Grafana UI login
- Default: `admin`
- Used for first login and administrative tasks

**grafana.admin_password** (required):

- Administrator password for Grafana UI login
- Default: `admin`
- **⚠️ SECURITY WARNING**: Always change the default password before deploying to production environments

**grafana.domain** (optional):

- Domain name for HTTPS access via Caddy reverse proxy
- When present with `use_tls_proxy: true`, Grafana is accessible via HTTPS at this domain
- Caddy automatically obtains and renews TLS certificates

**grafana.use_tls_proxy** (optional):

- Boolean to enable HTTPS via Caddy reverse proxy
- Default: `false` (HTTP only)
- Requires `domain` to be set
- When enabled, port 3100 is not exposed; access is via HTTPS (port 443)

**Examples**:

```json
// Development environment (HTTP only)
{
  "grafana": {
    "admin_user": "admin",
    "admin_password": "devpass123"
  }
}

// Production environment with HTTPS
{
  "grafana": {
    "admin_user": "grafana-admin",
    "admin_password": "Str0ng!P@ssw0rd#2024",
    "domain": "grafana.example.com",
    "use_tls_proxy": true
  }
}
```

> **Note**: When using HTTPS, you must also configure the global `https` section with `admin_email`. See the [HTTPS Guide](https.md) for complete documentation.

**Real Example**: See [`envs/manual-test-grafana.json`](../../../envs/manual-test-grafana.json) for a working configuration.

### Prometheus Dependency

**IMPORTANT**: Grafana requires Prometheus to be enabled. If you include the `grafana` section but remove the `prometheus` section, environment creation will fail with a clear validation error.

**Valid Configuration** (both services):

```json
{
  "prometheus": { "scrape_interval_in_secs": 15 },
  "grafana": { "admin_user": "admin", "admin_password": "SecurePassword123!" }
}
```

**Invalid Configuration** (Grafana without Prometheus):

```json
{
  // Missing prometheus section - will fail validation
  "grafana": { "admin_user": "admin", "admin_password": "SecurePassword123!" }
}
```

## Disabling Grafana

To deploy without Grafana visualization, remove the entire `grafana` section from your environment config:

```json
{
  // ... environment, provider, ssh_credentials, tracker config ...
  "prometheus": {
    "scrape_interval_in_secs": 15
  }
  // No grafana section = visualization disabled
}
```

**Note**: Prometheus can run independently without Grafana. This is useful if you:

- Want programmatic access to metrics only
- Use custom visualization tools
- Need minimal resource usage

## Accessing Grafana

After deployment, the Grafana web UI is available at:

**HTTP (default)**:

```text
http://<vm-ip>:3100
```

**HTTPS (when TLS enabled)**:

```text
https://<your-domain>
```

Where `<vm-ip>` is the IP address of your deployed VM instance and `<your-domain>` is the configured domain (e.g., `grafana.example.com`).

### Finding Your VM IP

Use the `show` command to display environment information including the VM IP:

```bash
torrust-tracker-deployer show <env-name>
```

Look for the "Instance IP" field in the output.

> **Note**: JSON output format is planned for future releases to enable scripting.

### First Login

1. Open `http://<vm-ip>:3100` in your web browser
2. Enter your credentials:
   - **Username**: Value from `grafana.admin_user` in your config
   - **Password**: Value from `grafana.admin_password` in your config
3. You'll be taken to the Grafana home page

## Initial Setup

### Adding Prometheus Datasource

**Current Status**: Manual setup required (automation planned for future release)

After first login, you need to add Prometheus as a datasource:

1. **Navigate to Configuration**:
   - Click the gear icon (⚙️) in the left sidebar
   - Select **Data Sources**

2. **Add New Datasource**:
   - Click **Add data source**
   - Select **Prometheus** from the list

3. **Configure Datasource**:
   - **Name**: `Prometheus` (or any name you prefer)
   - **URL**: `http://prometheus:9090`
   - **Access**: `Server (default)`
   - Leave other settings as default

4. **Verify Connection**:
   - Click **Save & Test** at the bottom
   - You should see a green "Data source is working" message

**Troubleshooting**: If connection fails, verify:

- Prometheus service is running: `docker ps | grep prometheus`
- Prometheus is on the same Docker network as Grafana
- URL uses internal Docker service name: `http://prometheus:9090` (not `localhost`)

### Importing Tracker Dashboards

The Torrust project provides two sample dashboards for visualizing tracker metrics:

#### Available Dashboards

1. **stats.json** - Statistics Dashboard
   - Displays data from the `/api/v1/stats` tracker endpoint
   - Shows high-level tracker statistics
   - Good for general monitoring

2. **metrics.json** - Metrics Dashboard
   - Displays data from the `/api/v1/metrics` tracker endpoint (Prometheus format)
   - Shows detailed performance metrics
   - Good for in-depth analysis

**Source**: [torrust-demo/share/grafana/dashboards/](https://github.com/torrust/torrust-demo/tree/main/share/grafana/dashboards)

#### Import Process

1. **Navigate to Dashboards**:
   - Click the **+** icon in the left sidebar
   - Select **Import**

2. **Upload Dashboard**:
   - Click **Upload JSON file** and select the dashboard file
   - Or paste the JSON content directly into the text area

3. **Configure Import**:
   - **Name**: Keep the default or customize
   - **Folder**: Select a folder or leave as default
   - **Prometheus**: Select the datasource you created earlier

4. **Complete Import**:
   - Click **Import**
   - The dashboard will open automatically

#### Customizing Dashboards

After importing, you can:

- Modify panels to show different metrics
- Add new panels with custom queries
- Change time ranges and refresh intervals
- Export modified dashboards for reuse
- Share dashboards with team members

## Using Grafana

### Dashboard Features

**Time Range Selection**:

- Use the time picker in the top-right to select ranges
- Common options: Last 5 minutes, Last 1 hour, Last 24 hours
- Custom ranges supported

**Auto-Refresh**:

- Enable auto-refresh for real-time monitoring
- Options: 5s, 10s, 30s, 1m, 5m, 15m, 30m, 1h
- Disable when not actively monitoring to reduce load

**Panel Interactions**:

- Click on legends to show/hide specific series
- Hover over graphs to see detailed values
- Click and drag to zoom into time ranges
- Double-click to reset zoom

### Creating Custom Dashboards

1. **New Dashboard**:
   - Click **+** icon → **Dashboard**
   - Click **Add visualization**

2. **Select Data Source**:
   - Choose your Prometheus datasource

3. **Write Query**:
   - Use Prometheus query language (PromQL)
   - Examples:

     ```promql
     # Total announced peers
     torrust_tracker_announced_peers_total

     # Rate of announcements per second
     rate(torrust_tracker_announced_peers_total[5m])

     # Active torrents
     torrust_tracker_active_torrents
     ```

4. **Customize Visualization**:
   - Choose panel type (Graph, Stat, Gauge, Table, etc.)
   - Set thresholds and colors
   - Add units and labels

5. **Save Dashboard**:
   - Click the save icon (💾) in the top-right
   - Give it a name and optional description

## Verification

### Manual Verification

For detailed step-by-step verification instructions, see the [Grafana Verification Guide](../../e2e-testing/manual/grafana-verification.md).

**Quick Check**:

```bash
# 1. Verify Grafana container is running
ssh torrust@<vm-ip> "docker ps | grep grafana"

# 2. Check Grafana is accessible
curl -u admin:SecurePassword123! http://<vm-ip>:3100/api/health

# Expected output: {"commit":"...","database":"ok","version":"11.4.0"}

# 3. Verify login credentials
curl -u admin:SecurePassword123! http://<vm-ip>:3100/api/org
# Should return HTTP 200 with organization info

# 4. Test with wrong credentials (should fail)
curl -u admin:wrongpassword http://<vm-ip>:3100/api/org
# Should return HTTP 401 Unauthorized
```

### Automated Verification

The E2E tests include automated Grafana validation:

```rust
// From tests/e2e/validators/grafana.rs
GrafanaValidator::validate(
    &ssh_credentials,
    &expected_credentials,
)?;
```

## Troubleshooting

### Login Fails with "Invalid username or password"

**Symptom**: Cannot log in with configured credentials.

**Possible Causes**:

1. **Password mismatch**: Check `data/<env-name>/environment.json` to verify the stored password
2. **Container restarted**: Environment variables not persisted across restarts
3. **Typo in configuration**: Verify exact password in config file

**Solution**:

```bash
# 1. Check stored password in environment state
cat data/<env-name>/environment.json | jq '.Created.user_inputs.grafana.admin_password'

# 2. Verify environment variable in container
ssh torrust@<vm-ip> "docker exec grafana printenv | grep GF_SECURITY"

# 3. Check .env file
ssh torrust@<vm-ip> "cat docker-compose/.env | grep GRAFANA"

# 4. If mismatch found, re-run release and run commands
torrust-tracker-deployer release <env-name>
torrust-tracker-deployer run <env-name>
```

### Grafana UI Not Accessible

**Symptom**: Browser cannot connect to `http://<vm-ip>:3100`.

**Diagnosis**:

```bash
# 1. Verify Grafana container is running
ssh torrust@<vm-ip> "docker ps | grep grafana"

# 2. Check port binding
ssh torrust@<vm-ip> "docker ps | grep grafana" | grep "3100"
# Should show: 0.0.0.0:3100->3000/tcp

# 3. Test from VM itself
ssh torrust@<vm-ip> "curl -s http://localhost:3100/api/health"

# 4. Check container logs
ssh torrust@<vm-ip> "docker logs grafana"
```

**Common Solutions**:

- Container not running: `docker start grafana`
- Port conflict: Check if port 3100 is already in use
- Network issues: Verify Docker network `backend_network` exists

### Prometheus Datasource Connection Fails

**Symptom**: "Data source is not working" error when testing Prometheus connection.

**Diagnosis**:

```bash
# 1. Verify Prometheus is running
ssh torrust@<vm-ip> "docker ps | grep prometheus"

# 2. Check Prometheus accessibility from Grafana container
ssh torrust@<vm-ip> "docker exec grafana curl -s http://prometheus:9090/api/v1/status/config"

# 3. Verify both are on same network
ssh torrust@<vm-ip> "docker network inspect backend_network"
```

**Common Solutions**:

- Wrong URL: Must use `http://prometheus:9090` (Docker service name, not `localhost`)
- Network issue: Ensure both containers are on `backend_network`
- Prometheus not running: Start Prometheus container first

### Dashboards Show No Data

**Symptom**: Panels show "No data" or empty graphs.

**Diagnosis**:

1. **Check Time Range**: Ensure time range covers when tracker was running
2. **Verify Datasource**: Confirm Prometheus datasource is selected in dashboard
3. **Test Query**: Use Prometheus UI (`http://<vm-ip>:9090`) to verify data exists
4. **Check Tracker**: Ensure tracker is running and generating metrics

**Solution**:

```bash
# 1. Verify tracker is running and generating metrics
curl http://<vm-ip>:8080/api/v1/metrics

# 2. Check Prometheus is scraping metrics
# Go to http://<vm-ip>:9090/targets
# Verify tracker targets are "UP"

# 3. In Grafana, try a simple query first
# Query: up{job="tracker"}
# Should show 1 if tracker is up
```

## Architecture

### Deployment Structure

```text
VM Instance
├── Docker Containers
│   ├── grafana (port 3100 → 3000)
│   ├── prometheus (port 9090)
│   └── tracker (port 8080)
├── Docker Networks
│   └── backend_network (connects all services)
└── Docker Volumes
    └── grafana_data (persistent dashboards/datasources)
```

### Storage

**Named Volume**: `grafana_data`

- **Location**: `/var/lib/grafana` inside container
- **Contents**: Dashboards, datasources, user preferences, database
- **Persistence**: Survives container restarts and updates
- **Backup**: Requires Docker volume commands

**Backup Example**:

```bash
# Export dashboard JSON from Grafana UI, or:
docker run --rm -v grafana_data:/data -v $(pwd):/backup \
  ubuntu tar czf /backup/grafana-backup.tar.gz /data
```

### Port Exposure

**Port Mapping**: `3100:3000` (Host:Container)

- **Host Port**: `3100` - Accessible from outside VM
- **Container Port**: `3000` - Grafana's default internal port
- **Reason for 3100**: Avoid conflicts with other services commonly using port 3000

**Security Note**: Docker published ports **bypass UFW firewall rules**. The port is accessible from any network that can reach the host. This is acceptable for development/testing but requires reverse proxy with TLS for production (see roadmap).

### Environment Variables

Configuration via environment variables (injected from `.env` file):

- `GF_SECURITY_ADMIN_USER` - Admin username
- `GF_SECURITY_ADMIN_PASSWORD` - Admin password

### Service Dependencies

**Docker Compose**:

```yaml
services:
  grafana:
    depends_on:
      - prometheus
```

**Startup Order**: Prometheus starts first, then Grafana. Grafana UI remains functional even if Prometheus is temporarily unavailable.

## Future Enhancements

### Planned Automation

A separate issue is planned to add:

1. **Auto-Provision Prometheus Datasource**:
   - Automatically create datasource during deployment
   - Zero-config experience for users
   - No manual setup steps required

2. **Auto-Import Tracker Dashboards**:
   - Automatically import `stats.json` and `metrics.json`
   - Dashboards available immediately after deployment
   - Provisioning via `provisioning/dashboards/` directory

3. **Customizable Dashboard Templates**:
   - Support for user-provided dashboard JSON files
   - Template-based dashboard generation
   - Environment-specific dashboard configuration

### Roadmap Items

- **Reverse Proxy**: TLS termination for secure external access (Task 6)
- **Automated Backups**: Scheduled dashboard and configuration backups (Task 7)
- **Multi-Environment Dashboards**: Aggregate metrics from multiple deployments (Task 8)

## Related Documentation

- **[HTTPS Guide](https.md)** - Enable HTTPS with automatic TLS certificates
- **[Prometheus Service Guide](prometheus.md)** - Metrics collection service
- **[Manual Verification Guide](../../e2e-testing/manual/grafana-verification.md)** - Detailed verification steps
- **[Grafana Integration ADR](../../decisions/grafana-integration-pattern.md)** - Design decisions and rationale
- **[Sample Dashboards](https://github.com/torrust/torrust-demo/tree/main/share/grafana/dashboards)** - Torrust tracker dashboard examples
- **[Grafana Documentation](https://grafana.com/docs/grafana/latest/)** - Official Grafana documentation

## Support

For issues specific to Grafana integration in the deployer:

- Check the [troubleshooting section](#troubleshooting) above
- Review the [manual verification guide](../../e2e-testing/manual/grafana-verification.md)
- Search existing [GitHub issues](https://github.com/torrust/torrust-tracker-deployer/issues)
- Open a new issue with detailed logs and environment information

For general Grafana usage questions:

- [Grafana Community Forums](https://community.grafana.com/)
- [Grafana Documentation](https://grafana.com/docs/)
