---
name: check-system-dependencies
description: Check whether the system has all dependencies required by the deployer (OpenTofu, Ansible, LXD, cargo-machete). Use before running provisioning, E2E tests, or any infrastructure command. Triggers on "check dependencies", "check system dependencies", "are dependencies installed", "verify dependencies", "dependency check", or "missing tools".
metadata:
  author: torrust
  version: "1.0"
---

# Check System Dependencies

Use the built-in `dependency-installer` package to verify all required tools are installed.

## Commands

```bash
# Check all dependencies
cargo run -p torrust-dependency-installer --bin dependency-installer check

# Check a specific dependency
cargo run -p torrust-dependency-installer --bin dependency-installer check --dependency opentofu

# List all dependencies with status
cargo run -p torrust-dependency-installer --bin dependency-installer list

# Install all missing dependencies
cargo run -p torrust-dependency-installer --bin dependency-installer install
```

## Required Dependencies

| Dependency      | Purpose                                   |
| --------------- | ----------------------------------------- |
| `opentofu`      | Infrastructure provisioning (`provision`) |
| `ansible`       | VM configuration (`configure`, `release`) |
| `lxd`           | Local VM provider (LXD environments)      |
| `cargo-machete` | Unused dependency detection (linter / CI) |

## Exit Codes

- `0` — All checked dependencies are installed
- Non-zero — One or more dependencies are missing

## Interpreting Output

All output goes to stderr via structured logging. Look for the summary line:

```text
INFO ... All dependencies are installed
```

Or a missing dependency error:

```text
ERROR ... dependency is not installed dependency="opentofu"
```

## Notes

- Run this before any `provision`, `configure`, `release`, or E2E test command
- In CI/CD pipelines use `--log-level off` to suppress output and rely on exit code only:

  ```bash
  cargo run -p torrust-dependency-installer --bin dependency-installer check --log-level off
  ```

- To install missing tools automatically, use the `install` subcommand (requires system package manager access)
