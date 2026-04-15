# Dependency Security Reports

This directory tracks Rust dependency security scans for the deployer workspace.

## Current Status

- Last scan: 2026-04-10
- Tool: `cargo-audit`
- Status: no known RustSec vulnerabilities in `Cargo.lock`
- Latest report: [scans/2026-04-10-cargo-audit.md](scans/2026-04-10-cargo-audit.md)

## Scanning Standard

- Run command: `cargo audit`
- Record date, scanner output summary, and remediation actions.
- If findings remain and cannot be fixed quickly, open a follow-up GitHub issue and link it in the report.

## Related Automation

- Workflow: `.github/workflows/cargo-security-audit.yml`
- RustSec action: <https://github.com/rustsec/audit-check>
