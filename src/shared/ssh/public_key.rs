//! SSH public key representation and validation
//!
//! This module provides the `SshPublicKey` type for handling SSH public key values
//! with proper validation and serialization support.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;

/// Errors that can occur when working with SSH public keys
#[derive(Error, Debug, Clone)]
pub enum SshPublicKeyError {
    #[error("SSH public key cannot be empty")]
    Empty,

    #[error("SSH public key format is invalid: {0}")]
    InvalidFormat(String),
}

/// SSH public key representation using the newtype pattern
///
/// This type wraps a string containing a valid SSH public key and provides
/// validation to ensure the key follows basic SSH public key format requirements.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deployer_lib::shared::ssh::SshPublicKey;
///
/// let key_str = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQCw16sai+XVnawp/P/Q23kcXKekygZ6ALmQAyslREo6kbG8s5RScsmbQqOQEcIwnV2Vo88eeWVzX0N0H1dIczRa/ezijBEsGefthzmz9Ix/vM4lodzTPQFtW8c2eYw7ESy12/2x5//UQQ3mxawEWsz5Ri8XuyBEy/Xh7xH/KpoektaocIOt2/WdCe8CvZdMLd7AviGcTdHFWRiOVrmHM1Pd8znqeA3/1KQP/M4Ae5q21oPjchGjVfPkGh/e62Wt+Wo/2lT30AyMO7JHA1tB1W4xANRQkOd1Kb/TrDLXfg0PaHQ+Irmycjp/H4KkcdB06nzYawXMN5csd/5TWKwkb9/vofp6GQNP731U8+JR4cxRfD107KoHroDSJpG2Fanb2PVBkSXAiJl29YrtoP9vUtSIemQCD/aXFtTcpSv7Y16bdp7v+0adCEHwBmodm9GzLL808FpI2ZCzCi+Ae98P3z+yPCxbrnVAahU8AM2NSbrfyH1w2eb4hJ22oPjdd//tBYtkE1TZBw+i3n0vRn04s5BfPRwwj5GISxacTOZm/YWvoE4UU9axtFXOtMUniVKL3ycA+LEfK7C4velOKbluyL8fYYu4pUxHnYOOkYYeRoi2jf3oagbABOpznloPd93wYP3NoUpIdtMZW+iCF0NnZkVLC9lm1FbTcnmrfNzFtGVKCQ== testing@torrust-testing-infra";
/// let public_key = SshPublicKey::new(key_str).unwrap();
/// println!("{}", public_key.as_str());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SshPublicKey(String);

