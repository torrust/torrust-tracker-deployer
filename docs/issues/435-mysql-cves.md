# Issue #435: MySQL CVEs in mysql:8.4

**GitHub**: <https://github.com/torrust/torrust-tracker-deployer/issues/435>
**Image**: `mysql:8.4` (floating tag, resolved to `8.4.8` at time of last scan)

---

## Context

Current findings for `mysql:8.4`: **7 HIGH, 1 CRITICAL**.

Findings are in helper components (`gosu` and Python packages), not MySQL server
core. Investigation during PR #436 found that pinning to specific minor tags
(8.4.1–9.1) results in 98–100 HIGH — the floating `mysql:8.4` tag is already
the best available option.

## Decision

**Re-scan to check if the floating tag now resolves to a newer patch, then decide**:

- If the floating tag now resolves to a patch where `gosu`/Python CVEs are fixed:
  document the improvement. No code change needed (it's a floating tag).
- If still no practical fix: post comment confirming accepted risk and close #435

## Steps

- [x] Pull and scan the current floating tag:
      `docker pull mysql:8.4 && trivy image --severity HIGH,CRITICAL mysql:8.4`
- [x] Check which patch the floating tag currently resolves to:
      `docker inspect mysql:8.4 | grep -i version`
- [x] Compare results against the 8.4.8 baseline in
      `docs/security/docker/scans/mysql.md`
- [x] Check if `mysql:9.x` is now a viable option for the deployer (compatibility,
      LTS status):
      <https://hub.docker.com/_/mysql>
- [ ] **If CVE count has dropped**: update the scan doc; post comment; close #435
- [x] **If still 7 HIGH / 1 CRITICAL with no viable upgrade path**: post comment
      documenting accepted risk (helper components, not MySQL core); close #435

## Outcome

- Date: Apr 15, 2026
- Floating tag resolves to: `8.4.8` (unchanged from Apr 8 baseline)
- Previous findings (Apr 8, HIGH / CRITICAL): 7 HIGH / 1 CRITICAL
- Current findings (Apr 15, HIGH / CRITICAL): 9 HIGH / 1 CRITICAL (Trivy DB update; same image digest)
- mysql:9.6 (latest Innovation Release): identical CVE profile — 9 HIGH / 1 CRITICAL
- Decision: **accepted risk** — all CVEs in `gosu` helper binary and MySQL Shell Python tools, not MySQL Server core. No viable upgrade path. Requires MySQL upstream to ship updated `gosu` on Go ≥ 1.24.13.
