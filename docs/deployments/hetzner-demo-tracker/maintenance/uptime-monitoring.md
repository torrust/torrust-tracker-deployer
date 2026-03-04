# Uptime Monitoring

## Context

The Torrust demo instance previously hosted on DigitalOcean used DigitalOcean's
built-in **Monitoring** feature, which allows configuring uptime checks against
HTTP/HTTPS URLs. When a check fails, DigitalOcean sends an automatic email
alert. This made it easy to detect when services went down without any
additional infrastructure.

**Hetzner does not offer an equivalent native monitoring feature.** The Hetzner
Cloud Console has no uptime check or alert capability.

## Recommended External Tools

The following free or low-cost external services can replicate this
functionality:

| Tool                                                           | Free tier          | Notes                              |
| -------------------------------------------------------------- | ------------------ | ---------------------------------- |
| [UptimeRobot](https://uptimerobot.com/)                        | 50 monitors, 5 min | HTTP/HTTPS, email + webhook alerts |
| [Freshping](https://www.freshping.io/)                         | 50 monitors, 1 min | HTTP, email alerts                 |
| [Better Uptime](https://betterstack.com/)                      | Limited free tier  | HTTP, phone/SMS/email alerts       |
| [Checkly](https://www.checklyhq.com/)                          | Limited free tier  | HTTP + browser checks              |
| [statuspage.io](https://www.atlassian.com/software/statuspage) | Paid               | Public status page                 |

## What to Monitor

These are the public endpoints that should be checked:

| Endpoint                                                | Expected response |
| ------------------------------------------------------- | ----------------- |
| `https://api.torrust-tracker-demo.com/api/health_check` | HTTP 200          |
| `https://grafana.torrust-tracker-demo.com`              | HTTP 200          |
| `https://http1.torrust-tracker-demo.com/health_check`   | HTTP 200          |
| `https://http2.torrust-tracker-demo.com/health_check`   | HTTP 200          |

> **Note**: UDP tracker health cannot be checked by standard HTTP monitoring
> tools. Monitoring the HTTP API health check endpoint is sufficient as a
> proxy for overall service availability, since the tracker container runs
> all services (UDP + HTTP + API) as a single process.

## Status

⏳ Uptime monitoring not yet configured — tracked as a future improvement.
