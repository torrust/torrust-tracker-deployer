# Issue #434: Grafana CVEs after upgrade to 12.4.2

**GitHub**: <https://github.com/torrust/torrust-tracker-deployer/issues/434>
**Image**: `grafana/grafana:12.4.2`
**Default set in**: `src/domain/grafana/config.rs`

---

## Context

After PR #436 upgraded Grafana from `12.3.1` to `12.4.2`:

| Version  | HIGH | CRITICAL |
| -------- | ---- | -------- |
| `12.3.1` | 18   | 6        |
| `12.4.2` | 4    | 0        |

CRITICALs are fully cleared. 4 HIGH remain in upstream binary dependencies.

## Decision

**Re-scan with latest Grafana tag, then decide**:

- If a newer tag clears remaining HIGH: upgrade, update scan doc, close #434
- If not: post comment with scan results confirming no CRITICALs, document accepted
  risk, close #434

## Steps

- [ ] Check the latest Grafana release:
      <https://hub.docker.com/r/grafana/grafana/tags>
- [ ] Run Trivy against the latest tag:
      `trivy image --severity HIGH,CRITICAL grafana/grafana:LATEST_TAG`
- [ ] Compare results against the 12.4.2 baseline in
      `docs/security/docker/scans/grafana.md`
- [ ] **If a newer tag reduces HIGH count**: update `src/domain/grafana/config.rs`
      and the CI scan matrix; update the scan doc; post results comment; close #434
- [ ] **If no improvement**: post comment with current scan output confirming
      no CRITICALs and document accepted risk for remaining HIGH; close #434

## Outcome

<!-- Fill in after doing the work -->

- Date:
- Latest Grafana tag tested:
- Findings (HIGH / CRITICAL):
- Decision: upgrade / accept risk
- Comment/PR:
