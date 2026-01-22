# Command Outputs Reference

This folder contains captured CLI outputs from actual deployer command executions. These serve as:

- **Reference examples** for documentation and issues
- **AI agent context** for understanding expected command behavior
- **Regression baseline** for detecting output changes

## Structure

- `environment-configs/` - Environment configuration files used to generate outputs
- Per-command output files (e.g., `create.md`, `provision.md`, etc.)

## Regenerating Outputs

Outputs can be regenerated manually by running the deployer commands and capturing the output. In the future, this may be automated.

## Current Sessions

| Session                   | Provider | Domain   | TLS | Date       | Description                            |
| ------------------------- | -------- | -------- | --- | ---------- | -------------------------------------- |
| `lxd-local-example`       | LXD      | `.local` | No  | 2026-01-22 | Basic LXD deployment without TLS proxy |
| `lxd-local-https-example` | LXD      | `.local` | Yes | 2026-01-22 | LXD deployment with Caddy TLS proxy    |
