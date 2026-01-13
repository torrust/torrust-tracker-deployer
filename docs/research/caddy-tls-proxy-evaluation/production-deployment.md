# Production Deployment - Caddy Configuration

**Date**: January 13, 2026  
**Server**: Hetzner ccx23 (46.224.206.37)  
**Path**: `/opt/torrust/`

## Overview

After successful evaluation in `/root/experiments/caddy-full-stack/`, the Caddy configuration has been deployed to the production directory `/opt/torrust/`. This serves as a working reference for creating Tera templates when implementing official HTTPS support.

## Deployment Process

### 1. Backup Current Configuration

```bash
cd /opt/torrust
docker compose down
tar -czf ~/torrust-backup-$(date +%Y%m%d-%H%M%S).tar.gz .
```

### 2. Copy Working Configuration

```bash
mkdir -p /opt/torrust/storage/caddy/etc
cp /root/experiments/caddy-full-stack/Caddyfile /opt/torrust/storage/caddy/etc/
cp /root/experiments/caddy-full-stack/docker-compose.yml /opt/torrust/
cp /root/experiments/caddy-full-stack/.env /opt/torrust/
cp /root/experiments/caddy-full-stack/prometheus.yml /opt/torrust/
```

### 3. Deploy Stack

```bash
cd /opt/torrust
docker compose up -d
```

## Production Configuration Files

### File Structure

```text
/opt/torrust/
├── .env                   # Environment variables (secrets)
├── docker-compose.yml    # Full stack orchestration
├── prometheus.yml        # Prometheus scrape configuration
└── storage/              # Persistent data
    ├── caddy/
    │   └── etc/
    │       └── Caddyfile  # Caddy reverse proxy configuration
    ├── grafana/
    ├── prometheus/
    └── tracker/
```

### Services

| Service    | Container Name | Image                   | Ports                         | Status  |
| ---------- | -------------- | ----------------------- | ----------------------------- | ------- |
| Caddy      | caddy          | caddy:2.10              | 80, 443 (TCP), 443 (UDP/QUIC) | Healthy |
| Tracker    | tracker        | torrust/tracker:develop | 1212, 7070, 6969/udp          | Healthy |
| Prometheus | prometheus     | prom/prometheus:v3.5.0  | 9090 (localhost only)         | Healthy |
| Grafana    | grafana        | grafana/grafana:12.3.1  | 3100                          | Healthy |

### Networks

- **metrics_network**: Connects Tracker ↔ Prometheus ↔ Caddy
- **visualization_network**: Connects Prometheus ↔ Grafana ↔ Caddy

## Verification Tests

### HTTPS Endpoints

All three domains are now accessible via HTTPS with automatic Let's Encrypt certificates:

```bash
# Tracker REST API
curl -I https://api.torrust-tracker.com/api/health_check
# Expected: HTTP/2 200

# HTTP Tracker
curl -I https://http1.torrust-tracker.com/health_check
# Expected: HTTP/2 200

# Grafana UI
curl -I https://grafana.torrust-tracker.com/
# Expected: HTTP/2 302 (redirect to login)
```

### Certificate Acquisition

Certificates obtained successfully for all domains:

- `api.torrust-tracker.com` - ✅ Obtained
- `http1.torrust-tracker.com` - ✅ Obtained
- `grafana.torrust-tracker.com` - ✅ Obtained

**Issuer**: Let's Encrypt (acme-v02.api.letsencrypt.org)  
**Method**: HTTP-01 challenge  
**Time**: ~4-5 seconds for all three certificates

### Service Health

```bash
docker compose ps
```

All services report `(healthy)` status:

- Caddy: Healthcheck via `caddy validate`
- Tracker: Built-in healthcheck to port 1313
- Prometheus: Healthcheck via `/-/healthy` endpoint
- Grafana: Healthcheck via `/api/health` endpoint

## Key Differences from Experiment

The production deployment is identical to the experiment configuration, with the only differences being:

1. **Network names**: `torrust_*` instead of `caddy-full-stack_*` (based on directory name)
2. **Volume names**: `torrust_*` instead of `caddy-full-stack_*`
3. **Container prefix**: No prefix (directory context)

All other configuration (Caddyfile, environment variables, service definitions) remains exactly the same.

## Next Steps for Tera Templates

When implementing issue for official Caddy support, use these production files as reference:

1. **storage/caddy/etc/Caddyfile** → `templates/caddy/Caddyfile.tera`

   - Template domain names (e.g., `{{ tracker_api_domain }}`)
   - Template admin email (e.g., `{{ admin_email }}`)
   - Follows project convention: config files in `storage/<service>/etc/`

2. **docker-compose.yml** → `templates/docker-compose/docker-compose.yml.tera`

   - Add conditional Caddy service block
   - Template port mappings
   - Template volume mounts

3. **prometheus.yml** → Already templated in `templates/prometheus/prometheus.yml.tera`

   - Already uses Tera templating
   - No additional changes needed

4. **.env** → `templates/docker-compose/.env.tera`
   - Already templated
   - Add Caddy-specific variables if needed

## Rollback Procedure

If needed, rollback to previous configuration:

```bash
cd /opt/torrust
docker compose down
tar -xzf ~/torrust-backup-YYYYMMDD-HHMMSS.tar.gz
docker compose up -d
```

## Monitoring

- **Caddy logs**: `docker logs caddy -f`
- **Certificate renewal**: Automatic (Caddy handles this)
- **Service health**: `docker compose ps`
- **HTTPS status**: Monitor via standard tools (curl, browser)

## Notes

- Experiment directory (`/root/experiments/caddy-full-stack/`) is still available for testing
- Production configuration is now the source of truth for Tera template creation
- All secrets remain in `.env` file (not committed to version control)
- Certificate data stored in `caddy_data` and `caddy_config` Docker volumes
