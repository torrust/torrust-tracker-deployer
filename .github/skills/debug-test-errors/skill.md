---
name: debug-test-errors
description: Guide for understanding expected errors and warnings in test output for this project. Covers SSH host key warnings (normal, expected, not failures), logging level meanings (ERROR vs WARN vs DEBUG), and when to be concerned vs when to ignore. Use when debugging E2E test output, seeing red or yellow warnings in tests, or investigating unexpected-looking log messages. Triggers on "test error", "test warning", "SSH warning", "E2E output", "debug test", "test failure", "known issue", "expected error", or "test output".
metadata:
  author: torrust
  version: "1.0"
---

# Debugging Test Errors

## Expected: SSH Host Key Warnings

This warning is **normal and expected** — not a failure:

```text
WARN SSH warning detected, operation: "ssh_warning", host_ip: 127.0.0.1,
Warning: Permanently added '[127.0.0.1]:32825' (ED25519) to the list of known hosts.
```

**Why it appears**: SSH connects to new hosts with `-o StrictHostKeyChecking=no` for automation.
SSH adds the host key to `known_hosts` and informs us — this is normal security behavior.

**What it means**: SSH is connecting successfully. This is working correctly.

## Log Level Guide

| Level   | Meaning                                         | Action required? |
| ------- | ----------------------------------------------- | ---------------- |
| `ERROR` | Actual failure requiring attention              | ✅ Yes           |
| `WARN`  | Expected warning (e.g., SSH host key additions) | ❌ Usually no    |
| `DEBUG` | Detailed execution info for troubleshooting     | ❌ No            |

## When to Actually Be Concerned

Contact the team or file an issue for:

- **Process failures**: Overall command or test returns non-zero exit code
- **Connection errors**: "Connection refused", "Host unreachable"
- **Permission errors**: Unexpected "permission denied"
- **Service failures**: Docker containers not starting
- **Data corruption**: Invalid configs, lost state
- **Unknown patterns**: Errors not listed in known-issues

## How to Check if a Test Actually Failed

The overall exit code matters, not individual WARN messages:

```bash
# Run E2E test — check exit code
cargo run --bin e2e-infrastructure-lifecycle-tests; echo "Exit: $?"

# Look for these indicators of real failure (not SSH warnings):
# - "FAILED:", "ERROR:", "panicked at"
# - Non-zero exit code
```

## Common False Alarms

| What you see                           | Is it a failure? |
| -------------------------------------- | ---------------- |
| SSH host key WARN in test logs         | ❌ Expected      |
| WARN-level log lines                   | ❌ Usually no    |
| "Permanently added ... to known_hosts" | ❌ Expected      |
| "Grafana not ready yet, retrying..."   | ❌ Expected      |
| Test prints red/orange text            | Check exit code  |

## Reference

Known issues: [`docs/contributing/known-issues.md`](../../docs/contributing/known-issues.md)
E2E testing: [`docs/e2e-testing/`](../../docs/e2e-testing/)
