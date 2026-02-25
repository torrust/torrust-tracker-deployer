# Torrust Deployer Types

Shared value objects and traits for the Torrust Tracker Deployer ecosystem.

## Overview

This package provides foundational types shared between:

- `torrust-tracker-deployer` (root crate)
- `torrust-tracker-deployer-sdk` (SDK package)

These are validated value objects and cross-cutting traits with no business logic
and minimal external dependencies.

## Types

| Type                         | Description                                 |
| ---------------------------- | ------------------------------------------- |
| `Clock` / `SystemClock`      | Time abstraction for testability            |
| `DomainName`                 | Validated DNS-like domain name              |
| `Email`                      | Validated email address                     |
| `EnvironmentName`            | Validated environment identifier            |
| `Username`                   | Validated username string                   |
| `ServiceEndpoint`            | Validated URL + port combination            |
| `ApiToken` / `PlainApiToken` | Secret API token wrapper                    |
| `Password` / `PlainPassword` | Secret password wrapper                     |
| `ErrorKind`                  | High-level error categorization enum        |
| `Traceable`                  | Trait for structured error trace generation |

## Usage

```toml
[dependencies]
torrust-deployer-types = { path = "packages/deployer-types" }
```

```rust
use torrust_deployer_types::{EnvironmentName, DomainName, Clock};
```

## Architecture

```text
torrust-deployer-types           ← this package (no internal deps)
    ↑                   ↑
torrust-tracker-deployer  torrust-tracker-deployer-sdk
```

Both the root crate and the SDK package depend on this package for shared types.

## License

MIT
