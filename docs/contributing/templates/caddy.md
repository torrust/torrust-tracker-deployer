# Caddy Templates

Documentation for Caddy reverse proxy templates used for automatic HTTPS with Let's Encrypt.

## Overview

Caddy provides automatic HTTPS termination for HTTP services. The template generates a Caddyfile
based on which services have TLS configured in the environment configuration.

## Template Files

### `templates/caddy/Caddyfile.tera`

Dynamic Tera template that generates a Caddyfile. Only services with TLS configured
will have entries in the generated file.

## Template Variables

The template receives a `CaddyContext` with the following structure:

| Variable        | Type                       | Description                                         |
| --------------- | -------------------------- | --------------------------------------------------- |
| `admin_email`   | `String`                   | Admin email for Let's Encrypt notifications         |
| `use_staging`   | `bool`                     | Use Let's Encrypt staging environment (for testing) |
| `tracker_api`   | `Option<ServiceTlsConfig>` | TLS config for Tracker API (if enabled)             |
| `http_trackers` | `Vec<ServiceTlsConfig>`    | TLS configs for HTTP trackers (only those with TLS) |
| `grafana`       | `Option<ServiceTlsConfig>` | TLS config for Grafana (if enabled)                 |

### `ServiceTlsConfig` Structure

| Field    | Type     | Description                                   |
| -------- | -------- | --------------------------------------------- |
| `domain` | `String` | Domain name for this service                  |
| `port`   | `u16`    | Port number (pre-extracted from bind_address) |

## Context Data Preparation

Following the project's [Context Data Preparation Pattern](template-system-architecture.md#-context-data-preparation-pattern),
all data is pre-processed in Rust before being passed to the template:

- **Ports are extracted** from `bind_address` strings (e.g., `"0.0.0.0:7070"` → `7070`)
- **Only TLS-enabled services** are included in the context
- **The template receives ready-to-use values** - no parsing required

### Example: Port Extraction in Rust

```rust
// In the context builder (Rust code)
let http_api_port = tracker_config.http_api.bind_address.port(); // u16

// Context passed to template
CaddyContext {
    tracker_api: Some(ServiceTlsConfig {
        domain: "api.example.com".to_string(),
        port: http_api_port, // Already extracted as u16
    }),
    // ...
}
```

```tera
{# In the template - receives ready-to-use port #}
{{ tracker_api.domain }} {
    reverse_proxy tracker:{{ tracker_api.port }}
}
```

## Conditional Rendering

The template uses Tera conditionals to include only services with TLS configured:

- `{% if tracker_api %}` - Include API block only if TLS is enabled for API
- `{% for http_tracker in http_trackers %}` - Iterate only over trackers with TLS
- `{% if grafana %}` - Include Grafana block only if TLS is enabled

Services without TLS configuration remain accessible via HTTP on their configured ports.

## Let's Encrypt Environments

### Production (Default)

Uses the production Let's Encrypt API. Certificates are trusted by all browsers.

**Rate limits** (production):

- 50 certificates per registered domain per week
- 5 duplicate certificates per week

### Staging

Set `use_staging: true` in your environment configuration for testing:

```json
{
  "https": {
    "admin_email": "admin@example.com",
    "use_staging": true
  }
}
```

This configures Caddy to use `https://acme-staging-v02.api.letsencrypt.org/directory`.

**Important notes about staging**:

- Staging certificates will show browser warnings (not trusted by browsers)
- Use staging only for testing the HTTPS flow, not for production
- Staging has much higher rate limits than production

## Docker Compose Integration

When Caddy is enabled (any service has TLS configured), the following is added to `docker-compose.yml`:

- **Caddy service**: Runs `caddy:2.10` image with ports 80, 443, and 443/udp (HTTP/3)
- **proxy_network**: Network connecting Caddy to services it proxies
- **caddy_data volume**: Persists TLS certificates (critical for avoiding rate limits)
- **caddy_config volume**: Persists Caddy configuration cache

Services with TLS enabled are automatically connected to the `proxy_network`.

## Caddyfile Syntax Notes

- **Caddy requires TABS for indentation**, not spaces
- The template uses actual tab characters for proper Caddyfile formatting
- Global options are enclosed in `{ }` at the top of the file
- Site blocks use the format `domain.com { ... }`

## Related Documentation

- [Template System Architecture](template-system-architecture.md) - Overall template system design
- [Context Data Preparation Pattern](template-system-architecture.md#-context-data-preparation-pattern) - How to prepare data for templates
- [Tera Template Guidelines](tera.md) - Tera syntax and best practices
- [HTTPS Setup Guide](../../user-guide/https-setup.md) - User documentation (coming soon)
