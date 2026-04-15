# Issue #432: Caddy CVEs after upgrade to 2.10.2

**GitHub**: <https://github.com/torrust/torrust-tracker-deployer/issues/432>
**Image**: `caddy:2.10.2`
**Template**: `templates/docker-compose/docker-compose.yml.tera`

---

## Context

After PR #436 upgraded Caddy from `2.10` to `2.10.2`:

| Version  | HIGH | CRITICAL |
| -------- | ---- | -------- |
| `2.10`   | 18   | 6        |
| `2.10.2` | 14   | 4        |

4 CRITICAL remain in upstream Caddy binary dependencies.

## Decision

**Re-scan with latest Caddy tag, then decide**:

- If a newer tag clears CRITICALs: upgrade, update scan doc, close #432
- If not: post comment with scan results, document accepted risk, leave open with
  revisit note

## Steps

- [x] Check the latest Caddy release:
      <https://hub.docker.com/_/caddy> and <https://github.com/caddyserver/caddy/releases>
- [x] Run Trivy against the latest tag:
      `trivy image --severity HIGH,CRITICAL caddy:LATEST_TAG`
- [x] Compare results against the 2.10.2 baseline in
      `docs/security/docker/scans/caddy.md`
- [x] **If CRITICALs are cleared (or HIGH count drops meaningfully)**: update
      `templates/docker-compose/docker-compose.yml.tera` and the CI scan matrix;
      update the scan doc; post results comment; close #432
- [ ] **If CRITICALs remain**: post comment documenting which CVEs remain and why
      they cannot be fixed (upstream binary); add revisit note to #432; leave open

## Outcome

- Date: 2026-04-15
- Latest Caddy tag tested: `2.11.2` (released 2026-04-14)
- Decision: **upgrade to `caddy:2.11.2`** — HIGH count dropped meaningfully (14→10), CRITICAL halved (4→2)
- Action: updated `templates/docker-compose/docker-compose.yml.tera` and CI scan matrix
- Issue: **left open** — 2 CRITICAL CVEs remain in upstream binary dependencies
- PR: opened against `main` on branch `432-caddy-cves`

### Scan details — `caddy:2.11.2` (Trivy v0.69.3, 2026-04-15)

**Version comparison:**

| Version  | HIGH | CRITICAL |
| -------- | ---- | -------- |
| `2.10`   | 18   | 6        |
| `2.10.2` | 14   | 4        |
| `2.11.2` | 10   | 2        |

**Target breakdown:**

| Target         | HIGH | CRITICAL |
| -------------- | ---- | -------- |
| caddy (alpine) | 3    | 0        |
| usr/bin/caddy  | 7    | 2        |

**Remaining CRITICAL CVEs (upstream binary, cannot be fixed without Caddy release):**

| CVE            | Library                | Fix    | Notes                                                      |
| -------------- | ---------------------- | ------ | ---------------------------------------------------------- |
| CVE-2026-30836 | smallstep/certificates | 0.30.0 | Unauthenticated SCEP cert issuance                         |
| CVE-2026-33186 | google.golang.org/grpc | 1.79.3 | Authorization bypass via HTTP/2 path ⚠️ network-accessible |

**Revisit**: when Caddy ships updated grpc-go (≥1.79.3) and smallstep/certificates (≥0.30.0).
