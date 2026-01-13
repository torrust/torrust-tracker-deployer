# Caddy Experiment Configuration Files

These are the configuration files used in the Caddy TLS proxy evaluation experiment.

**⚠️ Security Note**: All secrets (passwords, API tokens) have been redacted with `<REDACTED>` placeholders.

## Files

- **`storage/caddy/etc/Caddyfile`**: Caddy reverse proxy configuration for 3 domains (follows project convention)
- **`docker-compose.yml`**: Full stack (Caddy + Tracker + Prometheus + Grafana)
- **`.env`**: Environment variables (with redacted secrets)
- **`prometheus.yml`**: Prometheus scrape configuration (with redacted API token)

## Directory Structure

```text
experiment-files/
├── .env
├── docker-compose.yml
├── prometheus.yml
└── storage/
    └── caddy/
        └── etc/
            └── Caddyfile
```

**Note**: Configuration files follow project conventions where service configs are stored in `storage/<service>/etc/`.

## Deployment

These files were deployed to `/root/experiments/caddy-full-stack/` on the Hetzner test server (46.224.206.37).

See [`../experiment-full-stack.md`](../experiment-full-stack.md) for complete deployment procedure and results.

## Usage

To use these files:

1. Replace `<REDACTED>` placeholders with actual values
2. Copy production storage: `scp -r root@server:/opt/torrust/storage ./`
3. Deploy: `docker compose up -d`

For detailed instructions, see the experiment documentation.
