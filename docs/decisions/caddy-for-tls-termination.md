# Decision: Caddy for TLS Termination

## Status

Accepted

## Date

2026-01-20

## Context

The Torrust Tracker Deployer needed automatic HTTPS support for all HTTP services:

- Tracker REST API
- HTTP Tracker(s)
- Grafana monitoring UI
- Health Check API

Key requirements:

1. **Automatic certificate management** - No manual certificate generation or renewal
2. **WebSocket support** - Grafana Live requires WebSocket connections
3. **Simple configuration** - Minimize operational complexity
4. **Docker-friendly** - Easy integration in Docker Compose deployments
5. **Production-ready** - Mature and reliable for production use

## Decision

We adopted **Caddy v2.10** as the TLS termination proxy for all HTTP services.

### Implementation

Caddy is deployed as a Docker container in the same Docker Compose stack, serving as a reverse proxy for all HTTP services that need HTTPS:

```yaml
services:
  caddy:
    image: caddy:2.10
    ports:
      - "80:80" # HTTP (ACME challenges)
      - "443:443" # HTTPS
      - "443:443/udp" # HTTP/3 (QUIC)
    volumes:
      - ./storage/caddy/etc/Caddyfile:/etc/caddy/Caddyfile:ro
      - caddy_data:/data # TLS certificates
      - caddy_config:/config
```

Configuration uses Caddy's simple Caddyfile format:

```caddyfile
{
    email admin@example.com
}

api.example.com {
    reverse_proxy tracker:1212
}

grafana.example.com {
    reverse_proxy grafana:3000
}
```

## Consequences

### Positive

- **Zero-configuration HTTPS**: Certificates are automatically obtained and renewed via Let's Encrypt
- **WebSocket support**: Caddy handles WebSocket upgrades transparently (no special configuration)
- **Simple configuration**: Caddyfile is ~21 lines vs hundreds for nginx+certbot
- **HTTP/3 support**: QUIC protocol support is included by default
- **Hot reload**: Configuration changes apply without service interruption
- **Production-proven**: Caddy has been stable since 2015 with large community

### Negative

- **Larger binary**: ~40MB vs ~4MB for Pingoo or ~1MB for nginx
- **Post-quantum cryptography requires custom build**: Standard Caddy doesn't include PQC, but can be compiled with [Cloudflare's Go fork (CFGo)](https://github.com/cloudflare/go) to support X25519Kyber768 key exchange. Pingoo includes X25519MLKEM768 by default.
- **Additional container**: One more service in the Docker Compose stack

### Neutral

- **Certificate storage**: Requires persistent volume (`caddy_data`) for certificates

## Alternatives Considered

### 1. Pingoo (Rust-based TLS proxy)

**Evaluated in**: [Issue #234](https://github.com/torrust/torrust-tracker-deployer/issues/234)

**Rejected because**: Pingoo strips the `Upgrade` HTTP header, breaking WebSocket connections required by Grafana Live.

```rust
// From Pingoo source (http_proxy_service.rs):
let dominated_headers = &[
    "host",
    "upgrade",  // ← WebSocket upgrade stripped!
    "connection",
    ...
];
```

**Result**: Experiments 1-3 (HTTP) succeeded, but Experiment 4 (Grafana WebSocket) failed.

**Upstream issue filed**: [pingooio/pingoo#23](https://github.com/pingooio/pingoo/issues/23)

### 2. nginx + certbot

**Traditional approach** with manual configuration.

**Rejected because**:

1. **Manual setup**: Must run certbot manually to generate first certificate
2. **Cron-based renewal**: Requires bash script/cronjob for certificate renewal
3. **Complex configuration**: ~200+ lines of nginx.conf for SSL, headers, locations
4. **WebSocket configuration**: Requires explicit `proxy_set_header Upgrade` and `Connection` headers
5. **Multi-domain complexity**: Each subdomain needs separate certificate management

### 3. Traefik

**Alternative reverse proxy** with automatic HTTPS.

**Not evaluated** because Caddy already met all requirements with simpler configuration.

## Related Decisions

- [prometheus-integration-pattern.md](./prometheus-integration-pattern.md) - Prometheus is enabled by default
- [grafana-integration-pattern.md](./grafana-integration-pattern.md) - Grafana requires Prometheus dependency

## References

- [Issue #270 - Evaluate Caddy for HTTPS Termination](https://github.com/torrust/torrust-tracker-deployer/issues/270)
- [Issue #272 - Add HTTPS Support with Caddy](https://github.com/torrust/torrust-tracker-deployer/issues/272)
- [Issue #234 - Pingoo Evaluation](https://github.com/torrust/torrust-tracker-deployer/issues/234)
- [Caddy Official Documentation](https://caddyserver.com/docs/)
- [Caddy GitHub Repository](https://github.com/caddyserver/caddy)
- [Let's Encrypt Documentation](https://letsencrypt.org/docs/)
- [Go Post-Quantum with Caddy](https://sam-burns.com/posts/go-post-quantum-with-caddy/) - Tutorial on compiling Caddy with Cloudflare's Go fork for PQC support
