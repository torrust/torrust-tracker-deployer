# Manual Prometheus Service Verification

This guide provides step-by-step instructions for manually verifying that the Prometheus metrics collection service is correctly deployed, configured, and collecting metrics from the Torrust Tracker.

## Prerequisites

- A deployed environment with Prometheus enabled in the configuration
- SSH access to the target instance
- The tracker service must be running
- Basic knowledge of Docker and Prometheus

## Environment Setup

This guide assumes you have completed the full deployment workflow:

```bash
# 1. Create environment with Prometheus enabled
cargo run -- create environment --env-file envs/your-config.json

# 2. Provision infrastructure
cargo run -- provision your-env

# 3. Configure services
cargo run -- configure your-env

# 4. Release software
cargo run -- release your-env

# 5. Run services
cargo run -- run your-env
```

Your environment configuration should include the `prometheus` section:

```json
{
  "environment": { "name": "your-env" },
  "tracker": { ... },
  "prometheus": {
    "scrape_interval": 15
  }
}
```

## Getting the VM IP Address

First, get the IP address of your deployed VM:

### For LXD VMs

```bash
# List all LXD instances
lxc list

# Find your instance (e.g., torrust-tracker-vm-your-env)
# Look for the IP address in the enp5s0 interface column
```

Example output:

```text
| torrust-tracker-vm-your-env | RUNNING | 10.140.190.249 (enp5s0) | ... | VIRTUAL-MACHINE |
```

The VM IP in this example is `10.140.190.249`.

## Verification Steps

### 1. Verify Prometheus Container is Running

SSH into the VM and check that the Prometheus container is running:

```bash
# SSH into the VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP>

# Check running containers
docker ps
```

**Expected output:**

You should see two containers running:

```text
CONTAINER ID   IMAGE                     COMMAND                  STATUS
b2d988505fae   prom/prometheus:v3.0.1    "/bin/prometheus --c…"   Up 2 minutes
f0e3124878de   torrust/tracker:develop   "/usr/local/bin/entr…"   Up 2 minutes (healthy)
```

**Key verification points:**

- ✅ `prom/prometheus:v3.0.1` container is present
- ✅ Container status shows "Up" (not "Restarting" or "Exited")
- ✅ Port 9090 is exposed (`0.0.0.0:9090->9090/tcp`)

### 2. Verify Prometheus Configuration File

Check that the Prometheus configuration file was deployed correctly:

```bash
# Check file exists and has correct permissions
ls -la /opt/torrust/storage/prometheus/etc/prometheus.yml

# View the configuration
cat /opt/torrust/storage/prometheus/etc/prometheus.yml
```

**Expected output:**

```yaml
# Prometheus Configuration for Torrust Tracker Metrics Collection

global:
  scrape_interval: 15s # How often to scrape metrics from targets

scrape_configs:
  # Tracker Statistics - Aggregate metrics about tracker state
  - job_name: "tracker_stats"
    metrics_path: "/api/v1/stats"
    params:
      token: ["<YOUR_ADMIN_TOKEN>"]
      format: ["prometheus"]
    static_configs:
      - targets: ["tracker:1212"]

  # Tracker Metrics - Detailed operational metrics
  - job_name: "tracker_metrics"
    metrics_path: "/api/v1/metrics"
    params:
      token: ["<YOUR_ADMIN_TOKEN>"]
      format: ["prometheus"]
    static_configs:
      - targets: ["tracker:1212"]
```

**Key verification points:**

- ✅ File exists at the correct path
- ✅ File is readable (permissions: `0644`)
- ✅ `scrape_interval` matches your configuration (e.g., `15s`)
- ✅ Admin token matches your tracker configuration
- ✅ Port matches your tracker HTTP API port (default: `1212`)
- ✅ Both `tracker_stats` and `tracker_metrics` jobs are configured

### 3. Verify Prometheus Targets are Up

Check that Prometheus is successfully scraping both tracker endpoints:

```bash
# From your local machine (not inside the VM)
curl -s http://<VM_IP>:9090/api/v1/targets | python3 -m json.tool
```

**Expected output:**

Look for the `activeTargets` array containing both jobs with `"health": "up"`:

```json
{
  "status": "success",
  "data": {
    "activeTargets": [
      {
        "labels": {
          "instance": "tracker:1212",
          "job": "tracker_metrics"
        },
        "scrapeUrl": "http://tracker:1212/api/v1/metrics?format=prometheus&token=...",
        "lastError": "",
        "health": "up",
        "scrapeInterval": "15s"
      },
      {
        "labels": {
          "instance": "tracker:1212",
          "job": "tracker_stats"
        },
        "scrapeUrl": "http://tracker:1212/api/v1/stats?format=prometheus&token=...",
        "lastError": "",
        "health": "up",
        "scrapeInterval": "15s"
      }
    ]
  }
}
```

