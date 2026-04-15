# Deployer Docker Image Scan Results

Historical security scan results for Docker images used by the deployer itself.
These are [Priority 3](../../README.md) images — lower risk, short-lived, not internet-exposed.

For production image scans, see [`../../../production/scans/`](../../../production/scans/).

## Current Status Summary

| Image                      | Version | HIGH | CRITICAL | Status                                 | Last Scan    | Details                             |
| -------------------------- | ------- | ---- | -------- | -------------------------------------- | ------------ | ----------------------------------- |
| `torrust/tracker-deployer` | trixie  | 46   | 1        | ⚠️ CRITICAL blocked (OpenTofu grpc-go) | Apr 15, 2026 | [View](torrust-tracker-deployer.md) |

## Scan Archives

- [torrust-tracker-deployer.md](torrust-tracker-deployer.md) — Deployer (base: rust:trixie)

For backup service scans (production container), see [`../../../production/scans/torrust-tracker-backup.md`](../../../production/scans/torrust-tracker-backup.md).
For SSH server and provisioned-instance scans (testing only), see [`../../../testing/scans/`](../../../testing/scans/).

## Build and Scan

```bash
# Build deployer image
docker build --target release --tag torrust/tracker-deployer:local --file docker/deployer/Dockerfile .

# Scan
trivy image --severity HIGH,CRITICAL torrust/tracker-deployer:local
```

See [`../README.md`](../README.md) for detailed scanning instructions.
