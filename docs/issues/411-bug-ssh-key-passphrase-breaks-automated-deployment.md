# Bug: Passphrase-Protected SSH Key Silently Fails During Automated Deployment

**Issue**: #411
**Parent Epic**: None
**Related**: #405 - Deploy Hetzner Demo Tracker and Document the Process

## Overview

When a user configures a passphrase-protected SSH private key in `ssh_credentials`, the
deployer fails silently during the `provision` step with a misleading
`Permission denied (publickey,password)` error. The root cause — that the key is
encrypted and cannot be decrypted without a passphrase in an unattended environment —
is never surfaced to the user.

This bug was triggered during the Hetzner demo deployment (#405) where a fresh
deployment key was created with a passphrase for security. In an interactive terminal
session the OS SSH agent transparently decrypts the key, so the error only appears when
running the deployer in an automated context (Docker container, CI/CD pipeline) where no
agent is available.

There is no code fix required — passphrase-protected keys are a valid configuration for
some workflows (e.g. with SSH agent forwarding into the container). The two actions
needed are:

1. **Add an early warning** in `create environment` when the private key file is detected
   as passphrase-protected, so users can make an informed decision before reaching the
   `provision` step.
2. **Add documentation** covering SSH key requirements and the three supported workflows.

## Goals

- [ ] Detect passphrase-protected private keys during `create environment` and emit a
      user-visible warning (not an error — the choice belongs to the user)
- [ ] Add a documentation section to the user guide on SSH key handling, covering key
      requirements and all supported workflows
- [ ] Update the Hetzner provider guide to call out the passphrase requirement for
      Docker-based deployments

## Specifications

### Root Cause

SSH private keys can be stored in two formats:

- **Unencrypted**: the key material is in plaintext in the PEM file.
- **Encrypted (passphrase-protected)**: the key material is encrypted; decryption
  requires the passphrase, an SSH agent holding the unlocked key, or a TTY for
  interactive prompting.

The deployer invokes the system `ssh` binary for connectivity probes and remote
commands. When running inside a Docker container, there is no SSH agent socket and no
TTY. If the key file is encrypted, `ssh` cannot authenticate and every attempt returns
`Permission denied (publickey,password)`. This is indistinguishable from a wrong key
or an unconfigured `authorized_keys` file — the log output reveals nothing about the
passphrase being the cause.

An encrypted OpenSSH private key file contains `ENCRYPTED` in its PEM header:

```text
-----BEGIN OPENSSH PRIVATE KEY-----   ← unencrypted
-----BEGIN OPENSSH PRIVATE KEY-----   ← also unencrypted (need to read body)
```

The reliable detection approach is to read the first line of the PEM file:

- RSA/EC legacy PEM format: encrypted files contain `ENCRYPTED` in the header:
  `-----BEGIN ENCRYPTED PRIVATE KEY-----` or `Proc-Type: 4,ENCRYPTED`
- OpenSSH format: the body begins with `bcrypt` if passphrase-protected; the header
  alone is not sufficient — the first few bytes of the decoded body must be checked.

### Detection Approaches Considered

Two approaches were evaluated for detecting whether a key is passphrase-protected:

#### Option A — Byte inspection (chosen)

Read the raw bytes of the key file and check:

- For legacy PEM: `ENCRYPTED` appears in the header
- For OpenSSH format: after the base64-decoded body, the string `bcrypt` appears near
  the start (OpenSSH uses bcrypt KDF for passphrase derivation)

|     |                                                                                 |
| --- | ------------------------------------------------------------------------------- |
| ✅  | Pure Rust, no external tools required                                           |
| ✅  | Fast — reads only the first ~64 bytes of the file                               |
| ✅  | Works in any environment, including minimal Docker images                       |
| ✅  | No process spawning overhead                                                    |
| ❌  | Must handle two PEM variants (legacy `ENCRYPTED` header, OpenSSH `bcrypt` body) |
| ❌  | False-negative risk for exotic or future key formats (acceptable per spec)      |

#### Option B — `ssh-keygen -y` probe

Spawn `ssh-keygen -y -f <key> < /dev/null`: this command attempts to derive the public
key from the private key and exits non-zero with a passphrase prompt when the key is
encrypted.

|     |                                                                                                     |
| --- | --------------------------------------------------------------------------------------------------- |
| ✅  | Handles all key formats transparently — no format-specific parsing                                  |
| ✅  | Implementation is a single `std::process::Command` call                                             |
| ❌  | Requires `ssh-keygen` to be present in the runtime environment                                      |
| ❌  | Spawns an external process just for detection                                                       |
| ❌  | Must distinguish "encrypted" from "file not found" / "unsupported format" via exit codes and stderr |
| ❌  | Slower than reading a few bytes from a file                                                         |

**Chosen approach**: Option A (byte inspection). The deployer already targets Docker
containers where `ssh-keygen` may not be installed, and the detection is best-effort
(a missed warning is acceptable — a false positive is not). An ADR documents this
choice in full (see Implementation Plan).

The check only needs to be a best-effort heuristic — it is used to emit a warning, not
to block the user. A false negative (missing the warning) is acceptable; a false
positive (warning for an unencrypted key) would be confusing and should be avoided.

### Warning Behavior

The warning should be emitted during the `create environment` command, after the
configuration is loaded and the private key path is resolved but before the environment
state is persisted. It must:

- Be non-blocking — the environment is still created normally.
- Be clearly labelled as a warning, not an error.
- Explain the consequence (automated runs without an SSH agent will fail).
- Describe all three resolution options (see below).

Example warning text:

```text
⚠ Warning: SSH private key appears to be passphrase-protected.
  Key: /home/deployer/.ssh/torrust_tracker_deployer_ed25519

  Automated deployment (e.g. Docker, CI/CD) requires an SSH key that can be used
  without interactive input. A passphrase-protected key will cause the `provision`
  step to fail with "Permission denied" unless one of the following is arranged:

  Option 1 — Remove the passphrase (recommended for dedicated deployment keys):
    ssh-keygen -p -f /path/to/your/private_key

  Option 2 — Forward your SSH agent socket into the Docker container:
    docker run ... -v "$SSH_AUTH_SOCK:/tmp/ssh-agent.sock" \
                   -e SSH_AUTH_SOCK=/tmp/ssh-agent.sock ...

  Option 3 — Use a separate passphrase-free deployment key and configure it in
    ssh_credentials.private_key_path.

  You can continue now — the environment will be created. If you plan to run
  the deployer without an SSH agent, resolve this before running `provision`.
```

### Affected Modules and Types

#### Detection utility

A small free function (or method on `SshCredentials`) to check whether a key file
appears to be passphrase-protected. Location: `src/adapters/ssh/ssh/credentials.rs` or
a new `src/adapters/ssh/ssh/key_inspector.rs`.

The function signature could be:

```rust
/// Returns `true` if the private key at `path` appears to be passphrase-protected.
/// Returns `false` if the key is unencrypted or if the file cannot be read/parsed.
pub fn is_passphrase_protected(path: &Path) -> bool
```

This is best-effort: it returns `false` on any I/O or parse error (no key found,
unrecognized format) to avoid blocking normal flow with spurious warnings.

#### `create environment` handler

`src/presentation/cli/controllers/create/subcommands/environment/handler.rs`:

After configuration is loaded (the `LoadConfiguration` step), call the detection
function on `config.ssh_credentials.ssh_priv_key_path`. If it returns `true`, emit the
warning through `user_output` before proceeding to the `CreateEnvironment` step.

No changes are needed in the application or domain layers — this is a pure
presentation-layer concern.

### Documentation

#### New page: SSH Key Handling

Create `docs/user-guide/ssh-keys.md` covering:

- Why the deployer requires SSH keys (remote provisioning, configuration, release, run)
- Key requirements for unattended automation (no passphrase, or agent forwarding)
- The three workflows:
  1. Passphrase-free dedicated deployment key (recommended)
  2. SSH agent forwarding into Docker
  3. Direct (non-Docker) execution with an SSH agent running on the host
- How to generate a deployment key pair:

  ```bash
  ssh-keygen -t ed25519 -C "torrust-tracker-deployer" \
      -f ~/.ssh/torrust_tracker_deployer_ed25519
  # Leave passphrase empty for automated use
  ```

- How to remove an existing passphrase:

  ```bash
  ssh-keygen -p -f ~/.ssh/torrust_tracker_deployer_ed25519
  ```

- Security notes: dedicated deployment keys, key rotation after use, filesystem
  permissions (`0600`)
- Reference to the `ssh_credentials` fields in the environment config JSON schema

#### Update: Hetzner provider guide

`docs/user-guide/providers/hetzner.md` — add a "SSH Key Requirements" section or
callout box noting that Docker-based deployments require a passphrase-free key (or agent
forwarding) and linking to the new SSH keys page.

#### Update: `create environment` command docs

`docs/user-guide/commands/create.md` — mention that a warning is shown if the
configured private key appears to be passphrase-protected.

## Implementation Plan

### Phase 1: Detection and warning (code change)

- [ ] Implement `is_passphrase_protected(path: &Path) -> bool` in
      `src/adapters/ssh/ssh/credentials.rs` (or a new `key_inspector.rs` module)
  - Check for `ENCRYPTED` in PEM header (legacy format)
  - Check for `bcrypt` near the start of the decoded OpenSSH body
  - Return `false` on any I/O or parse error
- [ ] In the `create environment` handler
      (`handler.rs`): after `LoadConfiguration`, call the detection function and emit
      a warning via `user_output` if the key appears to be passphrase-protected
- [ ] Add unit test `it_detects_passphrase_protected_key` (using a test fixture key
      with and without passphrase if available, or by constructing the minimal PEM
      structure in the test)

### Phase 2: ADR

- [ ] Create `docs/decisions/XXX-ssh-key-passphrase-detection.md` documenting:
  - Why byte inspection was chosen over the `ssh-keygen -y` probe
  - Pros and cons of each approach
  - Consequences and limitations (best-effort, false-negative acceptable)
- [ ] Register the new ADR in `docs/decisions/README.md`

### Phase 3: Documentation

- [ ] Create `docs/user-guide/ssh-keys.md` covering all workflows and security notes
- [ ] Update `docs/user-guide/providers/hetzner.md` with an SSH key requirements note
- [ ] Update `docs/user-guide/commands/create.md` to mention the passphrase warning
- [ ] Update `docs/user-guide/README.md` to link to the new `ssh-keys.md` page

### Phase 4: Linting and pre-commit

- [ ] Run linters: `cargo run --bin linter all`
- [ ] Run pre-commit: `./scripts/pre-commit.sh`

## Acceptance Criteria

> **Note for Contributors**: These criteria define what the PR reviewer will check.
> Use this as your pre-review checklist before submitting the PR to minimize
> back-and-forth iterations.

**Quality Checks**:

- [ ] Pre-commit checks pass: `./scripts/pre-commit.sh`

**Task-Specific Criteria**:

- [ ] `create environment` emits a visible warning (not an error) when the configured
      private key file is passphrase-protected
- [ ] `create environment` still succeeds (environment is created) even when the warning
      is emitted — the user is not blocked
- [ ] `create environment` emits no warning when the key is unencrypted
- [ ] The warning message names all three resolution options (remove passphrase, agent
      forwarding, separate key)
- [ ] `docs/user-guide/ssh-keys.md` exists and covers key requirements, workflows, and
      security notes
- [ ] `docs/user-guide/providers/hetzner.md` references the SSH key requirements

## Related Documentation

- [docs/deployments/hetzner-demo-tracker/commands/provision/problems.md](../deployments/hetzner-demo-tracker/commands/provision/problems.md) — root cause analysis and resolution for the Hetzner deployment failure
- [src/adapters/ssh/ssh/credentials.rs](../../src/adapters/ssh/ssh/credentials.rs) — `SshCredentials` struct
- [src/presentation/cli/controllers/create/subcommands/environment/handler.rs](../../src/presentation/cli/controllers/create/subcommands/environment/handler.rs) — where the warning is added
- [docs/user-guide/providers/hetzner.md](../user-guide/providers/hetzner.md) — Hetzner provider guide
