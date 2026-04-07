//! SSH private key inspection utilities
//!
//! This module provides best-effort heuristics for inspecting SSH private key
//! files without requiring external tools. The primary entry point is
//! [`is_passphrase_protected`], which is used during `create environment` to
//! emit an early warning when a passphrase-protected key is detected.
//!
//! See ADR: `docs/decisions/ssh-key-passphrase-detection.md`

use std::path::Path;

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;

/// Returns `true` if the private key at `path` appears to be passphrase-protected.
///
/// This is a best-effort heuristic used to emit an early warning during
/// `create environment`. It is not a security check:
/// - A **false negative** (encrypted key not detected) is acceptable — the warning
///   is advisory and the user is not blocked.
/// - A **false positive** (unencrypted key flagged) must be avoided — it would
///   confuse users with a spurious warning.
///
/// Returns `false` on any I/O or parse error (file not found, unrecognized format).
///
/// See ADR: `docs/decisions/ssh-key-passphrase-detection.md`
#[must_use]
pub fn is_passphrase_protected(path: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };

    // Legacy PEM formats declare encryption in the header line.
    if content.contains("BEGIN ENCRYPTED PRIVATE KEY") || content.contains("Proc-Type: 4,ENCRYPTED")
    {
        return true;
    }

    // OpenSSH format: encryption info is embedded in the binary body, not the header.
    if content.contains("BEGIN OPENSSH PRIVATE KEY") {
        return is_openssh_key_passphrase_protected(&content);
    }

    false
}

/// Checks whether an OpenSSH-format PEM body uses the bcrypt KDF.
///
/// OpenSSH private key binary layout (after base64-decoding the body):
///   `"openssh-key-v1\0"` (16-byte magic)
///   string `cipher_name`
///   string `kdf_name`      ← `"bcrypt"` when passphrase-protected, `"none"` otherwise
///   …
///
/// "string" is a uint32 length prefix followed by that many bytes.
///
/// Rather than fully parsing the structure, we scan the first 100 decoded bytes
/// for the literal byte sequence `b"bcrypt"`.
fn is_openssh_key_passphrase_protected(pem: &str) -> bool {
    let body: String = pem.lines().filter(|l| !l.starts_with("-----")).collect();

    let Ok(decoded) = STANDARD.decode(body.as_bytes()) else {
        return false;
    };

    let scan_len = decoded.len().min(100);
    decoded[..scan_len].windows(6).any(|w| w == b"bcrypt")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn project_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn it_should_return_false_when_key_is_unencrypted() {
        // Arrange
        let key_path = project_root().join("fixtures/testing_rsa");

        // Act
        let result = is_passphrase_protected(&key_path);

        // Assert
        assert!(
            !result,
            "Unencrypted key should not be detected as passphrase-protected"
        );
    }

    #[test]
    fn it_should_return_true_when_key_is_passphrase_protected() {
        // Arrange
        let key_path = project_root().join("fixtures/testing_ed25519_encrypted");

        // Act
        let result = is_passphrase_protected(&key_path);

        // Assert
        assert!(result, "Passphrase-protected key should be detected");
    }

    #[test]
    fn it_should_return_false_when_key_file_does_not_exist() {
        // Arrange
        let key_path = PathBuf::from("/nonexistent/path/to/key");

        // Act
        let result = is_passphrase_protected(&key_path);

        // Assert
        assert!(
            !result,
            "Missing file should return false (no spurious warning)"
        );
    }

    #[test]
    fn it_should_return_true_when_legacy_pem_header_contains_encrypted() {
        // Arrange: write a minimal legacy-format PKCS#8 encrypted PEM to a temp file
        let dir = tempfile::TempDir::new().unwrap();
        let key_path = dir.path().join("key.pem");
        std::fs::write(
            &key_path,
            // cspell:disable-next-line
            "-----BEGIN ENCRYPTED PRIVATE KEY-----\nZmFrZWtleQ==\n-----END ENCRYPTED PRIVATE KEY-----\n",
        )
        .unwrap();

        // Act
        let result = is_passphrase_protected(&key_path);

        // Assert
        assert!(result);
    }
}
