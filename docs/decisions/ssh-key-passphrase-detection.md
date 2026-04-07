# Decision: SSH Key Passphrase Detection via Byte Inspection

## Status

✅ Accepted

## Date

2026-04-07

## Context

When a user configures a passphrase-protected SSH private key in `ssh_credentials`,
the deployer fails silently during the `provision` step with a misleading
`Permission denied (publickey,password)` error. The root cause — that the key is
encrypted and cannot be decrypted without a passphrase in an unattended environment —
is never surfaced.

To give users an early, actionable warning, the `create environment` command needs to
detect whether the configured private key is passphrase-protected before it runs the
deployment workflow. Two approaches were considered.

See also: [Issue #411](../../docs/issues/411-bug-ssh-key-passphrase-breaks-automated-deployment.md)

## Decision

Detect passphrase protection by reading and inspecting the raw bytes of the key file
(**byte inspection**), implemented as a pure-Rust free function
`is_passphrase_protected(path: &Path) -> bool` in
`src/adapters/ssh/credentials.rs`.

Detection rules:

- **Legacy PEM format** (`PKCS#8` / traditional): the header line contains
  `BEGIN ENCRYPTED PRIVATE KEY` or the `Proc-Type: 4,ENCRYPTED` header is present.
- **OpenSSH format** (`-----BEGIN OPENSSH PRIVATE KEY-----`): the binary body
  (base64-decoded) contains the byte sequence `bcrypt` within the first 100 bytes.
  OpenSSH uses `bcrypt` as the KDF name when a passphrase is set; the KDF name is
  `none` for unencrypted keys.

The check is **best-effort**:

- A false negative (encrypted key not detected) is acceptable — the warning is advisory.
- A false positive (unencrypted key flagged) must be avoided — it would confuse users.
- Any I/O or parse error returns `false` (no spurious warning).

## Consequences

**Positive**:

- Pure Rust, zero new dependencies. No external process is spawned just for detection.
- Fast: reads only the first ~150 bytes of the file (header + base64 start).
- Works in any environment, including minimal Docker images where `ssh-keygen` may not
  be present.
- Handles both common key formats (legacy PEM, OpenSSH).

**Negative / Risks**:

- False-negative risk for exotic or future key formats (e.g., PKCS#1 passphrase-
  protected RSA keys with `Proc-Type` elsewhere in the file). Acceptable per spec.
- Requires maintaining a small inline base64 decoder (~25 lines) because no base64
  crate is in the dependency list. The decoder is minimal but covers the standard
  alphabet only; malformed base64 returns `false` rather than an error.

## Alternatives Considered

### `ssh-keygen -y` probe

Spawn `ssh-keygen -y -f <key> < /dev/null`: this exits non-zero when the key is
passphrase-protected, allowing detection via the exit code.

**Rejected because**:

- Requires `ssh-keygen` to be present at runtime. Docker images used in CI/CD may not
  include it, making the check environment-dependent.
- Spawns an external process with I/O redirection just for an advisory check — this
  adds latency and error-handling complexity (exit codes must distinguish "encrypted"
  from "file not found" or "unsupported format").
- The byte-inspection approach is faster, self-contained, and sufficient for the
  best-effort goal.

## Related Decisions

- [Validated Deserialization for Domain Types](./validated-deserialization-for-domain-types.md)

## References

- [Issue #411 spec](../issues/411-bug-ssh-key-passphrase-breaks-automated-deployment.md)
- [OpenSSH key format](https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.key)
- Implementation: `src/adapters/ssh/credentials.rs`
