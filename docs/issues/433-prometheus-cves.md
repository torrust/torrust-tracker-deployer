# Issue #433: Prometheus CVEs after upgrade to v3.5.1

**GitHub**: <https://github.com/torrust/torrust-tracker-deployer/issues/433>
**Image**: `prom/prometheus:v3.5.1`
**Default set in**: `src/domain/prometheus/config.rs`

---

## Context

After PR #436 upgraded Prometheus from `v3.5.0` to `v3.5.1`:

| Version  | HIGH | CRITICAL |
| -------- | ---- | -------- |
| `v3.5.0` | 16   | 4        |
| `v3.5.1` | 6    | 4        |

4 CRITICAL remain in upstream binary dependencies.

## Decision

**Re-scan with latest Prometheus tag, then decide**:

- If a newer tag clears CRITICALs: upgrade, update scan doc, close #433
- If not: post comment with scan results, document accepted risk, leave open with
  revisit note

## Steps

- [ ] Check the latest Prometheus release:
      <https://hub.docker.com/r/prom/prometheus/tags>
- [ ] Run Trivy against candidate newer tags:
      `trivy image --severity HIGH,CRITICAL prom/prometheus:LATEST_TAG`
- [ ] Compare results against the v3.5.1 baseline in
      `docs/security/docker/scans/prometheus.md`
- [ ] **If CRITICALs are cleared**: update `src/domain/prometheus/config.rs` and
      the CI scan matrix; update the scan doc; post results comment; close #433
- [ ] **If CRITICALs remain**: post comment documenting which CVEs remain and why
      they cannot be fixed (upstream binary); add revisit note to #433; leave open

## Outcome

<!-- Fill in after doing the work -->

- Date:
- Latest Prometheus tag tested:
- Findings (HIGH / CRITICAL):
- Decision: upgrade / accept risk / leave open
- Comment/PR:
