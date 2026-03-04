# Deployment Journal: Hetzner Demo Tracker

**Issue**: [#405](https://github.com/torrust/torrust-tracker-deployer/issues/405)
**Date started**: 2026-03-03
**Domain**: `torrust-tracker-demo.com`
**Provider**: Hetzner Cloud

## Purpose

Deploy a public Torrust Tracker demo instance to Hetzner Cloud and document every step of the process. This journal will serve as the source material for a blog post on [torrust.com](https://torrust.com).

## Table of Contents

1. [Prerequisites](prerequisites.md) — Account setup, tools, SSH keys
2. [Deployment Specification](deployment-spec.md) — What we want to deploy: config decisions,
   endpoints, sanitized config
3. Deployment commands — step-by-step per deployer command:
   - [create](commands/create/README.md) — generate template, validate, create environment
   - [provision](commands/provision/README.md) — create the Hetzner VM
   - [configure](commands/configure/README.md) — install Docker and Docker Compose on the server
   - [release](commands/release/README.md) — pull and stage Docker images
   - [run](commands/run/README.md) — start all services
4. Post-provision manual steps (done once, before `configure`):
   - [DNS setup](post-provision/dns-setup.md) — assign floating IPs, create DNS records, verify
   - [Volume setup](post-provision/volume-setup.md) — create and mount Hetzner volume for storage
   - [Hetzner Backups](post-provision/hetzner-backups.md) — enable automated server backups (can be done any time after provisioning)
5. [Service Verification](verify/README.md) — verifying all services after deployment:
   - [HTTP Tracker](verify/http-tracker.md)
   - [UDP Tracker](verify/udp-tracker.md)
   - [Tracker API](verify/api.md)
   - [Grafana](verify/grafana.md)
   - [Health Check](verify/health-check.md)
   - [Docker Services](verify/docker-services.md)
   - [MySQL Database](verify/mysql.md)
   - [Storage Volume](verify/storage.md)
   - [Backup](verify/backup.md)
6. Problems — issues encountered, per command:
   - [create problems](commands/create/problems.md)
   - [provision problems](commands/provision/problems.md)
7. Improvements — recommended deployer improvements found during this deployment:
   - [provision improvements](commands/provision/improvements.md)
8. [Observations](observations.md) — cross-cutting insights and learnings about the deployer

## Deployment

> This section will be filled in as we execute each deployment phase.

### Phase 1: Setup and Prerequisites

See [prerequisites.md](prerequisites.md) for the complete checklist.

### Phase 2: Create and Configure Environment

See [deployment-spec.md](deployment-spec.md) for config decisions and the sanitized config.
See [commands/create/README.md](commands/create/README.md) for running the `create template`, `validate`, and
`create environment` commands.

### Phase 3: Provision Infrastructure

See [commands/provision/README.md](commands/provision/README.md) for running the `provision` command and server
details.

### Phase 3.5: Post-Provision Setup

Manual steps done once after provisioning, required before `configure`:

1. [DNS setup](post-provision/dns-setup.md) — assign floating IPs to the server and create DNS
   records for all six domains.
2. [Volume setup](post-provision/volume-setup.md) — create a 50 GB Hetzner volume and mount it
   at `/opt/torrust/storage` so persistent data lives on a separate disk.
3. [Hetzner Backups](post-provision/hetzner-backups.md) — enable automated daily server backups
   via the Hetzner Console (can be done at any time after provisioning).

See [post-provision/README.md](post-provision/README.md) for the full overview.

### Phase 4: Configure Instance

See [commands/configure/README.md](commands/configure/README.md) for running the `configure`
command. Installs Docker 28.2.2 and Docker Compose v2.29.2.

### Phase 5: Release Application

See [commands/release/README.md](commands/release/README.md) for running the `release`
command. Pulled and staged all Docker images (~134 s, state=`Released`).

### Phase 6: Run Services

See [commands/run/README.md](commands/run/README.md) for running the `run`
command. All services started successfully (state=`Running`).

### Phase 7: Verify Deployment

See [verify/README.md](verify/README.md) for the full verification index.
All 9 services verified — HTTP tracker, UDP tracker, Tracker API, Grafana,
health check, Docker services, MySQL database, storage volume, and backup.
Verification included end-to-end announce tests using the Torrust reference
client (`http_tracker_client` and `udp_tracker_client`).

## Service Endpoints

> Will be filled after deployment.

| Service        | URL                                               | Status     |
| -------------- | ------------------------------------------------- | ---------- |
| HTTP Tracker 1 | `https://http1.torrust-tracker-demo.com/announce` | ✅ Running |
| HTTP Tracker 2 | `https://http2.torrust-tracker-demo.com/announce` | ✅ Running |
| UDP Tracker 1  | `udp://udp1.torrust-tracker-demo.com:6969`        | ✅ Running |
| UDP Tracker 2  | `udp://udp2.torrust-tracker-demo.com:6868`        | ✅ Running |
| Tracker API    | `https://api.torrust-tracker-demo.com/api/v1`     | ✅ Running |
| Health Check   | `http://127.0.0.1:1313/health_check` (internal)   | ✅ Running |
| Grafana        | `https://grafana.torrust-tracker-demo.com`        | ✅ Running |

## Cost

> Will be documented after choosing server type.

| Resource | Monthly Cost (EUR) |
| -------- | ------------------ |
| Server   | TBD                |
| Total    | TBD                |