**Key verification points:**

- ✅ Both `tracker_metrics` and `tracker_stats` jobs are present
- ✅ `health` field shows `"up"` for both targets
- ✅ `lastError` field is empty (`""`)
- ✅ `scrapeInterval` matches your configuration
- ✅ `lastScrape` timestamp is recent (within the last minute)

**If targets are down:**

Check the `lastError` field for error messages:

- **Connection refused**: Tracker container might not be running or healthy
- **Authentication error**: Admin token mismatch between config files
- **Timeout**: Network connectivity issues or tracker overloaded

### 4. Verify Tracker Endpoints Directly

Test the tracker metrics endpoints directly to ensure they're accessible:

```bash
# Test the stats endpoint
curl -s "http://<VM_IP>:1212/api/v1/stats?token=<YOUR_ADMIN_TOKEN>&format=prometheus"

# Test the metrics endpoint
curl -s "http://<VM_IP>:1212/api/v1/metrics?token=<YOUR_ADMIN_TOKEN>&format=prometheus"
```

**Expected output (stats endpoint):**

```text
torrents 0
seeders 0
completed 0
leechers 0
tcp4_connections_handled 0
tcp4_announces_handled 0
tcp4_scrapes_handled 0
udp_requests_aborted 0
udp4_requests 18
udp4_connections_handled 18
...
```

**Expected output (metrics endpoint):**

```text
# HELP torrust_tracker_announce_requests_total Total number of announce requests
# TYPE torrust_tracker_announce_requests_total counter
torrust_tracker_announce_requests_total 0

# HELP torrust_tracker_torrents_total Total number of torrents tracked
# TYPE torrust_tracker_torrents_total gauge
torrust_tracker_torrents_total 0
...
```

**Key verification points:**

- ✅ Both endpoints return metrics data (not authentication errors)
- ✅ Response is in Prometheus text format
- ✅ Metrics contain tracker-specific data (torrents, peers, etc.)

### 5. Verify Prometheus UI is Accessible

Access the Prometheus web UI to verify it's working:

```bash
# Test that Prometheus UI is accessible
curl -s http://<VM_IP>:9090 | head -5
```

**Expected output:**

```html
<a href="/query">Found</a>.
```

**Alternative verification:**

Open a web browser and navigate to:

```text
http://<VM_IP>:9090
```

You should see the Prometheus UI with:

- ✅ Query interface at the top
- ✅ Navigation menu (Alerts, Graph, Status, Help)
- ✅ No error messages

**Try a sample query:**

1. Navigate to `http://<VM_IP>:9090/graph`
2. In the query box, enter: `torrust_tracker_torrents_total`
3. Click "Execute"
4. Switch to "Graph" tab

