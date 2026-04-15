# Testing Image Scan Results

Historical security scan results for Docker images used only in automated tests or local development.
These are [Priority 4](../../README.md) images — they never run in production.

## Current Status Summary

| Image                                  | Version | HIGH | CRITICAL | Status                    | Last Scan   | Details                                         |
| -------------------------------------- | ------- | ---- | -------- | ------------------------- | ----------- | ----------------------------------------------- |
| `torrust/tracker-ssh-server`           | 3.23.3  | 0    | 0        | ✅ Remediated (vuln scan) | Apr 8, 2026 | [View](torrust-ssh-server.md)                   |
| `torrust/tracker-provisioned-instance` | 24.04   | 0    | 0        | ✅ Remediated (vuln scan) | Apr 8, 2026 | [View](torrust-tracker-provisioned-instance.md) |

## Scan Archives

- [torrust-ssh-server.md](torrust-ssh-server.md) — SSH test server (base: alpine:3.23.3), used for E2E integration tests
- [torrust-tracker-provisioned-instance.md](torrust-tracker-provisioned-instance.md) — Ubuntu VM simulation (base: ubuntu:24.04), used for E2E deployment workflow tests

## Build and Scan

```bash
# Build testing images
docker build --tag torrust/tracker-ssh-server:local docker/ssh-server/
docker build --tag torrust/tracker-provisioned-instance:local docker/provisioned-instance/

# Scan
trivy image --severity HIGH,CRITICAL torrust/tracker-ssh-server:local
trivy image --severity HIGH,CRITICAL torrust/tracker-provisioned-instance:local
```
