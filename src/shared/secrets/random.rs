//! Cryptographically random password generation

use rand::seq::IndexedRandom as _;
use rand::seq::SliceRandom as _;
use rand::Rng as _;

use super::password::Password;

const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const DIGIT: &[u8] = b"0123456789";
// Note: `{`, `}`, `<`, `>` are shell redirection/expansion characters.
// They are safe inside Docker `.env` quoted values but may need escaping
// if the password is ever interpolated in a raw shell context.
const SYMBOL: &[u8] = b"!@#$%^&*()-_=+[]{}<>?";

fn full_charset() -> Vec<u8> {
    [LOWER, UPPER, DIGIT, SYMBOL].concat()
}

/// Generate a cryptographically secure MySQL-compatible password.
///
/// Design rationale:
/// - `rand::rng()`: thread-local CSPRNG seeded from the OS and periodically
///   reseeded — suitable for secrets in rand 0.9 (direct `OsRng` no longer
///   implements the high-level `Rng` trait required by `choose`/`shuffle`)
/// - `choose`: avoids modulo bias — uniform distribution
/// - Explicit class inclusion: satisfies `MySQL` `validate_password` MEDIUM policy
/// - Shuffle: removes structural bias from fixed positions
///
/// The generated password is 32 characters long and always contains at least
/// one lowercase letter, one uppercase letter, one digit, and one symbol.
///
/// # Panics
///
/// Panics if any character set constant is empty, which cannot happen in practice
/// as they are defined as non-empty byte string literals.
#[must_use]
pub fn generate_random_password() -> Password {
    let mut rng = rand::rng();

    // Ensure required character classes (MySQL policy compliance)
    let mut password: Vec<u8> = vec![
        *LOWER
            .choose(&mut rng)
            .expect("LOWER charset is non-empty; selection must succeed"),
        *UPPER
            .choose(&mut rng)
            .expect("UPPER charset is non-empty; selection must succeed"),
        *DIGIT
            .choose(&mut rng)
            .expect("DIGIT charset is non-empty; selection must succeed"),
        *SYMBOL
            .choose(&mut rng)
            .expect("SYMBOL charset is non-empty; selection must succeed"),
    ];

    // Fill remaining characters with maximum entropy
    let charset = full_charset();
    for _ in password.len()..32 {
        let idx = rng.random_range(0..charset.len());
        password.push(charset[idx]);
    }

    // Remove positional bias
    password.shuffle(&mut rng);

    // Safe: charset only contains valid ASCII bytes
    Password::new(
        String::from_utf8(password)
            .expect("Generated password contains only valid ASCII; UTF-8 conversion must succeed"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_generate_password_satisfying_mysql_medium_policy() {
        for _ in 0..100 {
            let pwd = generate_random_password();
            let s = pwd.expose_secret();

            assert_eq!(s.len(), 32, "password must be 32 characters");
            assert!(s.chars().any(char::is_uppercase), "must contain uppercase");
            assert!(s.chars().any(char::is_lowercase), "must contain lowercase");
            assert!(s.chars().any(|c| c.is_ascii_digit()), "must contain digit");
            assert!(
                s.chars().any(|c| "!@#$%^&*()-_=+[]{}<>?".contains(c)),
                "must contain symbol"
            );
            assert!(s.is_ascii(), "must be ASCII (safe in .env files and shell)");
        }
    }

    #[test]
    fn it_should_generate_unique_passwords() {
        let a = generate_random_password();
        let b = generate_random_password();
        assert_ne!(
            a.expose_secret(),
            b.expose_secret(),
            "two consecutive calls must not produce the same password"
        );
    }
}
