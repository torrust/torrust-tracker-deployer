---
name: install-system-dependencies
description: Install the system dependencies required by the deployer (OpenTofu, Ansible, LXD, cargo-machete) using the built-in dependency-installer. This is the setup/installer step before using the deployer for the first time. Triggers on "install dependencies", "install system dependencies", "setup deployer", "first time setup", "missing dependency", "install opentofu", "install ansible", "install lxd", or "deployer setup".
metadata:
  author: torrust
  version: "1.0"
---

# Install System Dependencies

Use the built-in `dependency-installer` package to install all tools required to run the deployer.

## Recommended Setup Workflow

```bash
# 1. Check what is already installed
cargo run -p torrust-dependency-installer --bin dependency-installer check

# 2. Install everything missing in one command
cargo run -p torrust-dependency-installer --bin dependency-installer install

# 3. Verify all dependencies are now present
cargo run -p torrust-dependency-installer --bin dependency-installer check
```

## Commands

```bash
# Install all missing dependencies
cargo run -p torrust-dependency-installer --bin dependency-installer install

# Install a specific dependency
cargo run -p torrust-dependency-installer --bin dependency-installer install --dependency opentofu

# Install with verbose output (shows download/install steps)
cargo run -p torrust-dependency-installer --bin dependency-installer install --verbose
```

## Dependencies Installed

| Dependency      | Purpose                                   |
| --------------- | ----------------------------------------- |
| `opentofu`      | Infrastructure provisioning (`provision`) |
| `ansible`       | VM configuration (`configure`, `release`) |
| `lxd`           | Local VM provider (LXD environments)      |
| `cargo-machete` | Unused dependency detection (linter / CI) |

## Exit Codes

- `0` — All dependencies installed successfully
- `1` — One or more installations failed
- `2` — Invalid arguments
- `3` — Internal error

## Notes

- The `install` subcommand requires **system package manager access** (may prompt for `sudo`)
- Internet access is required to download installers (OpenTofu fetches from `opentofu.org`)
- After installation, re-run `check` to confirm everything is ready
- In GitHub Copilot agent environments, outbound network access to `opentofu.org` must be allowlisted — see `docs/contributing/copilot-agent/firewall.md`
