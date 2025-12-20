# Manual Grafana Service Verification

This guide provides step-by-step instructions for manually verifying that the Grafana visualization service is correctly deployed, configured, and connected to Prometheus for displaying Torrust Tracker metrics.

## Prerequisites

- A deployed environment with both Prometheus and Grafana enabled
- SSH access to the target instance
- The tracker and Prometheus services must be running
- Basic knowledge of Docker and Grafana

## Environment Setup

This guide assumes you have completed the full deployment workflow:

```bash
# 1. Create environment with Prometheus and Grafana enabled
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

Your environment configuration should include both `prometheus` and `grafana` sections:

```json
{
  "environment": { "name": "your-env" },
  "tracker": { ... },
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
| torrust-tracker-vm-your-env | RUNNING | 10.140.190.167 (enp5s0) | ... | VIRTUAL-MACHINE |
```

The VM IP in this example is `10.140.190.167`.

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
- ✅ Port 3100 is exposed (`0.0.0.0:3100->3000/tcp`)

### 2. Verify Grafana Web Interface is Accessible

Test that you can access the Grafana web interface from your local machine:

```bash
# Test HTTP response (should get redirect to login page)
curl -v http://<VM_IP>:3100/
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
http://<VM_IP>:3100/
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
curl -u admin:SecurePassword123! http://<VM_IP>:3100/api/datasources
```

**Expected output:**

```json
[]
```

An empty array indicates successful authentication (no datasources configured yet via API).

**Test with wrong credentials:**

```bash
# This should fail
curl -u admin:wrongpassword http://<VM_IP>:3100/api/datasources
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

### 5. Verify Prometheus Datasource Configuration

Check the Prometheus datasource configuration in Grafana. Since datasources are configured through Grafana provisioning (via the Docker Compose deployment), we can verify they exist:

```bash
# List configured datasources
curl -u admin:SecurePassword123! http://<VM_IP>:3100/api/datasources
```

**Expected output (if datasource was pre-configured):**

```json
[
  {
    "id": 1,
    "uid": "prometheus-ds",
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
    "jsonData": {},
    "readOnly": false
  }
]
```

**If datasource doesn't exist, add it via API:**

```bash
# Create Prometheus datasource
curl -X POST \
  -H "Content-Type: application/json" \
  -u admin:SecurePassword123! \
  http://<VM_IP>:3100/api/datasources \
  -d '{
    "name": "Prometheus",
    "type": "prometheus",
    "url": "http://prometheus:9090",
    "access": "proxy",
    "isDefault": true
  }'
```

**Expected output:**

```json
{
  "datasource": {
    "id": 1,
    "uid": "...",
    "orgId": 1,
    "name": "Prometheus",
    "type": "prometheus"
  },
  "id": 1,
  "message": "Datasource added",
  "name": "Prometheus"
}
```

**Key verification points:**

- ✅ Datasource type is `"prometheus"`
- ✅ URL is `"http://prometheus:9090"` (using Docker service name)
- ✅ Access mode is `"proxy"` (requests go through Grafana backend)
- ✅ Datasource is set as default (`"isDefault": true`)

### 6. Test Datasource Connection and Query Metrics

Test that Grafana can successfully query metrics from Prometheus:

```bash
# Test datasource health check
curl -u admin:SecurePassword123! \
  "http://<VM_IP>:3100/api/datasources/proxy/1/api/v1/query?query=up"
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
  "http://<VM_IP>:3100/api/datasources/proxy/1/api/v1/query?query=tracker_announces_total"
```

**Key verification points:**

- ✅ Status is `"success"`
- ✅ Both `tracker_metrics` and `tracker_stats` targets show `"1"` (up)
- ✅ Tracker-specific metrics return valid data
- ✅ Timestamps are recent (within last few seconds)

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

- Port 3100 already in use on the VM
- Invalid environment variable in `.env` file
- Insufficient permissions on data directory

### Cannot Access Grafana Web Interface

**Symptoms:**

- `curl http://<VM_IP>:3100/` times out or connection refused
- Browser cannot load the page

**Diagnosis:**

```bash
# Check if port is listening
ssh torrust@<VM_IP> "netstat -tlnp | grep 3100"

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

   - Navigate to `http://<VM_IP>:3100/`
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

After successful verification:

1. **Create Dashboards**: Design custom dashboards for your metrics
2. **Configure Alerts**: Set up alerting for important metrics
3. **Backup Grafana Data**: Export dashboards and datasource configurations
4. **Document Custom Queries**: Save useful PromQL queries for your team

## Future Automation

**Note:** The manual datasource configuration via API (shown in Step 5) could be automated in a future iteration by:

1. Creating a Grafana provisioning configuration file in the templates
2. Adding it to the Docker Compose volume mounts
3. Letting Grafana auto-configure datasources on startup

This would eliminate the need for manual API calls to create the datasource.

## References

- [Grafana Documentation](https://grafana.com/docs/grafana/latest/)
- [Grafana HTTP API](https://grafana.com/docs/grafana/latest/developers/http_api/)
- [Grafana Provisioning](https://grafana.com/docs/grafana/latest/administration/provisioning/)
- [Prometheus Data Source](https://grafana.com/docs/grafana/latest/datasources/prometheus/)
- [Torrust Tracker Metrics](https://github.com/torrust/torrust-tracker)
