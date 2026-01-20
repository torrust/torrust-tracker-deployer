# HTTPS Support (TLS/SSL)

This guide covers enabling HTTPS for your Torrust Tracker deployment using automatic TLS certificates.

## Overview

The deployer includes [Caddy](https://caddyserver.com/) as an automatic TLS reverse proxy. When you enable HTTPS for any service, Caddy:

- Automatically obtains and renews TLS certificates from Let's Encrypt
- Handles HTTPS termination (services run HTTP internally)
- Redirects HTTP to HTTPS automatically
- Supports HTTP/2 and HTTP/3

## Prerequisites

Before enabling HTTPS, ensure you have:

1. **Domain names** - Valid domain names pointing to your server's IP address
2. **DNS configured** - A records for each domain pointing to your server
3. **Ports 80 and 443 open** - Required for Let's Encrypt certificate validation
4. **Public IP** - Your server must be reachable from the internet

> **Note**: For local testing with `.local` domains, Caddy uses its internal CA. Certificates will show browser warnings but work correctly.

## Configuration

### Global HTTPS Settings

Add an `https` section to your environment configuration:

```json
{
  "https": {
    "admin_email": "admin@example.com",
    "use_staging": false
  }
}
```

**Configuration Fields**:

| Field         | Required                      | Default | Description                             |
| ------------- | ----------------------------- | ------- | --------------------------------------- |
| `admin_email` | Yes (if any service uses TLS) | -       | Email for Let's Encrypt notifications   |
| `use_staging` | No                            | `false` | Use Let's Encrypt staging (for testing) |

### Enabling TLS Per Service

Each service that supports HTTPS has two fields:

- `domain` - The domain name for certificate acquisition
- `use_tls_proxy` - Whether to enable HTTPS via Caddy

**Both fields are required** to enable HTTPS for a service.

#### Tracker HTTP API

```json
{
  "tracker": {
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MySecureToken",
      "domain": "api.tracker.example.com",
      "use_tls_proxy": true
    }
  }
}
```

#### HTTP Trackers

Each HTTP tracker can independently have HTTPS enabled:

```json
{
  "tracker": {
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070",
        "domain": "http1.tracker.example.com",
        "use_tls_proxy": true
      },
      {
        "bind_address": "0.0.0.0:7071",
        "domain": "http2.tracker.example.com",
        "use_tls_proxy": true
      },
      {
        "bind_address": "0.0.0.0:7072"
      }
    ]
  }
}
```

In this example, the first two trackers use HTTPS while the third uses HTTP only.

#### Health Check API

```json
{
  "tracker": {
    "health_check_api": {
      "bind_address": "0.0.0.0:1313",
      "domain": "health.tracker.example.com",
      "use_tls_proxy": true
    }
  }
}
```

#### Grafana

```json
{
  "grafana": {
    "admin_user": "admin",
    "admin_password": "SecurePassword123!",
    "domain": "grafana.example.com",
    "use_tls_proxy": true
  }
}
```

## Let's Encrypt

### Production vs Staging

| Environment              | CA URL                                 | Rate Limits                      | Browser Trust     |
| ------------------------ | -------------------------------------- | -------------------------------- | ----------------- |
| **Production** (default) | `acme-v02.api.letsencrypt.org`         | 50 certs/week, 5 duplicates/week | ✅ Trusted        |
| **Staging**              | `acme-staging-v02.api.letsencrypt.org` | Much higher                      | ❌ Shows warnings |

**Use staging for**:

- Initial testing and development
- CI/CD environments
- Verifying configuration before production

```json
{
  "https": {
    "admin_email": "admin@example.com",
    "use_staging": true
  }
}
```

### Rate Limits

Production Let's Encrypt has these rate limits:

- **50 certificates per week** per registered domain
- **5 duplicate certificates per week** per domain set
- **300 pending authorizations** per account
- **5 failed validations** per hostname per hour

> **Tip**: Always test with `"use_staging": true` first to avoid hitting rate limits.

### Certificate Renewal

Caddy automatically renews certificates:

- Renewal attempts begin **30 days** before expiry
- Renewal happens **daily at random times** to distribute load
- **No manual intervention** required
- Admin email receives warnings if renewal fails

## Configuration Examples

### Example 1: All Services with HTTPS

Production deployment with HTTPS for all services:

```json
{
  "environment": {
    "name": "production"
  },
  "ssh_credentials": {
    "private_key_path": "~/.ssh/id_rsa",
    "public_key_path": "~/.ssh/id_rsa.pub"
  },
  "provider": {
    "provider": "lxd",
    "profile_name": "torrust-profile-prod"
  },
  "https": {
    "admin_email": "admin@example.com"
  },
  "tracker": {
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MySecureToken",
      "domain": "api.tracker.example.com",
      "use_tls_proxy": true
    },
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070",
        "domain": "http.tracker.example.com",
        "use_tls_proxy": true
      }
    ]
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "SecurePassword123!",
    "domain": "grafana.example.com",
    "use_tls_proxy": true
  },
  "prometheus": {
    "scrape_interval_in_secs": 15
  }
}
```

### Example 2: Only Tracker API with HTTPS

Secure only the API (contains sensitive admin token), other services use HTTP:

```json
{
  "https": {
    "admin_email": "admin@example.com"
  },
  "tracker": {
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MySecureToken",
      "domain": "api.tracker.example.com",
      "use_tls_proxy": true
    },
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070"
      }
    ]
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "SecurePassword123!"
  }
}
```

### Example 3: Local Testing with `.local` Domains

For local development using LXD with self-signed certificates:

```json
{
  "https": {
    "admin_email": "admin@tracker.local",
    "use_staging": true
  },
  "tracker": {
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken",
      "domain": "api.tracker.local",
      "use_tls_proxy": true
    },
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070",
        "domain": "http1.tracker.local",
        "use_tls_proxy": true
      }
    ]
  },
  "grafana": {
    "admin_user": "admin",
    "admin_password": "admin-password",
    "domain": "grafana.tracker.local",
    "use_tls_proxy": true
  }
}
```

> **Important**: Add entries to `/etc/hosts` on your machine to resolve `.local` domains:
>
> ```text
> <VM_IP>   api.tracker.local
> <VM_IP>   http1.tracker.local
> <VM_IP>   grafana.tracker.local
> ```

### Example 4: No HTTPS

To deploy without HTTPS, simply omit the `https` section and `domain`/`use_tls_proxy` fields:

```json
{
  "tracker": {
    "http_api": {
      "bind_address": "0.0.0.0:1212",
      "admin_token": "MyAccessToken"
    },
    "http_trackers": [
      {
        "bind_address": "0.0.0.0:7070"
      }
    ]
  }
}
```

## Disabling HTTPS

To disable HTTPS for a service, either:

1. **Remove both fields** - Omit `domain` and `use_tls_proxy`
2. **Set `use_tls_proxy: false`** - Keep domain but disable TLS

If no services use HTTPS, you can also remove the `https` section entirely.

## Port Behavior with HTTPS

When HTTPS is enabled for a service:

| Service      | Without TLS               | With TLS                              |
| ------------ | ------------------------- | ------------------------------------- |
| Tracker API  | Port exposed (e.g., 1212) | Port hidden, accessed via Caddy (443) |
| HTTP Tracker | Port exposed (e.g., 7070) | Port hidden, accessed via Caddy (443) |
| Grafana      | Port 3000 exposed         | Port hidden, accessed via Caddy (443) |

**Security benefit**: Backend service ports are not exposed when TLS is enabled, reducing attack surface.

## Verification

After deployment, verify HTTPS is working:

### Check Certificate

```bash
# Get VM IP from environment using the show command
torrust-tracker-deployer show <env-name>
# Look for "Instance IP" in the output and set it:
INSTANCE_IP=<your-vm-ip>

# Test HTTPS endpoint (replace domain with your actual domain)
curl -v --resolve api.tracker.example.com:443:$INSTANCE_IP https://api.tracker.example.com/api/health_check
```

### Verify HTTP to HTTPS Redirect

```bash
curl -I --resolve api.tracker.example.com:80:$INSTANCE_IP http://api.tracker.example.com/
# Should return: HTTP/1.1 308 Permanent Redirect
```

### Check Caddy Status

```bash
ssh -i <key> torrust@$INSTANCE_IP "docker logs caddy --tail 20"
```

### Expected Responses

| Endpoint     | Expected Response             | Notes                            |
| ------------ | ----------------------------- | -------------------------------- |
| Tracker API  | HTTP 500 (Unauthorized)       | Auth required, but TLS works     |
| HTTP Tracker | HTTP 404                      | GET not supported, but TLS works |
| Grafana      | HTTP 302 (Redirect to /login) | Login page loads                 |

## Troubleshooting

### Certificate Acquisition Failed

**Symptoms**: Caddy logs show ACME errors, browser shows certificate warnings.

**Solutions**:

1. **Check DNS** - Ensure domain points to your server's IP
2. **Check ports** - Ports 80 and 443 must be open and reachable
3. **Check rate limits** - Try staging mode first
4. **Check domain** - Must be a valid, publicly resolvable domain

### Connection Refused

**Symptoms**: `curl: (7) Failed to connect`

**Solutions**:

1. **Check Caddy is running**: `docker ps | grep caddy`
2. **Check firewall**: Ports 80, 443 must be open
3. **Check logs**: `docker logs caddy`

### Self-Signed Certificate Warning

**For `.local` domains**: This is expected. Caddy uses its internal CA for domains that can't be validated via Let's Encrypt.

**For real domains**: Check that DNS is configured correctly and the domain is publicly reachable.

### Invalid Configuration Errors

| Error                     | Cause                                    | Solution                                          |
| ------------------------- | ---------------------------------------- | ------------------------------------------------- |
| `TlsProxyWithoutDomain`   | `use_tls_proxy: true` without `domain`   | Add `domain` field                                |
| `InvalidDomain`           | Invalid domain format                    | Check domain syntax                               |
| `InvalidAdminEmail`       | Invalid email format                     | Check email syntax                                |
| `HttpsRequiresTlsService` | `https` section without any TLS services | Add `use_tls_proxy: true` to at least one service |
| `TlsRequiresHttpsSection` | TLS service without `https` section      | Add `https` section with `admin_email`            |

## Architecture

### How It Works

```text
Internet → Port 443 → Caddy (TLS termination) → HTTP → Service Container
                                               ↑
                                    Reverse proxy by domain
```

1. **Caddy receives HTTPS requests** on port 443
2. **Terminates TLS** using Let's Encrypt certificates
3. **Proxies to backend** via internal Docker network (HTTP)
4. **Returns response** over the encrypted connection

### Docker Network

- All services run in the same Docker network (`torrust-network`)
- Caddy accesses backend services by container name (e.g., `tracker:1212`)
- Backend ports are only exposed to Caddy, not to the host (when TLS enabled)

### Certificate Storage

Caddy stores certificates in Docker volumes:

- `caddy_data` - Certificates and private keys
- `caddy_config` - Caddy configuration cache

These volumes persist across container restarts, preventing unnecessary certificate reissuance.

## Related Documentation

- **[Security Guide](../security.md)** - Firewall and security configuration
- **[Grafana Service](grafana.md)** - Grafana-specific configuration
- **[Prometheus Service](prometheus.md)** - Prometheus-specific configuration
- **[Template Customization](../template-customization.md)** - Advanced template options
