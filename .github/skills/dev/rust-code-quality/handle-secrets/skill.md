---
name: handle-secrets
description: Guide for handling sensitive data (secrets) in this Rust project. NEVER use String for API tokens, passwords, private keys, or database credentials. Use ApiToken or Password wrapper types from src/shared/secrets/, and PlainApiToken/PlainPassword at DTO boundaries. Call .expose_secret() only when the actual value is needed. Prevents accidental exposure through Debug output and logs. Use when working with credentials, API keys, tokens, passwords, or any sensitive configuration. Triggers on "secret", "API token", "password", "credential", "sensitive data", "ApiToken", "secrecy", or "expose secret".
metadata:
  author: torrust
  version: "1.0"
---

# Handling Sensitive Data (Secrets)

## Core Rule

**NEVER use `String` for sensitive data.** Use wrapper types from `src/shared/secrets/`.

```rust
// ❌ WRONG: secret leaked in Debug output
pub struct HetznerConfig {
    pub api_token: String,
}
println!("{config:?}"); // → HetznerConfig { api_token: "hetzner_abc123" } — LEAKED!
```

```rust
// ✅ CORRECT: secret redacted in Debug
use crate::shared::secrets::ApiToken;
pub struct HetznerConfig {
    pub api_token: ApiToken,
}
println!("{config:?}"); // → HetznerConfig { api_token: Secret([REDACTED]) }
```

## Type Reference

| Type            | Use for                                    | Where          |
| --------------- | ------------------------------------------ | -------------- |
| `ApiToken`      | Cloud API tokens, admin tokens, API keys   | Domain / Infra |
| `Password`      | DB passwords, SSH passwords, service creds | Domain / Infra |
| `PlainApiToken` | DTO boundaries accepting raw string input  | Config DTOs    |
| `PlainPassword` | DTO boundaries accepting raw string input  | Config DTOs    |

`PlainApiToken` and `PlainPassword` are type aliases for `String` — they signal "this will become a secret".

## Using Secrets

```rust
use crate::shared::secrets::ApiToken;
use secrecy::ExposeSecret;

let token = ApiToken::from("my-api-token-123");

// Only call .expose_secret() when making the actual API call
let token_str: &str = token.expose_secret();
```

## DTO Boundary Pattern

```rust
use crate::shared::secrets::{PlainApiToken, ApiToken};

// DTO layer (user input)
#[derive(Deserialize)]
pub struct HetznerSection {
    pub api_token: PlainApiToken,  // signals "becomes secret"
}

// Convert to secret type before passing to domain
let config = HetznerConfig {
    api_token: ApiToken::from(dto.api_token),
};
```

## Checklist

- [ ] No `String` fields for tokens, passwords, or private keys
- [ ] `ApiToken` or `Password` used in domain and infrastructure structs
- [ ] `PlainApiToken`/`PlainPassword` used only at DTO/deserialization boundaries
- [ ] `.expose_secret()` called only at the last moment (API call or connection string)
- [ ] No `.expose_secret()` in log statements or error messages

## Reference

Full guide: [`docs/contributing/secret-handling.md`](../../docs/contributing/secret-handling.md)
ADR: [`docs/decisions/secrecy-crate-for-sensitive-data.md`](../../docs/decisions/secrecy-crate-for-sensitive-data.md)
