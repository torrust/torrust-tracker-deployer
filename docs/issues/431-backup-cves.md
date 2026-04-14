# Issue #431: Backup Image CVEs after Remediation Pass 1

**GitHub**: <https://github.com/torrust/torrust-tracker-deployer/issues/431>
**Image**: `torrust/tracker-backup:local`
**Dockerfile**: `docker/backup/Dockerfile`

---

## Context

After PR #436 added `apt-get upgrade -y` to the base layer, findings did not change
(upstream Debian packages were not patched at the time):

| Pass               | HIGH | CRITICAL |
| ------------------ | ---- | -------- |
| Before remediation | 6    | 0        |
| After pass 1       | 6    | 0        |

All 6 HIGH are Debian 13.4 (trixie) base package CVEs.

## Decision

**Rebuild and re-scan to check if Debian packages are now patched, then decide**:

- If package fixes are now available: `docker build --no-cache` will pick them up
  automatically via `apt-get upgrade -y`; verify and close #431
- If still unpatched: post comment with current scan confirming same count, document
  accepted risk, close #431

## Steps

- [ ] Rebuild the image from scratch:
      `docker build --no-cache -t torrust/tracker-backup:local docker/backup/`
- [ ] Re-scan: `trivy image --severity HIGH,CRITICAL torrust/tracker-backup:local`
- [ ] Compare against the pass-1 baseline in
      `docs/security/docker/scans/torrust-tracker-backup.md`
- [ ] For each remaining CVE, check fix availability:
      <https://security-tracker.debian.org/tracker/>
- [ ] Update `docs/security/docker/scans/torrust-tracker-backup.md` with the new
      scan results
- [ ] **If HIGH count dropped**: post comment with before/after results; close #431
- [ ] **If no change**: post comment documenting that Debian upstream has not yet
      patched these CVEs with a revisit note; close #431

## Outcome

<!-- Fill in after doing the work -->

- Date:
- Findings after rebuild (HIGH / CRITICAL):
- Debian packages patched: yes / no
- Decision: resolved / accepted risk
- Comment/PR:
