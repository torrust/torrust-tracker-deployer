# Issue #429: Deployer Image CVEs after Remediation Pass 1

**GitHub**: <https://github.com/torrust/torrust-tracker-deployer/issues/429>
**Image**: `torrust/tracker-deployer:local`
**Dockerfile**: `docker/deployer/Dockerfile`

---

## Context

After PR #436 removed `gnupg` from the runtime layer:

| Pass               | HIGH | CRITICAL |
| ------------------ | ---- | -------- |
| Before remediation | 49   | 1        |
| After pass 1       | 44   | 1        |

Remaining findings split into two areas:

1. **Debian 13.4 (trixie) base packages** — HIGH, blocked on upstream patches
2. **OpenTofu binary** — 2 HIGH + 1 CRITICAL, blocked on OpenTofu release

## Decision

**Re-scan and check OpenTofu release, then decide**:

- If a newer OpenTofu release clears the CRITICAL: update the pinned version,
  rebuild, re-scan, update scan doc, close #429
- If Debian packages are now patched: `docker build --no-cache` will pick them up;
  re-scan, update scan doc, re-evaluate #429
- If nothing has changed: post comment documenting current state and accepted risk;
  leave open with revisit note

## Steps

- [x] Check current OpenTofu version pinned in the Dockerfile:
      `grep -i opentofu docker/deployer/Dockerfile`
- [x] Check latest OpenTofu release:
      <https://github.com/opentofu/opentofu/releases>
- [x] Rebuild and re-scan:

  ```bash
  docker build --no-cache -t torrust/tracker-deployer:local docker/deployer/
  trivy image --severity HIGH,CRITICAL torrust/tracker-deployer:local
  ```

- [x] Compare against the pass-1 baseline in
      `docs/security/deployer/docker/scans/torrust-tracker-deployer.md`
- [x] For Debian base package CVEs, check fix availability:
      <https://security-tracker.debian.org/tracker/>
- [x] Update `docs/security/deployer/docker/scans/torrust-tracker-deployer.md` with new
      scan results
- [ ] **If CRITICAL is cleared**: update Dockerfile OpenTofu version; post results
      comment; close #429
- [ ] **If only Debian packages improved**: post results comment; re-evaluate open
      status
- [x] **If no change**: post comment with accepted risk rationale for remaining
      CVEs; label `accepted-risk`; leave open with revisit note

## Outcome

- Date: Apr 15, 2026
- Current OpenTofu version in Dockerfile: installed via script (no pinned version)
- Latest OpenTofu release: v1.11.6 (2026-04-08) — installed in rebuilt image
- Findings after rebuild (HIGH / CRITICAL): 46 HIGH / 1 CRITICAL
  - Debian OS: 42 HIGH, 0 CRITICAL
  - `usr/bin/tofu` (v1.11.6): 4 HIGH, 1 CRITICAL
- Decision: **leave open** — CRITICAL CVE-2026-33186 (grpc-go gRPC auth bypass) remains in tofu binary; requires OpenTofu upstream to bump grpc-go to v1.79.3+
- Comment/PR: PR #458, comment on #429
- Revisit: when OpenTofu ships v1.11.7+ or v1.12.x with updated grpc-go dependency
