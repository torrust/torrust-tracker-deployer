# Service Verification

Manual verification procedures for all services in the Hetzner demo tracker
deployment. Run these after the `test` command to confirm that each service
is fully functional end-to-end.

## Services

| Service         | URL                                               | File                                     |
| --------------- | ------------------------------------------------- | ---------------------------------------- |
| HTTP Tracker    | `https://http1.torrust-tracker-demo.com/announce` | [http-tracker.md](http-tracker.md)       |
| UDP Tracker     | `udp://udp1.torrust-tracker-demo.com:6969`        | [udp-tracker.md](udp-tracker.md)         |
| Tracker API     | `https://api.torrust-tracker-demo.com/api/v1`     | [api.md](api.md)                         |
| Grafana         | `https://grafana.torrust-tracker-demo.com`        | [grafana.md](grafana.md)                 |
| Health Check    | `http://127.0.0.1:1313/health_check` (internal)   | [health-check.md](health-check.md)       |
| Docker Services | All containers                                    | [docker-services.md](docker-services.md) |
| MySQL Database  | `torrust_tracker` DB (internal)                   | [mysql.md](mysql.md)                     |
| Storage Volume  | `/opt/torrust/storage` on `sdb` (internal)        | [storage.md](storage.md)                 |

## Status

| Service         | Status      |
| --------------- | ----------- |
| HTTP Tracker    | ✅ Verified |
| UDP Tracker     | ✅ Verified |
| Tracker API     | ✅ Verified |
| Grafana         | ✅ Verified |
| Health Check    | ✅ Verified |
| Docker Services | ✅ Verified |
| MySQL Database  | ✅ Verified |
| Storage Volume  | ✅ Verified |

## Prerequisites

- The environment must be in `Running` state.
- The `test` command must have passed (even with DNS warnings).
- For UDP tracker tests: `openssl` and `xxd` must be available locally, or use
  a BitTorrent client.
- For API tests: `curl` must be available locally.

## Network Notes

All domain names resolve to the floating IP `116.202.176.169`. The instance IP
`46.225.234.201` is only used for direct SSH access. The health check endpoint
is bound to `localhost` on the server and is only accessible via SSH.
