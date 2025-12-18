# Secret Handling Guide

This guide explains how to properly handle sensitive data (secrets) in the codebase to prevent accidental exposure through logs, debug output, or error messages.

## 📋 Quick Reference

**Never use `String` for sensitive data.** Always use wrapper types from `src/shared/secrets/`:

- **API Tokens**: `ApiToken` (domain/infrastructure), `PlainApiToken` (DTO boundaries)
- **Passwords**: `Password` (domain/infrastructure), `PlainPassword` (DTO boundaries)

## 🔐 Why Secret Types?

### Security Risks with Plain Strings

Using plain `String` for secrets exposes them in multiple ways:

```rust
// ❌ WRONG - Secret exposed everywhere
pub struct HetznerConfig {
    pub api_token: String, // Visible in Debug, logs, error messages
}

let config = HetznerConfig {
    api_token: "hetzner_abc123def456".to_string(),
};

println!("{:?}", config);
// Output: HetznerConfig { api_token: "hetzner_abc123def456" }
//         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//         SECRET LEAKED!
```

### Protection with Secret Types

Secret types prevent exposure:

```rust
// ✅ CORRECT - Secret protected
use crate::shared::secrets::ApiToken;

pub struct HetznerConfig {
    pub api_token: ApiToken, // Redacted in Debug output
}

let config = HetznerConfig {
    api_token: ApiToken::from("hetzner_abc123def456"),
};

println!("{:?}", config);
// Output: HetznerConfig { api_token: Secret([REDACTED]) }
//                                    ^^^^^^^^^^^^^^^^^^^
//                                    SECRET PROTECTED!
```

## 🎯 Secret Types Reference

### ApiToken

Use for API authentication tokens (REST APIs, cloud providers, etc.).

**Common Fields**:

- `HetznerConfig.api_token` - Cloud provider API token
- HTTP API admin tokens
- Third-party service API keys

**Example**:

```rust
use crate::shared::secrets::ApiToken;
use secrecy::ExposeSecret;

// Creating
let token = ApiToken::from("my-api-token-123");

// Using (only when needed for actual API calls)
let token_str: &str = token.expose_secret();
```

### Password

Use for authentication passwords (database, SSH, services).

**Common Fields**:

- `MysqlConfig.password` - Database password
- SSH passwords
- Service account passwords

**Example**:

```rust
use crate::shared::secrets::Password;
use secrecy::ExposeSecret;

// Creating
let password = Password::from("secure_password");

// Using (only when building connection strings)
let password_str: &str = password.expose_secret();
```

### PlainApiToken & PlainPassword

Type aliases for `String` used at DTO boundaries to mark temporarily transparent secrets.

**When to Use**: Configuration DTOs that accept user input before converting to secure types.

**Example**:

```rust
use crate::shared::secrets::{PlainApiToken, ApiToken};

// DTO layer (accepts plain string from user)
#[derive(Deserialize)]
pub struct HetznerProviderSection {
    pub api_token: PlainApiToken, // Type alias signals "this becomes secret"
}

// Domain layer (secure type)
pub struct HetznerConfig {
    pub api_token: ApiToken, // Actual secret protection
}

// Conversion
impl HetznerProviderSection {
    pub fn to_domain(&self) -> HetznerConfig {
        HetznerConfig {
            api_token: ApiToken::from(self.api_token.clone()), // String → ApiToken
        }
    }
}
```

## 🔄 Secret Lifecycle Pattern

Secrets follow this flow through application layers:

```text
User Input (PlainApiToken/String)
  ↓ Deserialization
DTO Layer (PlainApiToken for clarity)
  ↓ Conversion
Domain Layer (ApiToken for security)
  ↓ Pass through
Infrastructure Layer (ApiToken.expose_secret() only when needed)
  ↓ Use in external calls
External API/Database
```

## 📝 Usage Guidelines

### ✅ DO

- **Use secret types for all sensitive data**:

  ```rust
  pub struct MysqlConfig {
      pub password: Password, // ✅ Protected
  }
  ```

- **Use `PlainApiToken`/`PlainPassword` at DTO boundaries**:

  ```rust
  #[derive(Deserialize)]
  pub struct ConfigDto {
      pub api_token: PlainApiToken, // ✅ Clearly marked as temporary
  }
  ```

- **Call `.expose_secret()` only when needed**:

  ```rust
  fn connect_database(config: &MysqlConfig) -> Connection {
      let password = config.password.expose_secret(); // ✅ Only here
      Connection::new(&config.host, password)
  }
  ```

