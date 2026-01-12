# Phase 1: Environment Preparation

This document captures all pre-checks and environment preparation steps completed before running the Pingoo experiments.

## Test Environment

| Component           | Value                  |
| ------------------- | ---------------------- |
| **Server Provider** | Hetzner Cloud          |
| **Server Type**     | ccx23 (dedicated vCPU) |
| **Location**        | Nuremberg (nbg1)       |
| **OS**              | Ubuntu 24.04           |
| **IPv4**            | 46.224.206.37          |
| **Domain**          | torrust-tracker.com    |
| **DNS Provider**    | cdmon.com (registrar)  |

## Pre-Checks Completed

### 1. Domain Configuration

**Date**: 2026-01-12

The domain `torrust-tracker.com` was configured with the following DNS records:

| Subdomain | Type | Value         | Purpose                           |
| --------- | ---- | ------------- | --------------------------------- |
| `test`    | A    | 46.224.206.37 | Experiment 1: Hello World         |
| `api`     | A    | 46.224.206.37 | Experiment 2: Tracker API         |
| `http1`   | A    | 46.224.206.37 | Experiment 3: HTTP Tracker        |
| `grafana` | A    | 46.224.206.37 | Experiment 4: Grafana (WebSocket) |

**Note**: Initially attempted to use Hetzner DNS, but switched to cdmon.com (the domain registrar) DNS servers due to permission issues with Hetzner DNS management.

### 2. DNS Propagation Verification

**Date**: 2026-01-12

Verified that all subdomains resolve correctly to the server IP:

```bash
$ dig +short test.torrust-tracker.com A
46.224.206.37

$ dig +short api.torrust-tracker.com A
46.224.206.37

$ dig +short http1.torrust-tracker.com A
46.224.206.37

$ dig +short grafana.torrust-tracker.com A
46.224.206.37
```

**Verification using Google DNS** (to confirm public propagation):

```bash
$ dig +short test.torrust-tracker.com A @8.8.8.8
46.224.206.37

$ dig +short api.torrust-tracker.com A @8.8.8.8
46.224.206.37
```

**Result**: ✅ All DNS records propagated successfully.

### 3. Server Accessibility

**Date**: 2026-01-12

All server accessibility checks passed:

```bash
$ ssh -i ~/.ssh/torrust_tracker_rsa root@46.224.206.37 "docker --version"
Docker version 28.2.2, build 28.2.2-0ubuntu1~24.04.1

$ ssh -i ~/.ssh/torrust_tracker_rsa root@46.224.206.37 "docker compose version"
Docker Compose version v2.29.2

$ ssh -i ~/.ssh/torrust_tracker_rsa root@46.224.206.37 "ss -tlnp | grep ':443' || echo 'Port 443 is free'"
Port 443 is free
```

- [x] SSH access to server (root@46.224.206.37)
- [x] Docker installed and running (v28.2.2)
- [x] Docker Compose available (v2.29.2)
- [x] Port 443 open (required for Pingoo TLS termination)
- [x] Port 80 open (optional, for HTTP redirect)

**SSH Key Used**: `~/.ssh/torrust_tracker_rsa` (fingerprint: `MD5:84:53:ea:6f:4a:62:4f:9d:5e:9f:59:49:fa:10:d2:d4`)

### 4. Pingoo Requirements

Pingoo uses `tls-alpn-01` ACME challenge method, which requires:

- **Port 443** must be publicly accessible
- **Valid domain** pointing to the server (verified above)
- **No existing service** binding to port 443 before Pingoo starts

## Issues Encountered

### DNS Provider Change

**Problem**: Initially configured DNS records in Hetzner DNS, but the records were not resolving.

**Cause**: The Hetzner account did not have proper privileges to manage DNS for the domain, or the domain's NS records were not pointing to Hetzner DNS servers.

**Solution**: Switched to using cdmon.com (the domain registrar) DNS servers, which have authoritative control over the domain.

**Lesson Learned**: When using a domain registrar, it's often simpler to use the registrar's DNS service rather than delegating to a third-party DNS provider, especially for testing purposes.

## Next Steps

~~Once all server accessibility checks pass:~~

All checks passed. Experiment 1 has been completed successfully.

1. ~~Proceed to [Experiment 1: Hello World](experiment-1-hello-world.md)~~ ✅ Complete
2. ~~Deploy Pingoo + nginx static page~~ ✅ Complete
3. ~~Verify automatic certificate generation~~ ✅ Complete
4. ~~Test HTTPS access to `https://test.torrust-tracker.com`~~ ✅ Complete

**Next**: Proceed to Experiment 2 (Tracker API).

## Checklist Summary

| Check               | Status | Notes                          |
| ------------------- | ------ | ------------------------------ |
| Domain purchased    | ✅     | torrust-tracker.com            |
| DNS records created | ✅     | 4 A records for subdomains     |
| DNS propagation     | ✅     | All subdomains resolving       |
| Server provisioned  | ✅     | Hetzner ccx23, Ubuntu 24.04    |
| SSH access          | ✅     | Using torrust_tracker_rsa key  |
| Docker ready        | ✅     | Docker 28.2.2, Compose v2.29.2 |
| Port 443 open       | ✅     | Available for Pingoo           |
