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

- [x] Check the latest Prometheus release:
      <https://hub.docker.com/r/prom/prometheus/tags>
- [x] Run Trivy against candidate newer tags:
      `trivy image --severity HIGH,CRITICAL prom/prometheus:LATEST_TAG`
- [x] Compare results against the v3.5.1 baseline in
      `docs/security/docker/scans/prometheus.md`
- [x] **If CRITICALs are cleared**: update `src/domain/prometheus/config.rs` and
      the CI scan matrix; update the scan doc; post results comment; close #433
- [ ] **If CRITICALs remain**: post comment documenting which CVEs remain and why
      they cannot be fixed (upstream binary); add revisit note to #433; leave open

## Outcome

- Date: 2026-04-14
- Latest Prometheus tag tested: `v3.11.2` (released 2026-04-13)
- Decision: **upgrade to `prom/prometheus:v3.11.2`** — all CRITICALs eliminated
- Action: updated `src/domain/prometheus/config.rs`; updated scan doc; updated CI matrix comment
- PR: opened against `main` on branch `433-prometheus-cves`

### Scan details — `prom/prometheus:v3.11.2` (Trivy, 2026-04-14)

**Version comparison:**

| Version   | HIGH | CRITICAL |
| --------- | ---- | -------- |
| `v3.5.0`  | 16   | 4        |
| `v3.5.1`  | 6    | 2        |
| `v3.11.2` | 4    | 0 ✅     |

**Target breakdown (`v3.11.2`):**

| Target           | HIGH | CRITICAL |
| ---------------- | ---- | -------- |
| `bin/prometheus` | 3    | 0        |
| `bin/promtool`   | 1    | 0        |

No OS layer — pure Go binaries, no Alpine/Debian base.

**Remaining CVEs (all HIGH, no remote attack path):**

| CVE            | Library          | Installed | Fixed In | Notes                                     |
| -------------- | ---------------- | --------- | -------- | ----------------------------------------- |
| CVE-2026-32285 | buger/jsonparser | v1.1.1    | 1.1.2    | DoS via malformed JSON; internal use only |
| CVE-2026-34040 | moby/docker      | v28.5.2   | 29.3.1   | Auth bypass; Docker-client code path      |
| CVE-2026-39883 | otel/sdk         | v1.42.0   | 1.43.0   | Local PATH hijack; no remote path         |

**Overall risk**: All 4 remaining findings are local-only. No remote attack path.
Upgrade to v3.11.2 is the recommended action and was applied.
