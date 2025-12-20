# Manual Tracker Service Verification

This guide provides Tracker-specific verification steps for manual E2E testing. For the complete deployment workflow, see the [Manual E2E Testing Guide](README.md).

## Overview

This guide covers:

- HTTP tracker announce/scrape endpoint testing
- UDP tracker endpoint testing (overview and future tooling)
- Tracker REST API testing
- Health check verification
- Tracker-specific troubleshooting

## Prerequisites

Complete the standard deployment workflow first (see [Manual E2E Testing Guide](README.md)):

1. ✅ Environment created
2. ✅ Infrastructure provisioned
3. ✅ Services configured
4. ✅ Software released
5. ✅ Services running

**Your environment configuration must include tracker settings**:

```json
{
  "tracker": {
    "core": {
      "database": {
        "driver": "sqlite3",
        "database_name": "tracker.db"
      }
    },
    "udp_trackers": [
      {
        "bind_address": "0.0.0.0:6969"
      }
    ],
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070"
      }
    ],
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    }
  }
}
```

## Tracker-Specific Verification

This section provides detailed tracker verification steps that should be performed after completing the standard deployment workflow.

### Get the VM IP Address

Extract the instance IP from the environment state (see [main guide](README.md#step-3-provision-infrastructure) for details):

```bash
cat data/<env-name>/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip'
```

> **💡 Tip**: A `show` command is planned (issue [#241](https://github.com/torrust/torrust-tracker-deployer/issues/241)) that will display environment information including the IP address in a more user-friendly format.

## Verification Steps

### 1. Verify Tracker Container is Running

SSH into the VM and check that the tracker container is running:

```bash
# SSH into the VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP>

# Check running containers
docker ps
```

**Expected output:**

```text
CONTAINER ID   IMAGE                     COMMAND                  CREATED          STATUS                    PORTS                                                                                                                                             NAMES
acb7e4fe0569   torrust/tracker:develop   "/usr/local/bin/entr…"   32 minutes ago   Up 32 minutes (healthy)   0.0.0.0:1212->1212/tcp, [::]:1212->1212/tcp, 0.0.0.0:7070->7070/tcp, [::]:7070->7070/tcp, 1313/tcp, 0.0.0.0:6969->6969/udp, [::]:6969->6969/udp   tracker
```

**Key verification points:**

- ✅ `torrust/tracker:develop` container is present
- ✅ Container status shows "Up" with "(healthy)" indicator
- ✅ UDP port 6969 is exposed (`0.0.0.0:6969->6969/udp`)
- ✅ HTTP port 7070 is exposed (`0.0.0.0:7070->7070/tcp`)
- ✅ API port 1212 is exposed (`0.0.0.0:1212->1212/tcp`)

### 2. Test HTTP Tracker Health Check

Verify the HTTP tracker health endpoint responds:

```bash
# From your local machine
export VM_IP=<your-vm-ip>

# Test HTTP tracker health
curl http://$VM_IP:7070/health_check
```

**Expected response:**

```json
{ "status": "Ok" }
```

**Verification points:**

- ✅ HTTP 200 OK status code
- ✅ JSON response with `"status":"Ok"` (note the capital 'O')
- ✅ Response received within 1-2 seconds

### 3. Test Tracker REST API Health Check

Verify the REST API health endpoint responds:

```bash
# Test API health
curl http://$VM_IP:1212/api/health_check
```

**Expected response:**

```json
{ "status": "Ok" }
```

**Verification points:**

- ✅ HTTP 200 OK status code
- ✅ JSON response with `"status":"Ok"` (note the capital 'O')

**Alternative API endpoints to verify:**

```bash
# Get tracker statistics
curl http://$VM_IP:1212/api/v1/stats?token=MyAccessToken

# Get Prometheus metrics
curl http://$VM_IP:1212/api/v1/metrics?token=MyAccessToken
```

### 4. Test HTTP Tracker Announce Endpoint

Test the HTTP tracker announce endpoint with a sample announce request:

```bash
# Test announce endpoint
curl "http://$VM_IP:7070/announce?info_hash=%3B%24U%04%CF%5F%11%BB%DB%E1%20%1C%EAjk%F4Z%EE%1B%C0&peer_id=-qB00000000000000001&port=17548&uploaded=0&downloaded=0&left=0&event=started"
```

**Expected response (bencoded):**

```text
d8:completei1e10:incompletei0e8:intervali300e12:min intervali300e5:peerslee
```

This is a valid bencoded dictionary response indicating:

- `complete`: 1 seeder
- `incomplete`: 0 leechers
- `interval`: 300 seconds (time before next announce)
- `min interval`: 300 seconds
- `peers`: Empty list (no other peers to return)

**Note about reverse proxy mode:**

If your tracker is configured with `on_reverse_proxy = true`, you'll need to include the `X-Forwarded-For` header:

```bash
curl -H "X-Forwarded-For: 203.0.113.195" \
  "http://$VM_IP:7070/announce?info_hash=%3B%24U%04%CF%5F%11%BB%DB%E1%20%1C%EAjk%F4Z%EE%1B%C0&peer_id=-qB00000000000000001&port=17548&uploaded=0&downloaded=0&left=0&event=started"
```

Without this header, you'll get an error:

```text
d14:failure reason208:Error resolving peer IP: missing or invalid the right most X-Forwarded-For IP (mandatory on reverse proxy tracker configuration)e
```

### 5. Test HTTP Tracker Scrape Endpoint

Test the scrape endpoint to get torrent statistics:

```bash
# Test scrape endpoint
curl "http://$VM_IP:7070/scrape?info_hash=%3B%24U%04%CF%5F%11%BB%DB%E1%20%1C%EAjk%F4Z%EE%1B%C0"
```

**Expected response (bencoded):**

```text
d5:filesd20:;$U04CF5F11BBDBE1201CEAjkF4ZEE1BC0d8:completei1e10:downloadedi0e10:incompletei0eeee
```

This shows statistics for the torrent:

- `complete`: 1 (number of seeders)
- `downloaded`: 0 (number of completed downloads)
- `incomplete`: 0 (number of leechers)

**Note:** Same reverse proxy considerations apply - add `X-Forwarded-For` header if needed.

### 6. Test REST API Endpoints

Test various REST API endpoints:

#### Get Tracker Statistics

```bash
curl "http://$VM_IP:1212/api/v1/stats?token=MyAccessToken" | jq
```

**Expected response:**

```json
{
  "torrents": 1,
  "seeders": 1,
  "completed": 0,
  "leechers": 0,
  "tcp4_connections_handled": 12,
  "tcp4_announces_handled": 11,
  "tcp4_scrapes_handled": 1,
  "tcp6_connections_handled": 0,
  "tcp6_announces_handled": 0,
  "tcp6_scrapes_handled": 0,
  "udp4_connections_handled": 377,
  "udp4_announces_handled": 0,
  "udp4_scrapes_handled": 0,
  "udp4_requests": 377,
  "udp6_connections_handled": 0,
  "udp6_announces_handled": 0,
  "udp6_scrapes_handled": 0,
  "udp6_requests": 0
}
```

#### Get Prometheus Metrics

```bash
curl "http://$VM_IP:1212/api/v1/metrics?token=MyAccessToken"
```

**Expected response (JSON format with metrics):**

```json
{
  "metrics": [
    {
      "type": "counter",
      "name": "udp_tracker_server_connection_id_errors_total",
      "help": "Total number of connection ID errors in the UDP tracker server",
      "samples": [{ "labels": {}, "value": 0.0 }]
    },
    {
      "type": "counter",
      "name": "tracker_core_persistent_torrents_downloads_total",
      "help": "Total number of torrents successfully downloaded to tracker persistent storage",
      "samples": [{ "labels": {}, "value": 0.0 }]
    },
    {
      "type": "counter",
      "name": "swarm_coordination_registry_peers_added_total",
      "help": "Total number of peers added to the registry",
      "samples": [
        { "labels": { "peer_type": "seeder" }, "value": 11.0 },
        { "labels": { "peer_type": "leecher" }, "value": 0.0 }
      ]
    }
  ]
}
```

> **Note**: The metrics endpoint returns JSON format containing an array of metric objects. Each metric includes type, name, help text, and sample values with optional labels.

### 7. UDP Tracker Testing (Advanced)

Testing the UDP tracker requires a BitTorrent UDP protocol client. While HTTP endpoints can be easily tested with `curl`, UDP requires specialized tooling.

#### Current State

The Torrust Tracker project includes a UDP client implementation at:

- **Repository**: https://github.com/torrust/torrust-tracker
- **Path**: `console/tracker-client` (in `develop` branch)
- **Status**: Not yet published as a crate

#### Using the UDP Client

To test UDP tracker functionality:

1. **Clone the tracker repository**:

   ```bash
   git clone https://github.com/torrust/torrust-tracker.git
   cd torrust-tracker
   git checkout develop
   ```

2. **Run the UDP client**:

   ```bash
   cd console/tracker-client
   cargo run -- udp --tracker-url "udp://$VM_IP:6969" \
     --info-hash "3B2455044CF55F11BBDBE1201CEA6A6BF45AEE1BC0"
   ```

3. **Expected behavior**:
   - Connection to UDP tracker succeeds
   - Announce request returns peer list
   - Scrape request returns torrent statistics

#### Alternative: Basic UDP Testing

For basic UDP connectivity testing without the specialized client:

```bash
# Test if UDP port is open (from local machine)
nc -u -v -w3 $VM_IP 6969

# Note: This only tests connectivity, not protocol compliance
# The tracker won't respond to arbitrary UDP packets
```

#### Future Tooling

> **📋 Note**: The UDP tracker client will be published as a standalone Rust crate in a future release, making UDP testing much easier. Once published, you'll be able to install it with:
>
> ```bash
> cargo install torrust-tracker-client
> torrust-tracker-client udp --tracker-url "udp://$VM_IP:6969" --info-hash <hash>
> ```

### 8. Verify Tracker Logs

Check tracker logs for any errors or warnings:

```bash
# SSH into the VM
ssh -i fixtures/testing_rsa -o StrictHostKeyChecking=no torrust@<VM_IP>

# View tracker logs
docker logs tracker

# Follow logs in real-time
docker logs -f tracker
```

**Look for:**

- ✅ No ERROR level messages
- ✅ Successful announce/scrape operations
- ✅ Health check requests logged
- ✅ UDP and HTTP servers handling requests successfully

**Example healthy log output:**

```text
2025-12-20T20:10:38.800766Z  INFO request{method=GET uri=/api/health_check version=HTTP/1.1}: API: request method=GET uri=/api/health_check request_id=50ea1dc8-fce1-4941-8fdd-7af67af8464d
2025-12-20T20:10:38.800889Z  INFO request{method=GET uri=/api/health_check version=HTTP/1.1}: API: response latency_ms=0 status_code=200 OK server_socket_addr=0.0.0.0:1212 request_id=50ea1dc8-fce1-4941-8fdd-7af67af8464d
2025-12-20T20:10:38.801562Z  INFO request{method=GET uri=/health_check version=HTTP/1.1}: HEALTH CHECK API: response latency_ms=35 status_code=200 OK request_id=1e842d0e-ee0a-47ef-b8fd-22d541a4c723
2025-12-20T20:10:38.836476Z  INFO torrust_tracker_swarm_coordination_registry::swarm::registry: active_peers_total=1 inactive_peers_total=0 active_torrents_total=1 inactive_torrents_total=0
2025-12-20T20:10:43.888743Z  INFO request{method=GET uri=/health_check version=HTTP/1.1}: HTTP TRACKER: request server_socket_addr=0.0.0.0:7070 method=GET uri=/health_check request_id=4e8be641-fc3d-4551-a929-10e347f7b8ba
2025-12-20T20:10:43.888770Z  INFO request{method=GET uri=/health_check version=HTTP/1.1}: HTTP TRACKER: response server_socket_addr=0.0.0.0:7070 latency_ms=0 status_code=200 OK request_id=4e8be641-fc3d-4551-a929-10e347f7b8ba
```

## Troubleshooting

### Tracker Container Not Running

**Symptoms:**

- `docker ps` doesn't show tracker container
- Health checks timeout or fail

**Diagnosis:**

```bash
# Check if container exists (including stopped)
docker ps -a | grep tracker

# Check container logs
docker logs tracker

# Check Docker Compose status
cd /opt/torrust
docker-compose ps
````

**Common causes:**

- Configuration error in tracker.toml
- Port conflicts (6969, 7070, or 1212 already in use)
- Database file permissions issues
- Invalid database configuration (MySQL connection failed)

**Solutions:**

1. **Fix configuration and restart**:

   ```bash
   # Edit configuration
   nano /opt/torrust/config/tracker/tracker.toml

   # Restart services
   docker-compose restart tracker
   ```

2. **Check port availability**:

   ```bash
   # Check if ports are already in use
   ss -tulpn | grep -E ':(6969|7070|1212)'
   ```

3. **Verify database connectivity** (if using MySQL):

   ```bash
   # Check MySQL container
   docker ps | grep mysql
   # Test MySQL connection
   docker exec mysql mysql -u tracker_user -p -e "SHOW DATABASES;"
   ```

### HTTP Tracker Returns 404

**Symptoms:**

- `curl http://$VM_IP:7070/announce` returns 404 Not Found
- Health check works but announce/scrape don't

**Diagnosis:**

```bash
# Check if HTTP tracker is enabled in config
docker exec tracker cat /etc/torrust/tracker/tracker.toml | grep -A5 "http_trackers"
```

**Solutions:**

- Ensure `http_trackers` array is not empty in configuration
- Verify bind address is `0.0.0.0:7070` (not `127.0.0.1`)
- Check firewall rules allow port 7070

### UDP Tracker Not Responding

**Symptoms:**

- UDP client times out
- No response from UDP tracker

**Diagnosis:**

```bash
# Check if UDP tracker is enabled
docker exec tracker cat /etc/torrust/tracker/tracker.toml | grep -A5 "udp_trackers"

# Check UDP port is listening
ss -ulpn | grep 6969
```

**Solutions:**

1. **Verify UDP tracker configuration**:

   ```bash
   # Ensure bind address is 0.0.0.0:6969
   docker exec tracker cat /etc/torrust/tracker/tracker.toml
   ```

2. **Check firewall** (UDP port 6969 must be open):

   ```bash
   # Check firewall status
   sudo ufw status
   # Open UDP port if needed
   sudo ufw allow 6969/udp
   ```

3. **Restart tracker**:

   ```bash
   docker-compose restart tracker
   ```

### API Authentication Failed

**Symptoms:**

- `curl http://$VM_IP:1212/api/v1/stats?token=MyAccessToken` returns 401 Unauthorized

**Diagnosis:**

```bash
# Check API token in configuration
docker exec tracker cat /etc/torrust/tracker/tracker.toml | grep admin_token
```

**Solutions:**

- Verify token in environment configuration matches tracker.toml
- Ensure token is URL-encoded if it contains special characters
- Token is case-sensitive - verify exact match

### Reverse Proxy Mode Issues

**Symptoms:**

- Announces fail with "missing client IP" error
- Tracker rejects announces without X-Forwarded-For header

**Diagnosis:**

```bash
# Check reverse proxy setting
docker exec tracker cat /etc/torrust/tracker/tracker.toml | grep on_reverse_proxy
```

**Solutions:**

If `on_reverse_proxy = true`:

```bash
# Always include X-Forwarded-For header
curl -H "X-Forwarded-For: 203.0.113.1" \
  "http://$VM_IP:7070/announce?..."
```

If not behind a reverse proxy, set `on_reverse_proxy = false`.

## Testing Checklist

After deployment, verify all tracker functionality:

- [ ] Tracker container is running with healthy status
- [ ] HTTP tracker health check responds (port 7070)
- [ ] HTTP API health check responds (port 1212)
- [ ] HTTP announce endpoint accepts requests and returns bencoded response
- [ ] HTTP scrape endpoint returns torrent statistics
- [ ] REST API `/stats` endpoint returns tracker statistics
- [ ] REST API `/metrics` endpoint returns Prometheus metrics
- [ ] UDP tracker port is listening (port 6969)
- [ ] Tracker logs show no errors
- [ ] Database connectivity confirmed (if using MySQL)

## Next Steps

After successful tracker verification:

1. **Generate Load Testing**: Use torrent clients to generate realistic announce/scrape traffic
2. **Monitor Performance**: Check metrics via Prometheus/Grafana (see [prometheus-verification.md](prometheus-verification.md) and [grafana-verification.md](grafana-verification.md))
3. **Test Database Scaling**: If using MySQL, test with larger peer counts (see [mysql-verification.md](mysql-verification.md))
4. **Configure Backup**: Set up automated backups for tracker database
5. **Continue Testing**: Return to the [Manual E2E Testing Guide](README.md) for cleanup or additional verification

For troubleshooting common issues during manual testing, see the [Troubleshooting section](README.md#troubleshooting-manual-tests) in the main guide.

## References

- [Torrust Tracker Documentation](https://github.com/torrust/torrust-tracker)
- [BitTorrent Protocol Specification (BEP 3)](http://www.bittorrent.org/beps/bep_0003.html)
- [UDP Tracker Protocol (BEP 15)](http://www.bittorrent.org/beps/bep_0015.html)
- [Torrust Tracker UDP Client](https://github.com/torrust/torrust-tracker/tree/develop/console/tracker-client)
- [Tracker HTTP API Documentation](https://github.com/torrust/torrust-tracker/blob/develop/docs/http-api.md)
