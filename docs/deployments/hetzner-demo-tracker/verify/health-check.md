# Health Check Verification

**Status**: ✅ Verified (2026-03-04)

## Endpoint

The tracker exposes a health check API on `127.0.0.1:1313` **inside the
tracker container** (loopback only). It is not accessible from the host or
from outside Docker. It is also not routed through Caddy.

The `show` command reports it as:

```text
"health_check_url": "http://46.225.234.201:1313/health_check"
"health_check_is_localhost_only": true
```

## How to Access

Because the endpoint is bound to the container's loopback interface, it must
be reached via `docker compose exec` on the server:

```bash
ssh -i ~/.ssh/<SSH_KEY> torrust@46.225.234.201 \
  "cd /opt/torrust && sudo docker compose exec tracker \
   sh -c 'wget -qO- http://127.0.0.1:1313/health_check'"
```

## Expected Response

A healthy deployment returns a JSON object with `"status": "Ok"` and one
entry per configured service:

```json
{
  "status": "Ok",
  "message": "",
  "details": [
    {
      "service_binding": "http://[::]:7071/",
      "binding": "[::]:7071",
      "service_type": "http_tracker",
      "info": "checking http tracker health check at: http://[::]:7071/health_check",
      "result": { "Ok": "200 OK" }
    },
    {
      "service_binding": "http://[::]:1212/",
      "binding": "[::]:1212",
      "service_type": "tracker_rest_api",
      "info": "checking api health check at: http://[::]:1212/api/health_check",
      "result": { "Ok": "200 OK" }
    },
    {
      "service_binding": "udp://[::]:6969",
      "binding": "[::]:6969",
      "service_type": "udp_tracker",
      "info": "checking the udp tracker health check at: [::]:6969",
      "result": { "Ok": "Connected" }
    },
    {
      "service_binding": "udp://[::]:6868",
      "binding": "[::]:6868",
      "service_type": "udp_tracker",
      "info": "checking the udp tracker health check at: [::]:6868",
      "result": { "Ok": "Connected" }
    },
    {
      "service_binding": "http://[::]:7070/",
      "binding": "[::]:7070",
      "service_type": "http_tracker",
      "info": "checking http tracker health check at: http://[::]:7070/health_check",
      "result": { "Ok": "200 OK" }
    }
  ]
}
```

## Actual Output (2026-03-04)

The above response matches exactly what was returned on verification. All
five services reported healthy:

| Service                      | Port | Result       |
| ---------------------------- | ---- | ------------ |
| HTTP Tracker 2 (`http2`)     | 7071 | ✅ 200 OK    |
| Tracker REST API             | 1212 | ✅ 200 OK    |
| UDP Tracker 1 (`udp1`, 6969) | 6969 | ✅ Connected |
| UDP Tracker 2 (`udp2`, 6868) | 6868 | ✅ Connected |
| HTTP Tracker 1 (`http1`)     | 7070 | ✅ 200 OK    |

## Notes

- `wget` is available in the tracker container image (`torrust/tracker:develop`)
  but `curl` is not.
- The health check is a loopback-only endpoint by design — it is not intended
  to be exposed externally.
- Port `1313` appears in `docker compose ps` output but is not published to
  the host (no `0.0.0.0:1313->1313/tcp` mapping).

## Public REST API Health Check

The Tracker REST API (port 1212, routed through Caddy) also exposes its own
health check endpoint publicly:

```bash
curl -s "https://api.torrust-tracker-demo.com/api/health_check"
```

This is a lighter check — it only confirms the REST API itself is responding,
not the individual tracker services. It does not require an auth token.
