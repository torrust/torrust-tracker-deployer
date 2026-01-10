# Quick Start: Docker Deployment

Deploy a Torrust Tracker to Hetzner Cloud using Docker in minutes.

## ⚠️ Important Limitation

**Docker only supports cloud providers (Hetzner).** For LXD local development, see [Native Installation](native.md).

## Prerequisites

- **Docker** installed and running
- **SSH key pair** for VM access (e.g., `~/.ssh/id_ed25519`)
- **Hetzner Cloud account** with API token ([create one](https://console.hetzner.cloud/))

## Step 1: Pull the Docker Image

```bash
docker pull torrust/tracker-deployer:latest
```

## Step 2: Create Working Directories

```bash
mkdir -p data build envs
chmod 700 envs  # Contains sensitive configuration
```

## Step 3: Generate Configuration Template

```bash
docker run --rm \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  torrust/tracker-deployer:latest \
  create template --provider hetzner envs/my-hetzner-env.json
```

## Step 4: Edit Configuration

Open `envs/my-hetzner-env.json` and update:

```json
{
  "environment": {
    "name": "my-hetzner-env"
  },
  "ssh_credentials": {
    "private_key_path": "/home/deployer/.ssh/id_ed25519",
    "public_key_path": "/home/deployer/.ssh/id_ed25519.pub",
    "username": "torrust",
    "port": 22
  },
  "provider": {
    "provider": "hetzner",
    "api_token": "YOUR_HETZNER_API_TOKEN_HERE",
    "server_type": "cx22",
    "location": "nbg1",
    "image": "ubuntu-24.04"
  },
  "tracker": {
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "YOUR_SECURE_ADMIN_TOKEN"
    }
  },
  "health_check_api": {
    "bind_address": "127.0.0.1:1313"
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "YOUR_SECURE_GRAFANA_PASSWORD"
  }
}
```

> **Important**: The `private_key_path` and `public_key_path` are paths **inside the container**. When you mount `~/.ssh:/home/deployer/.ssh:ro`, your host keys become available at `/home/deployer/.ssh/` inside the container.

### Generating Secure Passwords

For publicly exposed servers, **never use default passwords**. Generate secure random credentials:

```bash
# Generate a 32-character token for tracker HTTP API admin_token
openssl rand -base64 32

# Generate a 24-character password for Grafana admin
openssl rand -base64 18
```

**Store these passwords securely** (password manager) - you'll need them to access the services.

### Configuration Reference

| Field                                | Description                                         | Example                              |
| ------------------------------------ | --------------------------------------------------- | ------------------------------------ |
| `environment.name`                   | Unique environment identifier                       | `my-hetzner-env`                     |
| `ssh_credentials.private_key_path`   | Container path to SSH private key                   | `/home/deployer/.ssh/id_ed25519`     |
| `ssh_credentials.public_key_path`    | Container path to SSH public key                    | `/home/deployer/.ssh/id_ed25519.pub` |
| `provider.api_token`                 | Hetzner API token                                   | `hcloud_xxx...`                      |
| `provider.server_type`               | Server size                                         | `cx22`, `cx32`, `cx42`               |
| `provider.location`                  | Datacenter                                          | `nbg1`, `fsn1`, `hel1`               |
| `tracker.http_api.admin_token`       | Tracker API authentication token                    | (generated secure token)             |
| `tracker.health_check_api.bind`      | Health check binding (use `127.0.0.1` for security) | `127.0.0.1:1313`                     |
| `grafana.admin_user`                 | Grafana admin username                              | `admin`                              |
| `grafana.admin_password`             | Grafana admin password                              | (generated secure password)          |
| `prometheus.scrape_interval_in_secs` | Metrics scrape interval                             | `15`                                 |

See [Hetzner Provider Guide](../providers/hetzner.md) for all options.

## Step 5: Create Environment

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  create environment --env-file /var/lib/torrust/deployer/envs/my-hetzner-env.json
```

## Step 6: Provision Infrastructure

Create the Hetzner Cloud server:

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  provision my-hetzner-env
```

**Duration**: ~60-90 seconds

## Step 7: Configure Server

Install Docker and dependencies on the server:

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  configure my-hetzner-env
```

**Duration**: ~60-90 seconds

## Step 8: Release Tracker

Pull the tracker Docker image on the server:

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  release my-hetzner-env
```

**Duration**: ~10-15 seconds

## Step 9: Run Tracker

Start the tracker service:

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  run my-hetzner-env
```

**Duration**: ~10-15 seconds

## Step 10: Verify Deployment

Check your tracker is running:

```bash
# Show environment details (includes server IP)
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  torrust/tracker-deployer:latest \
  show my-hetzner-env

# Run built-in health tests
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  test my-hetzner-env
```

### Service Endpoints

After deployment, your services are available at (replace `<IP>` with your server's IP):

| Service      | URL                         | Notes                                  |
| ------------ | --------------------------- | -------------------------------------- |
| HTTP Tracker | `http://<IP>:7070/announce` | BitTorrent HTTP tracker announce       |
| UDP Tracker  | `udp://<IP>:6969/announce`  | BitTorrent UDP tracker announce        |
| Tracker API  | `http://<IP>:1212/api`      | Requires `admin_token` for auth        |
| Health Check | `http://<IP>:1313/health`   | Bound to localhost only (internal)     |
| Grafana      | `http://<IP>:3100`          | Metrics dashboard (admin credentials)  |
| Prometheus   | `http://localhost:9090`     | Internal only - not exposed externally |

> **Note**: The health check endpoint (`1313`) is bound to `127.0.0.1` by default for security. Access it via SSH if needed.

### Testing the API

```bash
# Get tracker stats (requires admin token)
curl "http://<IP>:1212/api/v1/stats?token=YOUR_ADMIN_TOKEN"

# Test HTTP tracker (should return 200)
curl -s -o /dev/null -w "%{http_code}" "http://<IP>:7070/announce"
```

## Clean Up

When you're done, destroy the environment to stop billing:

```bash
docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest \
  destroy my-hetzner-env
```

> ⚠️ **Important**: Remember to destroy Hetzner resources when not in use to avoid charges.

## Shell Alias (Optional)

Add this to your `~/.bashrc` or `~/.zshrc` for convenience:

```bash
alias deployer='docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest'
```

Then use:

```bash
deployer create template --provider hetzner envs/my-env.json
deployer provision my-env
deployer configure my-env
deployer release my-env
deployer run my-env
deployer destroy my-env
```

## Troubleshooting

### Permission Denied on SSH Keys

```bash
# Ensure correct permissions
chmod 700 ~/.ssh
chmod 600 ~/.ssh/id_ed25519
```

### API Token Invalid

- Verify token in [Hetzner Console](https://console.hetzner.cloud/)
- Ensure token has **Read & Write** permissions
- Check token is correctly copied (no extra spaces)

### Environment Not Found

Ensure you're mounting the `data` directory consistently:

```bash
ls -la ./data/  # Should show your environment
```

### Provisioning Failed - Cannot Re-provision

If provisioning fails, the environment enters a `ProvisionFailed` state. You **cannot** re-provision directly - you must destroy and recreate:

```bash
# Destroy the failed environment
deployer destroy my-hetzner-env

# Recreate from scratch
deployer create environment --env-file /var/lib/torrust/deployer/envs/my-hetzner-env.json
deployer provision my-hetzner-env
```

> **Tip**: Check `data/<env-name>/environment.json` to see the current state and error details.

### LXD Provider Not Working

This is expected. Docker only supports cloud providers. For LXD, use [Native Installation](native.md).

## Next Steps

- [Hetzner Provider Guide](../providers/hetzner.md) - Server types, locations, pricing
- [Docker Image Reference](../../../docker/deployer/README.md) - Advanced Docker usage
- [Command Reference](../commands/README.md) - All available commands

## Complete Workflow Script

For automation, here's the full workflow:

```bash
#!/bin/bash
set -e

ENV_NAME="my-hetzner-env"

# Setup
mkdir -p data build envs
chmod 700 envs

# Common docker run prefix
DEPLOYER="docker run --rm \
  -v $(pwd)/data:/var/lib/torrust/deployer/data \
  -v $(pwd)/build:/var/lib/torrust/deployer/build \
  -v $(pwd)/envs:/var/lib/torrust/deployer/envs \
  -v ~/.ssh:/home/deployer/.ssh:ro \
  torrust/tracker-deployer:latest"

# Generate template (edit afterwards with your API token)
$DEPLOYER create template --provider hetzner envs/${ENV_NAME}.json

echo "Edit envs/${ENV_NAME}.json with your Hetzner API token, then press Enter"
read

# Deploy
$DEPLOYER create environment --env-file /var/lib/torrust/deployer/envs/${ENV_NAME}.json
$DEPLOYER provision ${ENV_NAME}
$DEPLOYER configure ${ENV_NAME}
$DEPLOYER release ${ENV_NAME}
$DEPLOYER run ${ENV_NAME}

echo "Deployment complete! Run: $DEPLOYER show ${ENV_NAME}"
```
