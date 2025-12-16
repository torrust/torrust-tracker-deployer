# Prometheus Monitoring Service

This guide covers the Prometheus monitoring service integration in the Torrust Tracker Deployer.

## Overview

The deployer includes Prometheus for metrics collection by default. Prometheus automatically scrapes metrics from the tracker's HTTP API endpoints, providing real-time monitoring and historical data analysis.

## Default Behavior

- **Enabled by default** in generated environment templates
- Metrics collected from both `/api/v1/stats` and `/api/v1/metrics` endpoints
- Accessible via web UI on port `9090`
- Scrape interval: 15 seconds (configurable)

## Configuration

### Basic Configuration

Add the `prometheus` section to your environment configuration file:

```json
{
  "environment": {
    "name": "my-env"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub",
    "username": "torrust",
    "port": 22
  },
  "prometheus": {
    "scrape_interval": 15
  }
}
```

### Configuration Fields

**prometheus.scrape_interval** (optional):

- Metrics collection interval in seconds
- Default: `15` seconds
- Minimum recommended: `5` seconds
- Typical values: `10-60` seconds

**Examples**:

```json
// High-frequency monitoring (5 seconds)
{
  "prometheus": {
    "scrape_interval": 5
  }
}

// Standard monitoring (15 seconds)
{
  "prometheus": {
    "scrape_interval": 15
  }
}

// Low-frequency monitoring (60 seconds)
{
  "prometheus": {
    "scrape_interval": 60
  }
}
```

## Disabling Prometheus

To deploy without Prometheus monitoring, simply remove the entire `prometheus` section from your environment config:

```json
{
  "environment": {
    "name": "my-env"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub",
    "username": "torrust",
    "port": 22
  }
  // No prometheus section = monitoring disabled
}
```

## Accessing Prometheus

After deployment, the Prometheus web UI is available at:

```text
http://<vm-ip>:9090
```

Where `<vm-ip>` is the IP address of your deployed VM instance.

### Finding Your VM IP

```bash
# Extract IP from environment state
INSTANCE_IP=$(cat data/<env-name>/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')
echo "Prometheus UI: http://$INSTANCE_IP:9090"
```

## Using the Prometheus UI

The Prometheus web interface provides several capabilities:

### 1. View Current Metrics

Navigate to **Status → Targets** to see:

- Tracker endpoint health (up/down status)
- Last scrape time
- Scrape duration
- Error messages (if any)

### 2. Query Metrics

Use the **Graph** tab to query metrics:

**Example Queries**:

```promql
# Total announced peers
torrust_tracker_announced_peers_total

# Scrape duration
up{job="tracker"}

# Rate of announcements per second
rate(torrust_tracker_announced_peers_total[5m])
```

### 3. Explore Available Metrics

Navigate to **Graph → Insert metric at cursor** to see all available metrics from the tracker.

### 4. Check Target Health

Navigate to **Status → Targets** to verify:

- Both tracker endpoints are being scraped
- No error messages
- Recent successful scrapes

## Verification

For complete Prometheus verification steps, see the [Prometheus Verification Guide](../../e2e-testing/manual/prometheus-verification.md).

### Quick Verification

```bash
# Get VM IP
INSTANCE_IP=$(cat data/<env-name>/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')

# Check Prometheus container is running
ssh -i fixtures/testing_rsa torrust@$INSTANCE_IP "docker ps | grep prometheus"

# Check Prometheus is accessible
curl -s http://$INSTANCE_IP:9090/-/healthy
# Expected: Prometheus is Healthy.

# Check tracker targets
curl -s http://$INSTANCE_IP:9090/api/v1/targets | jq '.data.activeTargets[] | {job: .labels.job, health: .health}'
```

## Troubleshooting

### Prometheus Container Not Running

**Check container status**:

```bash
INSTANCE_IP=$(cat data/<env-name>/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')
ssh -i fixtures/testing_rsa torrust@$INSTANCE_IP "docker ps -a | grep prometheus"
```

**Check container logs**:

```bash
ssh -i fixtures/testing_rsa torrust@$INSTANCE_IP "docker logs prometheus"
```

### Targets Showing as Down

**Check tracker is running**:

```bash
ssh -i fixtures/testing_rsa torrust@$INSTANCE_IP "docker ps | grep tracker"
```

**Check tracker HTTP API is accessible**:

```bash
ssh -i fixtures/testing_rsa torrust@$INSTANCE_IP "curl -s http://tracker:6969/api/v1/stats"
```

**Check Prometheus configuration**:

```bash
ssh -i fixtures/testing_rsa torrust@$INSTANCE_IP "cat /opt/torrust/storage/prometheus/etc/prometheus.yml"
```

### Metrics Not Being Scraped

**Verify scrape interval**:

```bash
# Check your environment config
cat envs/<your-config>.json | jq '.prometheus.scrape_interval'
```

**Check Prometheus config on VM**:

```bash
INSTANCE_IP=$(cat data/<env-name>/environment.json | jq -r '.Running.context.runtime_outputs.instance_ip')
ssh -i fixtures/testing_rsa torrust@$INSTANCE_IP "cat /opt/torrust/storage/prometheus/etc/prometheus.yml | grep scrape_interval"
```

### Port 9090 Not Accessible

**Check port is exposed in docker-compose**:

```bash
ssh -i fixtures/testing_rsa torrust@$INSTANCE_IP "cat /opt/torrust/docker-compose.yml | grep -A 5 'prometheus:'"
```

**Check firewall rules** (if applicable):

```bash
ssh -i fixtures/testing_rsa torrust@$INSTANCE_IP "sudo ufw status"
```

## Architecture

### Deployment Structure

Prometheus is deployed as a Docker container alongside the tracker:

```text
VM Instance
├── /opt/torrust/
│   ├── docker-compose.yml        # Defines prometheus service
│   ├── storage/
│   │   └── prometheus/
│   │       └── etc/
│   │           └── prometheus.yml # Prometheus configuration
│   └── .env                       # Environment variables
```

### Configuration Generation

The deployer generates the Prometheus configuration file from templates:

1. **Template**: `templates/tracker/prometheus.yml.tera`
2. **Build Directory**: `build/<env-name>/prometheus/prometheus.yml`
3. **Deployment**: Ansible copies to `/opt/torrust/storage/prometheus/etc/prometheus.yml`

### Docker Compose Integration

When Prometheus is enabled, the deployer adds the service to `docker-compose.yml`:

```yaml
services:
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./storage/prometheus/etc/prometheus.yml:/etc/prometheus/prometheus.yml:ro
    command:
      - "--config.file=/etc/prometheus/prometheus.yml"
    networks:
      - tracker-network
    depends_on:
      - tracker
```

## Related Documentation

- **[Prometheus Verification Guide](../../e2e-testing/manual/prometheus-verification.md)** - Detailed verification steps
- **[User Guide](../README.md)** - Main user guide
- **[Configuration Guide](../configuration/)** - Environment configuration details
- **[Quick Start Guide](../quick-start.md)** - Getting started with deployments
