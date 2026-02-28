---
name: handle-errors-in-code
description: Guide for error handling in this Rust project. Covers the four principles (clarity, context, actionability, explicit enums over anyhow), the thiserror pattern for structured errors, including what/where/when/why context, writing actionable help text, and avoiding vague errors. Use when writing error types, handling Results, adding error variants, or reviewing error messages. Triggers on "error handling", "error type", "Result", "thiserror", "anyhow", "error enum", "error message", "handle error", or "add error variant".
metadata:
  author: torrust
  version: "1.0"
---

# Handling Errors in Code

## Core Principles

1. **Clarity** — Users immediately understand what went wrong
2. **Context** — Include what/where/when/why
3. **Actionability** — Tell users how to fix it
4. **Explicit enums over `anyhow`** — Prefer structured errors for pattern matching

## Prefer Explicit Enum Errors

```rust
// ✅ Correct: explicit, matchable, clear
#[derive(Debug, thiserror::Error)]
pub enum ProvisionError {
    #[error("Instance '{instance_name}' already exists in {provider}")]
    InstanceAlreadyExists { instance_name: String, provider: String },

    #[error("SSH key not found at '{path}'. Generate with: ssh-keygen -t ed25519 -f '{path}'")]
    SshKeyNotFound { path: PathBuf },
}

// ❌ Wrong: opaque, hard to match
return Err(anyhow::anyhow!("Something went wrong"));
return Err("Invalid input".into());
```

## Include Actionable Fix Instructions in Display

```rust
impl Display for DeploymentError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::SshKeyNotFound { path } => write!(
                f,
                "SSH key not found at '{}'.\n\n\
                 To fix:\n\
                 1. Generate: ssh-keygen -t ed25519 -f '{}'\n\
                 2. Or specify --ssh-key-path",
                path.display(), path.display()
            ),
        }
    }
}
```

## Context Requirements

Each error should answer:

- **What**: What operation was being performed?
- **Where**: Which component, file, or resource?
- **When**: Under what conditions?
- **Why**: What caused the failure?

```rust
// ✅ Good: full context
#[error("Network timeout during '{operation}' to '{endpoint}' after {timeout:?}. Check connectivity.")]
NetworkTimeout { operation: String, timeout: Duration, endpoint: String },

// ❌ Bad: no context
return Err("timeout".into());
```

## Add help() for User-Facing Errors

```rust
impl ProvisionError {
    pub fn help(&self) -> &'static str {
        match self {
            Self::InstanceAlreadyExists { .. } =>
                "Use a different name or remove the existing instance with `destroy`.",
            Self::SshKeyNotFound { .. } =>
                "Run: ssh-keygen -t ed25519 or specify --ssh-key-path",
        }
    }
}
```

## Unwrap and Expect Policy

| Context                | `.unwrap()`   | `.expect("msg")`                             | `?` / `Result` |
| ---------------------- | ------------- | -------------------------------------------- | -------------- |
| Production code        | ❌ Never      | ✅ Only when failure is logically impossible | ✅ Default     |
| Tests and doc examples | ✅ Acceptable | ✅ Preferred when message adds clarity       | —              |

```rust
// ✅ Production: propagate errors with ?
fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| ConfigError::FileAccess { path: path.to_path_buf(), source: e })?;
    serde_json::from_str(&content)
        .map_err(|e| ConfigError::InvalidJson { path: path.to_path_buf(), source: e })
}

// ✅ Production: expect() only when failure is a code invariant violation
let pair = "key=value";
let (k, v) = pair.split_once('=')
    .expect("split on '=' always succeeds: the string literal contains '='");

// ❌ Production: never unwrap()
let value = some_result.unwrap();

// ✅ Tests and doc examples: unwrap() is fine
#[test]
fn it_should_parse_valid_config() {
    let config: Config = serde_json::from_str(VALID_JSON).unwrap();
    assert_eq!(config.name, "test");
}
```

## Quick Checklist

- [ ] Error type uses `thiserror::Error` derive
- [ ] Error message includes specific context (names, paths, values)
- [ ] Error message includes fix instructions where possible
- [ ] Prefer `enum` over `Box<dyn Error>` or `anyhow`
- [ ] No vague messages like "invalid input" or "error occurred"
- [ ] No `.unwrap()` in production code (tests and doc examples are fine)

## Reference

Full guide: [`docs/contributing/error-handling.md`](../../docs/contributing/error-handling.md)