impl SshPublicKey {
    /// Creates a new `SshPublicKey` from a string
    ///
    /// # Arguments
    /// * `key` - The SSH public key string
    ///
    /// # Errors
    /// Returns an error if the key is empty or has an invalid format
    pub fn new<S: Into<String>>(key: S) -> Result<Self, SshPublicKeyError> {
        let key = key.into();

        if key.trim().is_empty() {
            return Err(SshPublicKeyError::Empty);
        }

        // Basic SSH public key format validation
        // SSH public keys typically start with the key type (ssh-rsa, ssh-ed25519, etc.)
        let trimmed = key.trim();
        if !Self::is_valid_format(trimmed) {
            return Err(SshPublicKeyError::InvalidFormat(
                "SSH public key must start with a valid key type (ssh-rsa, ssh-dss, ssh-ed25519, ssh-ed448, rsa-sha2-256, rsa-sha2-512, ecdsa-sha2-*, etc.)".to_string()
            ));
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Basic format validation for SSH public keys
    ///
    /// Checks if the key starts with a recognized SSH key type and has the basic structure
    /// Based on IANA SSH Parameters registry: <https://www.iana.org/assignments/ssh-parameters/ssh-parameters.xhtml#ssh-parameters-19>
    ///
    /// **Note for maintainers**: When new SSH key types are added to the IANA registry,
    /// update the `valid_prefixes` array below to include them. Always check the official
    /// IANA SSH Parameters document for the most current list of registered key types.
    fn is_valid_format(key: &str) -> bool {
        let valid_prefixes = [
            // Standard SSH key types
            "ssh-rsa",
            "ssh-dss",
            "ssh-ed25519",
            "ssh-ed448",
            // RSA with specific hash algorithms
            "rsa-sha2-256",
            "rsa-sha2-512",
            // ECDSA variants
            "ssh-ecdsa",
            "ecdsa-sha2-nistp256",
            "ecdsa-sha2-nistp384",
            "ecdsa-sha2-nistp521",
            // SPKI signatures
            "spki-sign-rsa",
            "spki-sign-dss",
            // PGP signatures
            "pgp-sign-rsa",
            "pgp-sign-dss",
            // X.509 certificate types
            "x509v3-ssh-dss",
            "x509v3-ssh-rsa",
            "x509v3-rsa2048-sha256",
            // Note: x509v3-ecdsa-sha2-* handled by prefix matching below
            // Note: ecdsa-sha2-* handled by prefix matching below
            // Null key for testing
            "null",
        ];

        // Check if the key starts with a valid prefix
        let has_valid_prefix = valid_prefixes.iter().any(|prefix| key.starts_with(prefix));

        if has_valid_prefix {
            // Basic structure check: should have at least 2 space-separated parts for most keys
            // Format: <type> <key-data> [comment]
            let parts: Vec<&str> = key.split_whitespace().collect();
            return parts.len() >= 2 || key.starts_with("null"); // null key might be standalone
        }

        // Check for wildcard patterns not covered by exact prefixes
        let wildcard_patterns = [
            "ecdsa-sha2-",        // Matches ecdsa-sha2-* variants
            "x509v3-ecdsa-sha2-", // Matches x509v3-ecdsa-sha2-* variants
        ];

        for pattern in &wildcard_patterns {
            if key.starts_with(pattern) {
                let parts: Vec<&str> = key.split_whitespace().collect();
                return parts.len() >= 2;
            }
        }

        false
    }
    /// Returns the SSH public key as a string slice
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the `SshPublicKey` and returns the inner string
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl FromStr for SshPublicKey {
    type Err = SshPublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl fmt::Display for SshPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<SshPublicKey> for String {
    fn from(key: SshPublicKey) -> String {
        key.0
    }
}

impl AsRef<str> for SshPublicKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_RSA_KEY: &str = "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAACAQCw16sai+XVnawp/P/Q23kcXKekygZ6ALmQAyslREo6kbG8s5RScsmbQqOQEcIwnV2Vo88eeWVzX0N0H1dIczRa/ezijBEsGefthzmz9Ix/vM4lodzTPQFtW8c2eYw7ESy12/2x5//UQQ3mxawEWsz5Ri8XuyBEy/Xh7xH/KpoektaocIOt2/WdCe8CvZdMLd7AviGcTdHFWRiOVrmHM1Pd8znqeA3/1KQP/M4Ae5q21oPjchGjVfPkGh/e62Wt+Wo/2lT30AyMO7JHA1tB1W4xANRQkOd1Kb/TrDLXfg0PaHQ+Irmycjp/H4KkcdB06nzYawXMN5csd/5TWKwkb9/vofp6GQNP731U8+JR4cxRfD107KoHroDSJpG2Fanb2PVBkSXAiJl29YrtoP9vUtSIemQCD/aXFtTcpSv7Y16bdp7v+0adCEHwBmodm9GzLL808FpI2ZCzCi+Ae98P3z+yPCxbrnVAahU8AM2NSbrfyH1w2eb4hJ22oPjdd//tBYtkE1TZBw+i3n0vRn04s5BfPRwwj5GISxacTOZm/YWvoE4UU9axtFXOtMUniVKL3ycA+LEfK7C4velOKbluyL8fYYu4pUxHnYOOkYYeRoi2jf3oagbABOpznloPd93wYP3NoUpIdtMZW+iCF0NnZkVLC9lm1FbTcnmrfNzFtGVKCQ== testing@torrust-testing-infra";
    const VALID_ED25519_KEY: &str = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIG4rT3vTt99Ox5kndS4HmgTrKBT8tOajsHpzHtRG testing@example.com";
    const VALID_RSA_SHA2_256_KEY: &str =
        "rsa-sha2-256 AAAAB3NzaC1yc2EAAAADAQABAAABAQC7vbqajnc testing@example.com";
    const VALID_ECDSA_KEY: &str =
        "ecdsa-sha2-nistp256 AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTY testing@example.com";

    #[test]
    fn it_should_create_ssh_public_key_with_valid_rsa_key() {
        let key = SshPublicKey::new(VALID_RSA_KEY).unwrap();
        assert_eq!(key.as_str(), VALID_RSA_KEY);
    }

    #[test]
    fn it_should_create_ssh_public_key_with_valid_ed25519_key() {
        let key = SshPublicKey::new(VALID_ED25519_KEY).unwrap();
        assert_eq!(key.as_str(), VALID_ED25519_KEY);
    }

    #[test]
    fn it_should_create_ssh_public_key_with_rsa_sha2_256_key() {
        let key = SshPublicKey::new(VALID_RSA_SHA2_256_KEY).unwrap();
        assert_eq!(key.as_str(), VALID_RSA_SHA2_256_KEY);
    }

    #[test]
    fn it_should_create_ssh_public_key_with_ecdsa_key() {
        let key = SshPublicKey::new(VALID_ECDSA_KEY).unwrap();
        assert_eq!(key.as_str(), VALID_ECDSA_KEY);
    }

    /// Parameterized test for all supported SSH key prefixes
    ///
    /// This test validates that all SSH key types defined in the IANA SSH Parameters registry
    /// are properly recognized by our validation logic. When new key types are added to the
    /// registry, add them to this test data to ensure they're supported.
    #[test]
    fn it_should_support_all_iana_registered_ssh_key_types() {
        // Test data: (prefix, description)
        let supported_prefixes = [
            // Standard SSH key types
            ("ssh-rsa", "RSA keys"),
            ("ssh-dss", "DSS/DSA keys"),
            ("ssh-ed25519", "Ed25519 keys"),
            ("ssh-ed448", "Ed448 keys"),
            // RSA with specific hash algorithms
            ("rsa-sha2-256", "RSA with SHA-256"),
            ("rsa-sha2-512", "RSA with SHA-512"),
            // ECDSA variants
            ("ssh-ecdsa", "ECDSA keys (generic)"),
            ("ecdsa-sha2-nistp256", "ECDSA P-256"),
            ("ecdsa-sha2-nistp384", "ECDSA P-384"),
            ("ecdsa-sha2-nistp521", "ECDSA P-521"),
            // SPKI signatures
            ("spki-sign-rsa", "SPKI RSA signatures"),
            ("spki-sign-dss", "SPKI DSS signatures"),
            // PGP signatures
            ("pgp-sign-rsa", "PGP RSA signatures"),
            ("pgp-sign-dss", "PGP DSS signatures"),
            // X.509 certificate types
            ("x509v3-ssh-dss", "X.509v3 DSS certificates"),
            ("x509v3-ssh-rsa", "X.509v3 RSA certificates"),
            ("x509v3-rsa2048-sha256", "X.509v3 RSA 2048 SHA-256"),
            // Null key for testing
            ("null", "Null key type"),
        ];

        for (prefix, description) in supported_prefixes {
            let test_key = if prefix == "null" {
                // Null key might be standalone
                prefix.to_string()
            } else {
                // Standard format: <type> <key-data> [comment]
                format!("{prefix} AAAAB3NzaC1example_key_data test@example.com")
            };

            let result = SshPublicKey::new(&test_key);
            assert!(
                result.is_ok(),
                "Failed to validate {description} with prefix '{prefix}': {test_key}"
            );

            let key = result.unwrap();
            assert_eq!(key.as_str(), test_key);
        }
    }

    /// Test wildcard ECDSA variants that use pattern matching
    #[test]
    fn it_should_support_wildcard_ecdsa_variants() {
        let wildcard_variants = [
            ("ecdsa-sha2-custom", "Custom ECDSA SHA-2 variant"),
            ("ecdsa-sha2-nistp192", "ECDSA P-192"),
            ("x509v3-ecdsa-sha2-nistp256", "X.509v3 ECDSA P-256"),
            ("x509v3-ecdsa-sha2-custom", "X.509v3 ECDSA custom variant"),
        ];

        for (prefix, description) in wildcard_variants {
            let test_key = format!("{prefix} AAAAB3NzaC1example_key_data test@example.com");
            let result = SshPublicKey::new(&test_key);
            assert!(
                result.is_ok(),
                "Failed to validate {description} with prefix '{prefix}': {test_key}"
            );

            let key = result.unwrap();
            assert_eq!(key.as_str(), test_key);
        }
    }

    #[test]
    fn it_should_support_ssh_dss_keys() {
        let dss_key = "ssh-dss AAAAB3NzaC1kc3MAAACBAIr9... test@example.com";
        let key = SshPublicKey::new(dss_key).unwrap();
        assert_eq!(key.as_str(), dss_key);
    }

    #[test]
    fn it_should_support_ssh_ed448_keys() {
        let ed448_key = "ssh-ed448 AAAAGnNzaC1lZDQ0OAAAANLamVx1... test@example.com";
        let key = SshPublicKey::new(ed448_key).unwrap();
        assert_eq!(key.as_str(), ed448_key);
    }

    #[test]
    fn it_should_support_rsa_sha2_512_keys() {
        let rsa_sha2_512_key =
            "rsa-sha2-512 AAAAB3NzaC1yc2EAAAADAQABAAABAQC7vbqajnc test@example.com";
        let key = SshPublicKey::new(rsa_sha2_512_key).unwrap();
        assert_eq!(key.as_str(), rsa_sha2_512_key);
    }

    #[test]
    fn it_should_support_x509v3_keys() {
        let x509_key = "x509v3-ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABAQC7vbqajnc test@example.com";
        let key = SshPublicKey::new(x509_key).unwrap();
        assert_eq!(key.as_str(), x509_key);
    }

    #[test]
    fn it_should_fail_with_empty_key() {
        let result = SshPublicKey::new("");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SshPublicKeyError::Empty));
    }

    #[test]
    fn it_should_fail_with_whitespace_only_key() {
        let result = SshPublicKey::new("   \n  \t  ");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SshPublicKeyError::Empty));
    }

    #[test]
    fn it_should_fail_with_invalid_format() {
        let result = SshPublicKey::new("invalid-key-format");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SshPublicKeyError::InvalidFormat(_)
        ));
    }

    #[test]
    fn it_should_fail_with_incomplete_key() {
        let result = SshPublicKey::new("ssh-rsa");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SshPublicKeyError::InvalidFormat(_)
        ));
    }

    #[test]
    fn it_should_trim_whitespace() {
        let key_with_whitespace = format!("  \n{VALID_RSA_KEY}\n  ");
        let key = SshPublicKey::new(key_with_whitespace).unwrap();
        assert_eq!(key.as_str(), VALID_RSA_KEY);
    }

    #[test]
    fn it_should_convert_from_str() {
        let key: SshPublicKey = VALID_RSA_KEY.parse().unwrap();
        assert_eq!(key.as_str(), VALID_RSA_KEY);
    }

    #[test]
    fn it_should_display_correctly() {
        let key = SshPublicKey::new(VALID_RSA_KEY).unwrap();
        assert_eq!(format!("{key}"), VALID_RSA_KEY);
    }

    #[test]
    fn it_should_convert_to_string() {
        let key = SshPublicKey::new(VALID_RSA_KEY).unwrap();
        let string_key: String = key.into();
        assert_eq!(string_key, VALID_RSA_KEY);
    }

    #[test]
    fn it_should_serialize_to_json() {
        let key = SshPublicKey::new(VALID_RSA_KEY).unwrap();
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(json, format!("\"{VALID_RSA_KEY}\""));
    }

    #[test]
    fn it_should_deserialize_from_json() {
        let json = format!("\"{VALID_RSA_KEY}\"");
        let key: SshPublicKey = serde_json::from_str(&json).unwrap();
        assert_eq!(key.as_str(), VALID_RSA_KEY);
    }

    #[test]
    fn it_should_be_equal_when_same_key() {
        let key1 = SshPublicKey::new(VALID_RSA_KEY).unwrap();
        let key2 = SshPublicKey::new(VALID_RSA_KEY).unwrap();
        assert_eq!(key1, key2);
    }

    #[test]
    fn it_should_work_as_reference() {
        let key = SshPublicKey::new(VALID_RSA_KEY).unwrap();
        let key_ref: &str = key.as_ref();
        assert_eq!(key_ref, VALID_RSA_KEY);
    }
}
