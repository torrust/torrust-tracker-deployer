# Experiment 4: Grafana Dashboard with WebSocket

**Status**: ⚠️ PARTIAL SUCCESS (HTTP works, WebSocket FAILS)

## Objective

Test if Pingoo can serve a full Torrust monitoring stack including Grafana, which requires WebSocket connections for the Grafana Live feature (real-time updates).

## Configuration

**Domain**: `grafana.torrust-tracker.com`

### pingoo.yml

```yaml
listeners:
  https:
    address: https://0.0.0.0:443

tls:
  acme:
    domains:
      - grafana.torrust-tracker.com
    contact: admin@torrust.com

services:
  grafana:
    http_proxy: ["http://grafana:3000"]
```

### docker-compose.yml

```yaml
services:
  pingoo:
    image: pingooio/pingoo:latest
    container_name: pingoo
    network_mode: host
    volumes:
      - ./pingoo.yml:/etc/pingoo/pingoo.yml:ro
    restart: unless-stopped

  tracker:
    image: torrust/tracker:develop
    container_name: tracker
    ports:
      - "1212:1212" # API (internal)
      - "6969:6969/udp" # UDP tracker
      - "7070:7070" # HTTP tracker
    volumes:
      - ./tracker.toml:/etc/torrust/tracker/tracker.toml:ro
    restart: unless-stopped

  prometheus:
    image: prom/prometheus:v3.5.0
    container_name: prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml:ro
    restart: unless-stopped

  grafana:
    image: grafana/grafana:12.3.1
    container_name: grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=<your-secure-password>
      - GF_SERVER_ROOT_URL=https://grafana.torrust-tracker.com
      - GF_LIVE_ALLOWED_ORIGINS=https://grafana.torrust-tracker.com
    restart: unless-stopped
```

## Deployment Commands

```bash
ssh -i ~/.ssh/torrust_tracker_rsa root@46.224.206.37 "mkdir -p /root/experiments/experiment-4"

# Copy configuration files
scp -i ~/.ssh/torrust_tracker_rsa pingoo.yml root@46.224.206.37:/root/experiments/experiment-4/
scp -i ~/.ssh/torrust_tracker_rsa docker-compose.yml root@46.224.206.37:/root/experiments/experiment-4/
scp -i ~/.ssh/torrust_tracker_rsa tracker.toml root@46.224.206.37:/root/experiments/experiment-4/
scp -i ~/.ssh/torrust_tracker_rsa prometheus.yml root@46.224.206.37:/root/experiments/experiment-4/

# Deploy
ssh -i ~/.ssh/torrust_tracker_rsa root@46.224.206.37 "cd /root/experiments/experiment-4 && docker compose up -d"
```

## Results

### ✅ HTTP Requests - Working

Standard HTTP requests to Grafana work correctly:

```bash
curl -s -o /dev/null -w '%{http_code}' https://grafana.torrust-tracker.com/
# Returns: 200

curl -s -o /dev/null -w '%{http_code}' https://grafana.torrust-tracker.com/api/health
# Returns: 200

curl -s https://grafana.torrust-tracker.com/api/health
# Returns: {"commit":"9a98d91fd4","database":"ok","version":"12.3.1"}
```

The Grafana dashboard loads, user can log in, and dashboards display.

### ❌ WebSocket Connections - FAILING

Browser console error:

```text
WebSocket connection to 'wss://grafana.torrust-tracker.com/api/live/ws' failed:
WebSocket is closed before the connection is established
```

Pingoo container logs show:

```text
WARN [https] error serving HTTP connection: hyper::Error(IncompleteMessage)
DEBUG [https] peer closed connection without sending TLS close_notify, client=1.2.3.4:12345
```

### Root Cause Analysis

After investigating Pingoo's source code, the root cause was identified:

In [`http_proxy_service.rs`](https://github.com/pingooio/pingoo/blob/main/pingoo/services/http_proxy_service.rs#L26-L35), Pingoo explicitly removes "hop-by-hop" headers including the `Upgrade` header:

```rust
const HOP_HEADERS: &[&str] = &[
    "Connection",
    "Proxy-Connection",
    "Keep-Alive",
    "Proxy-Authenticate",
    "Proxy-Authorization",
    "Te",
    "Trailer",
    "Transfer-Encoding",
    "Upgrade",  // <-- THE PROBLEM
];
```

**The `Upgrade: websocket` header is required for HTTP-to-WebSocket protocol upgrade**, but Pingoo strips it from the request before forwarding to the upstream server.

### Technical Explanation

WebSocket connections begin as HTTP requests with special headers:

```http
GET /api/live/ws HTTP/1.1
Host: grafana.torrust-tracker.com
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: <base64-encoded-key>
Sec-WebSocket-Version: 13
```

The server responds with a `101 Switching Protocols` status to upgrade the connection:

```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: <base64-encoded-accept>
```

By removing the `Upgrade` header, Pingoo prevents this handshake from completing.

### Comparison with nginx

nginx handles WebSocket proxying with explicit configuration:

```nginx
location /api/live/ws {
    proxy_pass http://grafana:3000;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
}
```

Pingoo's HTTP proxy does not have equivalent configuration options for WebSocket support.

## Conclusion

**Pingoo v0.14.0 does not support WebSocket proxying.**

This is a known limitation of many simple reverse proxies. The `Upgrade` header is treated as a hop-by-hop header per HTTP/1.1 specification, but WebSocket requires it to be forwarded.

### Impact on Torrust Deployment

| Component    | Protocol   | Pingoo Support        |
| ------------ | ---------- | --------------------- |
| Tracker API  | HTTP/HTTPS | ✅ Works              |
| HTTP Tracker | HTTP/HTTPS | ✅ Works              |
| UDP Tracker  | UDP        | N/A (no proxy needed) |
| Grafana HTTP | HTTP/HTTPS | ✅ Works              |
| Grafana Live | WebSocket  | ❌ Fails              |

### Workarounds

1. **Disable Grafana Live**: Users can use Grafana without real-time updates (manual refresh)
2. **Hybrid architecture**: Use Pingoo for Tracker, nginx for Grafana
3. **TCP proxy mode**: Use Pingoo's `tcp+tls` listener instead of `https` (loses HTTP routing)
4. **Wait for Pingoo update**: WebSocket support may be added in future versions

## References

- [Pingoo HTTP Proxy Service Source](https://github.com/pingooio/pingoo/blob/main/pingoo/services/http_proxy_service.rs)
- [RFC 6455 - The WebSocket Protocol](https://datatracker.ietf.org/doc/html/rfc6455)
- [HTTP/1.1 Hop-by-Hop Headers](https://datatracker.ietf.org/doc/html/rfc7230#section-6.1)
- [Grafana Live Documentation](https://grafana.com/docs/grafana/latest/setup-grafana/set-up-grafana-live/)
