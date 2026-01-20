# Manual Grafana Service Verification

This guide provides Grafana-specific verification steps for manual E2E testing. For the complete deployment workflow, see the [Manual E2E Testing Guide](README.md).

## Overview

This guide covers:

- Grafana container health and connectivity
- Dashboard and datasource provisioning verification
- Prometheus datasource connection validation
- End-to-end data flow testing (Tracker → Prometheus → Grafana)
- Grafana-specific troubleshooting

## Prerequisites

Complete the standard deployment workflow first (see [Manual E2E Testing Guide](README.md)):

1. ✅ Environment created with Prometheus and Grafana configuration
2. ✅ Infrastructure provisioned
3. ✅ Services configured
4. ✅ Software released
5. ✅ Services running

**Your environment configuration must include both Prometheus and Grafana**:

```json
{
  "prometheus": {
    "scrape_interval_in_secs": 15
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "SecurePassword123!"
  }
}
```

**Note:** Grafana requires Prometheus to be configured. The environment creation will fail if you try to enable Grafana without Prometheus.

## Grafana-Specific Verification

This section provides detailed Grafana verification steps that should be performed after completing the standard deployment workflow.

### Get the VM IP Address

Extract the instance IP from the environment state (see [main guide](README.md#step-3-provision-infrastructure) for details):

```bash
cat data/<env-name>/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip'
```

> **💡 Tip**: A `show` command is planned (issue [#241](https://github.com/torrust/torrust-tracker-deployer/issues/241)) that will display environment information including the IP address in a more user-friendly format. Once implemented, you'll be able to use:
>
> ```bash
> cargo run -- show <env-name>
> ```

## Verification Steps

### 1. Verify Grafana Container is Running

SSH into the VM and check that the Grafana container is running:

```bash
# SSH into the VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP>

# Check running containers
docker ps
```

**Expected output:**

You should see three containers running:

```text
CONTAINER ID   IMAGE                       COMMAND                  STATUS
a1b2c3d4e5f6   grafana/grafana:11.4.0     "/run.sh"                Up 2 minutes
b2d988505fae   prom/prometheus:v3.0.1     "/bin/prometheus --c…"   Up 2 minutes
f0e3124878de   torrust/tracker:develop    "/usr/local/bin/entr…"   Up 2 minutes (healthy)
```

**Key verification points:**

- ✅ `grafana/grafana:11.4.0` container is present
- ✅ Container status shows "Up" (not "Restarting" or "Exited")
- ✅ Port 3000 is exposed (`0.0.0.0:3000->3000/tcp`)

### 2. Verify Grafana Web Interface is Accessible

Test that you can access the Grafana web interface from your local machine:

```bash
# Test HTTP response (should get redirect to login page)
curl -v http://<VM_IP>:3000/
```

**Expected output:**

```text
< HTTP/1.1 302 Found
< Location: /login
```

This confirms Grafana is running and accessible. The 302 redirect is expected - it's redirecting unauthenticated requests to the login page.

**Browser access:**

Open your web browser and navigate to:

```text
http://<VM_IP>:3000/
```

You should see the Grafana login page.

**Key verification points:**

- ✅ HTTP response is 302 (redirect)
- ✅ Location header points to `/login`
- ✅ Browser shows Grafana login interface

### 3. Verify Authentication with Configured Credentials

Test that you can authenticate with the credentials from your environment configuration:

```bash
# Test with your configured credentials
curl -u admin:SecurePassword123! http://<VM_IP>:3000/api/datasources
```

**Expected output:**

```json
[]
```

An empty array indicates successful authentication (no datasources configured yet via API).

**Test with wrong credentials:**

```bash
# This should fail
curl -u admin:wrongpassword http://<VM_IP>:3000/api/datasources
```

**Expected output:**

```json
{
  "message": "Invalid username or password",
  "messageId": "password-auth.failed",
  "statusCode": 401,
  "traceID": ""
}
```

**Key verification points:**

- ✅ Correct credentials return HTTP 200
- ✅ Wrong credentials return HTTP 401
- ✅ Error message is clear: "Invalid username or password"

### 4. Verify Prometheus is Accessible from Grafana Container

Since Prometheus binds to `127.0.0.1:9090` on the VM (internal only), it's not directly accessible from outside. However, Grafana needs to access it. Let's verify the Docker network connectivity:

```bash
# SSH into the VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP>

# Check that Prometheus is NOT accessible externally (expected to fail)
curl -s http://localhost:9090/api/v1/targets
```

**Expected output:**

```text
curl: (7) Failed to connect to localhost port 9090 after 0 ms: Couldn't connect to server
```

This is **correct behavior** - Prometheus is bound to 127.0.0.1 and not accessible from the Docker host network.

```bash
# Now test from within the Grafana container (should succeed)
docker exec -it <grafana_container_id> wget -q -O - http://prometheus:9090/api/v1/targets | head -c 100
```

**Expected output:**

```json
{"status":"success","data":{"activeTargets":[...]}}...
```

**Key verification points:**

- ✅ Prometheus is NOT accessible from VM host (localhost)
- ✅ Prometheus IS accessible from Grafana container via service name
- ✅ Docker network allows inter-container communication

### 5. Verify Grafana Provisioning Files Are Deployed

Check that the Grafana provisioning files (datasource and dashboards) were correctly deployed to the VM:

```bash
# SSH into the VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP>

# Check datasource provisioning file
cat /opt/torrust/storage/grafana/provisioning/datasources/prometheus.yml
```

**Expected output:**

```yaml
apiVersion: 1

datasources:
  - name: Prometheus
    type: prometheus
    uid: prometheus
    access: proxy
    url: http://prometheus:9090
    isDefault: true
    editable: false
    jsonData:
      timeInterval: "15s"
      httpMethod: POST
```

**Key verification points:**

- ✅ File exists at the correct path
- ✅ `uid: prometheus` is set (critical for dashboard compatibility)
- ✅ URL is `http://prometheus:9090` (Docker service name)
- ✅ `timeInterval` matches your configured scrape interval

**Check dashboard provider configuration:**

```bash
# Check dashboard provider file
cat /opt/torrust/storage/grafana/provisioning/dashboards/torrust.yml
```

**Expected output:**

```yaml
apiVersion: 1

providers:
  - name: "Torrust Dashboards"
    orgId: 1
    folder: "Torrust Tracker"
    type: file
    disableDeletion: false
    updateIntervalSeconds: 10
    allowUiUpdates: true
    options:
      path: /etc/grafana/provisioning/dashboards/torrust
      foldersFromFilesStructure: false
```

**Check dashboard JSON files:**

```bash
# List dashboard files
ls -lh /opt/torrust/storage/grafana/provisioning/dashboards/torrust/

# Verify datasource UID in dashboards
grep -c '"uid": "prometheus"' /opt/torrust/storage/grafana/provisioning/dashboards/torrust/*.json
```

**Expected output:**

```text
/opt/torrust/storage/grafana/provisioning/dashboards/torrust/metrics.json:20
/opt/torrust/storage/grafana/provisioning/dashboards/torrust/stats.json:20
```

This shows that both dashboard files contain 20 references to the `prometheus` datasource UID (one for each panel).

### 6. Verify Prometheus Datasource in Grafana API

Check the Prometheus datasource configuration via Grafana API:

```bash
# List configured datasources
curl -u admin:SecurePassword123! http://<VM_IP>:3000/api/datasources
```

**Expected output:**

```json
[
  {
    "id": 1,
    "uid": "prometheus",
    "orgId": 1,
    "name": "Prometheus",
    "type": "prometheus",
    "typeName": "Prometheus",
    "typeLogoUrl": "public/app/plugins/datasource/prometheus/img/prometheus_logo.svg",
    "access": "proxy",
    "url": "http://prometheus:9090",
    "user": "",
    "database": "",
    "basicAuth": false,
    "isDefault": true,
    "jsonData": {
      "httpMethod": "POST",
      "timeInterval": "15s"
    },
    "readOnly": false
  }
]
```

**Key verification points:**

- ✅ Datasource `uid` is `"prometheus"` (must match dashboard references)
- ✅ Datasource type is `"prometheus"`
- ✅ URL is `"http://prometheus:9090"` (using Docker service name)
- ✅ Access mode is `"proxy"` (requests go through Grafana backend)
- ✅ Datasource is set as default (`"isDefault": true`)
- ✅ `jsonData` contains `timeInterval` matching your configuration

**⚠️ Critical:** The datasource `uid` must be `"prometheus"` to match the dashboard configurations. If you see a different UID (like `"ce6lwx047kutca"` from an old deployment), the dashboards will fail to load with "Datasource was not found" errors.

### 7. Test Datasource Connection and Query Metrics

Test that Grafana can successfully query metrics from Prometheus:

```bash
# Test datasource health check
curl -u admin:SecurePassword123! \
  "http://<VM_IP>:3000/api/datasources/proxy/1/api/v1/query?query=up"
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
          "__name__": "up",
          "instance": "tracker:1212",
          "job": "tracker_metrics"
        },
        "value": [1734699623.123, "1"]
      },
      {
        "metric": {
          "__name__": "up",
          "instance": "tracker:1212",
          "job": "tracker_stats"
        },
        "value": [1734699623.123, "1"]
      }
    ]
  }
}
```

**Query tracker-specific metrics:**

```bash
# Query total announces
curl -u admin:SecurePassword123! \
  "http://<VM_IP>:3000/api/datasources/proxy/1/api/v1/query?query=tracker_announces_total"
```

**Key verification points:**

- ✅ Status is `"success"`
- ✅ Both `tracker_metrics` and `tracker_stats` targets show `"1"` (up)
- ✅ Tracker-specific metrics return valid data
- ✅ Timestamps are recent (within last few seconds)

### 8. Verify End-to-End Data Flow (Tracker → Prometheus → Grafana)

Now verify that data flows correctly from the tracker through Prometheus to Grafana by generating actual tracker activity:

#### Step 8.1: Generate Tracker Activity

Make HTTP announce requests to the tracker to generate metrics:

```bash
# SSH into the VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP>

# Send a single HTTP announce request
curl -s -H 'X-Forwarded-For: 203.0.113.195' \
  'http://localhost:7070/announce?info_hash=%3C%A6%7F%CB%3C%0B%DE%85%91%1C%82%16%7B%ED%15S%83%00%22%15&peer_id=-qB00000000000000001&port=17548&uploaded=0&downloaded=0&left=0&event=started'
```

**Expected response (bencoded):**

```text
d8:completei1e10:incompletei0e8:intervali300e12:min intervali300e5:peerslee
```

This indicates a successful announce (1 complete peer, 0 incomplete, empty peers list).

**Generate multiple requests for better visualization:**

```bash
# Send 10 announce requests with different peer IDs
for i in {1..10}; do
  curl -s -H "X-Forwarded-For: 203.0.113.$((RANDOM % 255))" \
    "http://localhost:7070/announce?info_hash=%3C%A6%7F%CB%3C%0B%DE%85%91%1C%82%16%7B%ED%15S%83%00%22%15&peer_id=-qB0000000000000000$i&port=17548&uploaded=0&downloaded=0&left=0&event=started" \
    > /dev/null
done
echo "Sent 10 announce requests"
```

#### Step 8.2: Verify Tracker Metrics API

Check that the tracker is exposing the metrics:

```bash
# Query tracker metrics endpoint (JSON format)
curl -s 'http://localhost:1212/api/v1/metrics?token=MyAccessToken' | head -100
```

**Expected output (truncated):**

```json
{
  "metrics": [
    {
      "type": "counter",
      "name": "http_tracker_core_requests_received_total",
      "samples": [
        {
          "value": 10,
          "labels": [
            { "name": "request_kind", "value": "announce" },
            { "name": "server_binding_protocol", "value": "http" }
          ]
        }
      ]
    },
    {
      "type": "gauge",
      "name": "swarm_coordination_registry_torrents_total",
      "samples": [{ "value": 1.0 }]
    },
    {
      "type": "gauge",
      "name": "swarm_coordination_registry_peer_connections_total",
      "samples": [
        {
          "value": 10.0,
          "labels": [{ "name": "peer_role", "value": "seeder" }]
        }
      ]
    }
  ]
}
```

**Key metrics to verify:**

- `http_tracker_core_requests_received_total`: Should show 10 requests
- `swarm_coordination_registry_torrents_total`: Should show 1 torrent
- `swarm_coordination_registry_peer_connections_total`: Should show 10 seeders

#### Step 8.3: Verify Prometheus Has Scraped the Data

Query Prometheus directly to confirm it's collecting the tracker metrics:

```bash
# Query HTTP requests metric
curl -s 'http://localhost:9090/api/v1/query?query=http_tracker_core_requests_received_total' | jq .

# Query torrents metric
curl -s 'http://localhost:9090/api/v1/query?query=swarm_coordination_registry_torrents_total' | jq .

# Query seeders metric
curl -s 'http://localhost:9090/api/v1/query?query=swarm_coordination_registry_peer_connections_total' | jq '.data.result[] | select(.metric.peer_role=="seeder")'
```

**Expected outputs:**

```json
{
  "status": "success",
  "data": {
    "result": [
      {
        "metric": {
          "__name__": "http_tracker_core_requests_received_total",
          "instance": "tracker:1212",
          "job": "tracker_metrics",
          "request_kind": "announce"
        },
        "value": [1766259745.624, "10"]
      }
    ]
  }
}
```

**Key verification points:**

- ✅ Status is `"success"`
- ✅ Metric values match what the tracker API reports
- ✅ `job` label shows `"tracker_metrics"` or `"tracker_stats"`
- ✅ Timestamp is recent (within last scrape interval)

#### Step 8.4: Verify Grafana Dashboards Display the Data

Finally, verify that the Grafana dashboards can display the data:

**Via Browser:**

1. Open Grafana: `http://<VM_IP>:3000/`
2. Login with your credentials (admin / SecurePassword123!)
3. Navigate to Dashboards → Browse
4. Open the "Torrust Tracker" folder
5. Open "Torrust Tracker - Metrics" or "Torrust Tracker - Stats" dashboard

**Expected results in dashboards:**

- **Torrents panel**: Should show `1`
- **Seeders panel**: Should show `10`
- **HTTP requests graphs**: Should show activity over time
- **No "Datasource not found" errors**

**Via API (alternative):**

```bash
# Query via Grafana datasource proxy
curl -u admin:SecurePassword123! \
  "http://<VM_IP>:3000/api/datasources/proxy/1/api/v1/query?query=swarm_coordination_registry_torrents_total{job=\"tracker_metrics\"}" | jq .
```

**Expected output:**

```json
{
  "status": "success",
  "data": {
    "result": [
      {
        "metric": {
          "__name__": "swarm_coordination_registry_torrents_total",
          "instance": "tracker:1212",
          "job": "tracker_metrics"
        },
        "value": [1766259767.583, "1"]
      }
    ]
  }
}
```

**Key verification points:**

- ✅ Grafana can query Prometheus through the datasource proxy
- ✅ Data is flowing from tracker → Prometheus → Grafana
- ✅ Dashboard panels display actual values (not "N/A" or errors)
- ✅ Graphs show historical data (if enough time has passed for multiple scrapes)

## Troubleshooting

### Grafana Container Not Running

**Symptoms:**

- `docker ps` doesn't show Grafana container
- Container status is "Exited" or "Restarting"

**Diagnosis:**

```bash
# Check container logs
docker logs <grafana_container_id>

# Check if container exits immediately
docker ps -a | grep grafana
```

**Common causes:**

- Port 3000 already in use on the VM
- Invalid environment variable in `.env` file
- Insufficient permissions on data directory

### Cannot Access Grafana Web Interface

**Symptoms:**

- `curl http://<VM_IP>:3000/` times out or connection refused
- Browser cannot load the page

**Diagnosis:**

```bash
# Check if port is listening
ssh torrust@<VM_IP> "netstat -tlnp | grep 3000"

# Check container networking
docker inspect <grafana_container_id> | grep IPAddress

# Check firewall rules (if applicable)
ssh torrust@<VM_IP> "sudo ufw status"
```

**Solutions:**

- Verify container is running: `docker ps`
- Check container logs for errors: `docker logs <grafana_container_id>`
- Verify port mapping in docker-compose.yml

### Authentication Fails with Configured Password

**Symptoms:**

- Configured password doesn't work
- Error: "Invalid username or password"
- Can only login with default "admin/admin"

**Diagnosis:**

```bash
# Check what password is in the .env file
cat /opt/torrust/storage/docker-compose/.env | grep GF_SECURITY_ADMIN_PASSWORD

# Check environment variables in running container
docker exec <grafana_container_id> env | grep GF_SECURITY_ADMIN_PASSWORD
```

**Root cause:**

This was a bug where the configured password wasn't being passed from the environment config to the `.env` file. It was fixed by updating:

- `UserInputs::with_tracker()` to accept optional Prometheus/Grafana configs
- `EnvironmentContext::with_working_dir_and_tracker()` to pass configs through
- `Environment::with_working_dir_and_tracker()` to accept configs
- Create handler to pass configs instead of using defaults

**Solution:**

If you encounter this:

1. Verify your environment config file has the correct password
2. Destroy and recreate the environment with the latest code
3. Check that `data/your-env/environment.json` contains the correct password
4. Verify `build/your-env/docker-compose/.env` has the correct `GF_SECURITY_ADMIN_PASSWORD`

### Prometheus Datasource Connection Failed

**Symptoms:**

- Datasource shows as "Not working" in Grafana UI
- API queries return empty results or errors
- Datasource health check fails

**Diagnosis:**

```bash
# Test Prometheus connectivity from Grafana container
docker exec -it <grafana_container_id> wget -O - http://prometheus:9090/-/healthy

# Check Prometheus container logs
docker logs <prometheus_container_id>

# Verify Docker network
docker network inspect <network_name>
```

**Common causes:**

- Prometheus container not running
- Wrong datasource URL (should be `http://prometheus:9090`, not `http://localhost:9090`)
- Network connectivity issues between containers
- Prometheus not fully initialized yet

**Solutions:**

1. Verify Prometheus is running: `docker ps | grep prometheus`
2. Check datasource URL: should use Docker service name `prometheus`
3. Test network: `docker exec <grafana_container_id> ping prometheus`
4. Wait a few seconds for Prometheus to initialize after container start

### Dashboards Show "Datasource Not Found" Error

**Symptoms:**

- Dashboards load but all panels show error: "Datasource [UID] was not found"
- Example error: "Datasource ce6lwx047kutca was not found"
- All dashboard panels are empty with red error messages

**Root cause:**

The dashboard JSON files contain hardcoded datasource UIDs that don't match the provisioned datasource UID. This typically happens if:

- Dashboard files were copied from another installation (like torrust-demo)
- Datasource was recreated with a different UID
- Dashboard files weren't updated when datasource UID changed

**Diagnosis:**

```bash
# SSH into the VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP>

# Check what UID the dashboards are using
grep '"uid":' /opt/torrust/storage/grafana/provisioning/dashboards/torrust/*.json | head -5

# Check what UID the datasource actually has
cat /opt/torrust/storage/grafana/provisioning/datasources/prometheus.yml | grep uid
```

**Example mismatch:**

```bash
# Dashboard expects:
"uid": "ce6lwx047kutca"  # ❌ Wrong - from old demo installation

# But datasource has:
uid: prometheus  # ✅ Correct - what we provisioned
```

**Solution:**

The datasource template and dashboard files must use matching UIDs. The correct configuration is:

1. **Datasource template** (`templates/grafana/provisioning/datasources/prometheus.yml.tera`):

   ```yaml
   datasources:
     - name: Prometheus
       uid: prometheus # ← Fixed UID
   ```

2. **Dashboard JSON files** must reference the same UID:

   ```json
   {
     "datasource": {
       "type": "prometheus",
       "uid": "prometheus"  # ← Must match datasource
     }
   }
   ```

**If you encounter this issue:**

1. Verify the template source has the correct UID
2. Destroy and recreate the environment with updated templates
3. Check the deployed files match: `grep -c '"uid": "prometheus"' /opt/torrust/storage/grafana/provisioning/dashboards/torrust/*.json`
4. Should show 20 matches per dashboard file (one per panel)

**Prevention:**

- Always use `uid: prometheus` in the datasource template
- When importing dashboards from external sources, update all datasource UID references
- Validate dashboard UIDs match datasource UID before deployment

### Dashboards Show Placeholder Domain

**Symptoms:**

- Dashboard descriptions reference `tracker.example.com`
- URLs in dashboard descriptions don't match your actual deployment

**Expected behavior:**

This is intentional. The dashboards use `tracker.example.com` as a generic placeholder to indicate where metrics are collected from. Users should understand this is a placeholder and replace it with their actual tracker domain or IP address when customizing dashboards.

**If you need to customize:**

The placeholder domain appears only in dashboard **descriptions** (not in actual queries). To customize:

1. Export the dashboard JSON from Grafana UI
2. Search and replace `tracker.example.com` with your domain
3. Re-import the customized dashboard

## Testing Checklist

Use this checklist when verifying a Grafana deployment:

- [ ] Three containers running (grafana, prometheus, tracker)
- [ ] Grafana web interface accessible (HTTP 302 redirect to /login)
- [ ] Can authenticate with configured credentials
- [ ] Wrong credentials are rejected (HTTP 401)
- [ ] Prometheus NOT accessible from VM host (security check)
- [ ] Prometheus accessible from Grafana container
- [ ] Prometheus datasource configured
- [ ] Datasource health check passes
- [ ] Can query `up` metric successfully
- [ ] Can query tracker-specific metrics
- [ ] Both tracker targets show as "up" in results

## Browser-Based Verification

For a complete verification, you can also test through the Grafana web UI:

1. **Login**:
   - Navigate to `http://<VM_IP>:3000/`
   - Login with your configured credentials

2. **Check Datasource**:
   - Go to Configuration → Data Sources
   - Verify "Prometheus" datasource exists
   - Click "Test" button → should show "Data source is working"

3. **Explore Metrics**:
   - Go to Explore (compass icon in sidebar)
   - Select "Prometheus" datasource
   - Try queries:
     - `up` → should show both tracker targets
     - `tracker_announces_total` → should show tracker metrics
     - `tracker_metrics_scrape_duration_seconds` → should show scrape timing

4. **Create Dashboard**:
   - Create → Dashboard
   - Add Panel
   - Query: `rate(tracker_announces_total[5m])`
   - Should show announce rate graph

## Next Steps

After successful Grafana verification:

1. **Explore Dashboards**: Review the pre-loaded Torrust tracker dashboards
2. **Customize Dashboards**: Modify existing dashboards or create new ones for your specific needs
3. **Configure Alerts**: Set up alerting rules for important metrics (requires Alertmanager)
4. **Backup Grafana Data**: Export customized dashboards for version control
5. **Continue Testing**: Return to the [Manual E2E Testing Guide](README.md) for cleanup or additional verification

For troubleshooting common issues during manual testing, see the [Troubleshooting section](README.md#troubleshooting-manual-tests) in the main guide.

## References

- [Grafana Documentation](https://grafana.com/docs/grafana/latest/)
- [Grafana HTTP API](https://grafana.com/docs/grafana/latest/developers/http_api/)
- [Grafana Provisioning](https://grafana.com/docs/grafana/latest/administration/provisioning/)
- [Prometheus Data Source](https://grafana.com/docs/grafana/latest/datasources/prometheus/)
- [Torrust Tracker Metrics](https://github.com/torrust/torrust-tracker)