- **Document why secrets are exposed**:

  ```rust
  // ✅ Clear reasoning
  fn build_connection_string(config: &MysqlConfig) -> String {
      // Expose password only for connection string construction
      let password = config.password.expose_secret();
      format!("mysql://{}:{}@{}", config.username, password, config.host)
  }
  ```

### ❌ DON'T

- **Never use `String` for secrets**:

  ```rust
  pub struct Config {
      pub api_token: String, // ❌ Will leak in debug output
  }
  ```

- **Never log or print secrets**:

  ```rust
  let token = config.api_token.expose_secret();
  println!("Token: {}", token); // ❌ NEVER DO THIS
  tracing::info!("Using token: {}", token); // ❌ NEVER DO THIS
  ```

- **Never include secrets in error messages**:

  ```rust
  // ❌ BAD
  return Err(format!("Invalid token: {}", token.expose_secret()));

  // ✅ GOOD
  return Err("Invalid token format".to_string());
  ```

- **Never store exposed secrets**:

  ```rust
  // ❌ BAD - Defeats the purpose
  let exposed_token = config.api_token.expose_secret().to_string();

  // ✅ GOOD - Keep as secret type
  let token = &config.api_token;
  ```

## 🧪 Testing with Secrets

### Test Data

Use `.from()` for test secrets:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::secrets::ApiToken;

    #[test]
    fn it_should_create_config_with_api_token() {
        let config = HetznerConfig {
            api_token: ApiToken::from("test-token-123"),
            server_type: "cx22".to_string(),
            // ...
        };

        // Secrets are automatically redacted in test failures
        assert_eq!(config.server_type, "cx22");
    }
}
```

### Comparing Secrets

Secret types implement `PartialEq` and `Eq`:

```rust
#[test]
fn it_should_compare_secrets() {
    let token1 = ApiToken::from("same-token");
    let token2 = ApiToken::from("same-token");
    let token3 = ApiToken::from("different-token");

    assert_eq!(token1, token2); // ✅ Works
    assert_ne!(token1, token3); // ✅ Works
}
```

## 🔍 Identifying Secret Fields

Ask these questions:

1. **Would it be a security issue if this appeared in logs?** → Use secret type
2. **Is this used for authentication/authorization?** → Use secret type
3. **Should this be redacted in debug output?** → Use secret type
4. **Does this access protected resources?** → Use secret type

**Common Secret Patterns**:

- Anything named `*_token`, `*_password`, `*_secret`, `*_key`
- Database credentials
- API keys and tokens
- Private keys
- Authentication credentials

## 🛠️ Implementation Checklist

When adding a new secret field:

- [ ] Choose correct type (`ApiToken` for tokens, `Password` for passwords)
- [ ] Use `Plain*` type alias at DTO boundaries if accepting user input
- [ ] Update struct field to use secret type
- [ ] Add `use crate::shared::secrets::{ApiToken, PlainApiToken};` import
- [ ] Update construction sites to use `.from()`
- [ ] Update usage sites to call `.expose_secret()` only when necessary
- [ ] Verify debug output redacts the secret
- [ ] Update all tests to use secret types
- [ ] Check serialization/deserialization works correctly
- [ ] Document why `.expose_secret()` is called at each location

## 🔗 Related Documentation

- **Architecture Decision**: [`docs/decisions/secrecy-crate-for-sensitive-data.md`](../decisions/secrecy-crate-for-sensitive-data.md) - Why we use the secrecy crate
- **Implementation Details**: `src/shared/secrets/` - Source code for secret types
- **Refactor Plan**: [`docs/refactors/plans/secret-type-introduction.md`](../refactors/plans/secret-type-introduction.md) - Migration tracking

## 🔐 Security Features

Secret types provide multiple layers of protection:

1. **Debug Redaction**: `Debug` output shows `Secret([REDACTED])` instead of actual value
2. **Memory Zeroing**: Secrets are wiped from memory when dropped (via `zeroize` crate)
3. **Explicit Access**: `.expose_secret()` makes secret usage visible in code
4. **Type Safety**: Compiler enforces correct handling
5. **Serialization Control**: Can serialize when needed (config files) but remains protected in memory

## ❓ FAQ

### Why not just be careful with logging?

Human error is inevitable. Type-level protection prevents mistakes at compile time.

### Can I serialize secrets?

Yes, secret types implement `Serialize`/`Deserialize` for configuration files. However, they remain protected in memory and debug output.

### What if I need the secret value?

Call `.expose_secret()` - but document why it's necessary. This makes secret usage explicit and auditable.

### Do I need to wrap everything?

No, only sensitive data. Regular strings for non-secret configuration values are fine.

### What about plain types (PlainApiToken)?

Use them at DTO boundaries to clearly mark "this will become a secret." They're just type aliases for `String` but signal intent.
