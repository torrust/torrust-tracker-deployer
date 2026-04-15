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

- [x] Rebuild the image from scratch:
      `docker build --no-cache -t torrust/tracker-backup:local docker/backup/`
- [x] Re-scan: `trivy image --severity HIGH,CRITICAL torrust/tracker-backup:local`
- [x] Compare against the pass-1 baseline in
      `docs/security/docker/scans/torrust-tracker-backup.md`
- [x] For each remaining CVE, check fix availability:
      <https://security-tracker.debian.org/tracker/>
- [x] Update `docs/security/docker/scans/torrust-tracker-backup.md` with the new
      scan results
- [ ] **If HIGH count dropped**: post comment with before/after results; close #431
- [x] **If no change**: post comment documenting that Debian upstream has not yet
      patched these CVEs with a revisit note; close #431

## Outcome

- Date: Apr 15, 2026
- Findings after rebuild (HIGH / CRITICAL): 6 HIGH / 0 CRITICAL (unchanged)
- CVEs: CVE-2025-69720 (ncurses `infocmp`) and CVE-2026-29111 (systemd IPC)
- Debian packages patched: no — both CVEs are `<no-dsa>` minor issues; fixes only in forky/sid
- Decision: **accepted risk** — neither CVE is reachable in our container's runtime (no `infocmp` call, no systemd PID 1)
- Comment/PR: PR #457, comment on #431