You should see a graph (even if it's flatlined at 0 if no torrents are tracked yet).

### 6. Verify Metrics are Being Collected

Query Prometheus to ensure it's storing metrics:

```bash
# Query for a specific metric
curl -s "http://<VM_IP>:9090/api/v1/query?query=up" | python3 -m json.tool
```

**Expected output:**

```json
{
  "status": "success",
  "data": {
    "resultType": "vector",
    "result": [
      {
        "metric": {
          "job": "tracker_metrics",
          "instance": "tracker:1212"
        },
        "value": [1734285600, "1"]
      },
      {
        "metric": {
          "job": "tracker_stats",
          "instance": "tracker:1212"
        },
        "value": [1734285600, "1"]
      }
    ]
  }
}
```

**Key verification points:**

- ✅ Query returns successfully (`"status": "success"`)
- ✅ Both targets show `"value": [..., "1"]` (indicating they're up)
- ✅ Timestamp is recent

### 7. Verify Data Over Time

Wait a few minutes (at least 2-3 scrape intervals) and check that data is accumulating:

```bash
# Query for metrics over the last 5 minutes
curl -s "http://<VM_IP>:9090/api/v1/query_range?query=up&start=$(date -u -d '5 minutes ago' +%s)&end=$(date -u +%s)&step=15s" | python3 -m json.tool
```

**Key verification points:**

- ✅ Multiple data points are returned (not just one)
- ✅ Data points are spaced according to `scrape_interval` (e.g., 15s apart)
- ✅ No gaps in the time series

## Common Issues and Troubleshooting

### Issue: Prometheus Container Not Running

**Symptoms:**

- `docker ps` doesn't show prometheus container
- Or shows container with "Restarting" or "Exited" status

**Diagnosis:**

```bash
# Check container logs
docker logs prometheus

# Check if container exists but stopped
docker ps -a | grep prometheus
```

**Common causes:**

1. **Configuration file syntax error**

   - Fix: Check prometheus.yml for YAML syntax errors
   - Validate: `docker run --rm -v /opt/torrust/storage/prometheus/etc:/etc/prometheus prom/prometheus:v3.0.1 promtool check config /etc/prometheus/prometheus.yml`

2. **Port 9090 already in use**

   - Check: `ss -tulpn | grep 9090`
   - Fix: Stop conflicting service or change Prometheus port

3. **Volume mount issues**
   - Fix: Verify `/opt/torrust/storage/prometheus/etc` exists and contains `prometheus.yml`

### Issue: Targets Showing as "Down"

**Symptoms:**

- Prometheus UI shows targets with red "DOWN" status
- `/api/v1/targets` shows `"health": "down"`

**Diagnosis:**

```bash
# Check tracker container is running and healthy
docker ps

# Test tracker endpoints manually
curl http://tracker:1212/api/v1/stats?token=<TOKEN>&format=prometheus

# Check Prometheus logs for scrape errors
docker logs prometheus | grep -i error
```

**Common causes:**

1. **Tracker container not running**

   - Fix: Check tracker container status with `docker ps`
   - Check logs: `docker logs tracker`

2. **Authentication token mismatch**

   - Verify: Token in `prometheus.yml` matches tracker's `admin_token`
   - Fix: Correct the token and restart Prometheus

3. **Network issues**
   - Verify: Containers are on the same Docker network
   - Check: `docker network inspect <network_name>`

### Issue: No Metrics Data

**Symptoms:**

- Prometheus UI shows empty graphs
- Queries return no data

**Possible causes:**

1. **Prometheus just started**

   - Wait for at least 1-2 scrape intervals
   - Check: `/api/v1/targets` shows `lastScrape` timestamp

2. **Query syntax error**

   - Verify metric names exist: `curl http://<VM_IP>:9090/api/v1/label/__name__/values`
   - Use Prometheus UI's autocomplete feature

3. **Time range issue**
   - Ensure you're querying the correct time range
   - Try: "Last 5 minutes" in the UI

## Advanced Verification

### Check Prometheus Configuration

Verify Prometheus is using the correct configuration:

```bash
# Inside the VM
docker exec prometheus cat /etc/prometheus/prometheus.yml
```

### Check Prometheus Storage

Verify Prometheus is persisting data:

```bash
# Inside the VM
docker exec prometheus ls -la /prometheus
```

### Monitor Scrape Duration

Check how long scrapes are taking:

```bash
curl -s "http://<VM_IP>:9090/api/v1/query?query=scrape_duration_seconds" | python3 -m json.tool
```

Scrape duration should be well under the scrape interval (e.g., < 1s for a 15s interval).

### Verify Prometheus Version

Confirm the correct Prometheus version is running:

```bash
curl -s http://<VM_IP>:9090/api/v1/status/buildinfo | python3 -m json.tool
```

Expected output includes:

```json
{
    "data": {
        "version": "3.0.1",
        ...
    }
}
```

## Success Criteria

Your Prometheus deployment is successful if:

- ✅ Prometheus container is running and stable
- ✅ Configuration file is correctly deployed
- ✅ Both tracker endpoints (stats and metrics) show `"health": "up"`
- ✅ Metrics are being collected and stored
- ✅ Prometheus UI is accessible
- ✅ Queries return expected data
- ✅ No errors in Prometheus or tracker logs
- ✅ Data accumulates over time

## Next Steps

Once Prometheus is verified:

1. **Add more scrape targets** - Configure additional services to monitor
2. **Set up alerts** - Define alerting rules for important metrics
3. **Connect Grafana** - Visualize metrics with dashboards
4. **Tune scrape intervals** - Adjust based on your monitoring needs
5. **Review retention** - Configure how long to keep metrics data

## Related Documentation

- [Prometheus Official Documentation](https://prometheus.io/docs/)
- [Torrust Tracker Metrics Documentation](https://github.com/torrust/torrust-tracker)
- [Main E2E Testing Guide](../manual-testing.md)
- [Prometheus Configuration Template](../../../templates/prometheus/prometheus.yml.tera)
