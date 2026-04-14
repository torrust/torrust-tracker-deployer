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

- [ ] Check current OpenTofu version pinned in the Dockerfile:
      `grep -i opentofu docker/deployer/Dockerfile`
- [ ] Check latest OpenTofu release:
      <https://github.com/opentofu/opentofu/releases>
- [ ] Rebuild and re-scan:

  ```bash
  docker build --no-cache -t torrust/tracker-deployer:local docker/deployer/
  trivy image --severity HIGH,CRITICAL torrust/tracker-deployer:local
  ```

- [ ] Compare against the pass-1 baseline in
      `docs/security/docker/scans/torrust-tracker-deployer.md`
- [ ] For Debian base package CVEs, check fix availability:
      <https://security-tracker.debian.org/tracker/>
- [ ] Update `docs/security/docker/scans/torrust-tracker-deployer.md` with new
      scan results
- [ ] **If CRITICAL is cleared**: update Dockerfile OpenTofu version; post results
      comment; close #429
- [ ] **If only Debian packages improved**: post results comment; re-evaluate open
      status
- [ ] **If no change**: post comment with accepted risk rationale for remaining
      CVEs; label `accepted-risk`; leave open with revisit note

## Outcome

<!-- Fill in after doing the work -->

- Date:
- Current OpenTofu version in Dockerfile:
- Latest OpenTofu release:
- Findings after rebuild (HIGH / CRITICAL):
- Decision: fixed / partial / accepted risk
- Comment/PR:
