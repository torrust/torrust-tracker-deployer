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

- [ ] Check the latest Caddy release:
      <https://hub.docker.com/_/caddy> and <https://github.com/caddyserver/caddy/releases>
- [ ] Run Trivy against the latest tag:
      `trivy image --severity HIGH,CRITICAL caddy:LATEST_TAG`
- [ ] Compare results against the 2.10.2 baseline in
      `docs/security/docker/scans/caddy.md`
- [ ] **If CRITICALs are cleared (or HIGH count drops meaningfully)**: update
      `templates/docker-compose/docker-compose.yml.tera` and the CI scan matrix;
      update the scan doc; post results comment; close #432
- [ ] **If CRITICALs remain**: post comment documenting which CVEs remain and why
      they cannot be fixed (upstream binary); add revisit note to #432; leave open

## Outcome

<!-- Fill in after doing the work -->

- Date:
- Latest Caddy tag tested:
- Findings (HIGH / CRITICAL):
- Decision: upgrade / accept risk / leave open
- Comment/PR:
