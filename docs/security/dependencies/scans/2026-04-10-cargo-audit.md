<!-- cspell:ignore RUSTSEC webpki pemfile -->

# Cargo Audit Security Scan - 2026-04-10

## Scan Metadata

- Date: 2026-04-10
- Tool: `cargo-audit`
- Workspace: `torrust-tracker-deployer`
- Command: `cargo audit`

## Baseline (Before Remediation)

Initial scan found 4 vulnerabilities and 1 warning:

1. `RUSTSEC-2026-0066` - `astral-tokio-tar 0.5.6`
1. `RUSTSEC-2026-0007` - `bytes 1.11.0`
1. `RUSTSEC-2026-0049` - `rustls-webpki 0.103.8`
1. `RUSTSEC-2026-0009` - `time 0.3.44`
1. `RUSTSEC-2025-0134` - `rustls-pemfile 2.2.0` (unmaintained warning)

Baseline output excerpt:

```text
error: 4 vulnerabilities found!
warning: 1 allowed warning found
```

## Remediation Actions

Applied updates:

1. Upgraded `testcontainers` in workspace root from `0.26` to `0.27`.
1. Upgraded `testcontainers` in `packages/dependency-installer` dev-dependencies from `0.25` to `0.27`.
1. Refreshed lockfile with `cargo update`.

These updates pulled patched transitive dependencies, including:

- `bytes 1.11.1`
- `time 0.3.47`
- `rustls-webpki 0.103.10`

## Verification (After Remediation)

Command rerun:

```bash
cargo audit
```

Result:

- Exit code: `0`
- No vulnerabilities reported for current lockfile.

Output excerpt:

```text
Fetching advisory database from `https://github.com/RustSec/advisory-db.git`
Loaded 1042 security advisories
Scanning Cargo.lock for vulnerabilities (380 crate dependencies)
```

## Follow-up Issues

No follow-up issue was required for this scan because all reported vulnerabilities were resolved through dependency updates.

## Related

- Main task: <https://github.com/torrust/torrust-tracker-deployer/issues/439>
- Workflow: `.github/workflows/cargo-security-audit.yml`
- Dependency report index: `docs/security/dependencies/README.md`
