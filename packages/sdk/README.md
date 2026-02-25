# Torrust Tracker Deployer SDK

Programmatic Rust SDK for the [Torrust Tracker Deployer](../../README.md).

## Overview

This package provides a typed Rust API for deploying and managing Torrust Tracker
instances programmatically, as an alternative to the CLI.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
torrust-tracker-deployer-sdk = { git = "https://github.com/torrust/torrust-tracker-deployer" }
```

Basic usage:

```rust,no_run
use torrust_tracker_deployer_sdk::{Deployer, EnvironmentCreationConfig};

let deployer = Deployer::builder()
    .working_dir("/path/to/workspace")
    .build()
    .expect("Failed to initialize deployer");

let environments = deployer.list().expect("Failed to list environments");
```

## Examples

Run the included examples:

```bash
cargo run --example sdk_basic_usage -p torrust-tracker-deployer-sdk
cargo run --example sdk_full_deployment -p torrust-tracker-deployer-sdk
cargo run --example sdk_error_handling -p torrust-tracker-deployer-sdk
cargo run --example sdk_create_from_json_file -p torrust-tracker-deployer-sdk
cargo run --example sdk_validate_config -p torrust-tracker-deployer-sdk
```

## Architecture

```text
torrust-tracker-deployer-sdk         ← this package
    │
    ▼
torrust-tracker-deployer (root crate)
    │
    ▼
Application Layer    (command_handlers/)
    │
    ▼
Domain Layer         (environment/, template/, topology/, ...)
```

> **Note**: This package currently depends on the root `torrust-tracker-deployer`
> crate for application-layer types. Plans 3 and 4 will progressively decouple
> this dependency by extracting shared types into a `packages/deployer-types/`
> package.

## Status

This package was extracted from `src/presentation/sdk/` in the root crate as part
of the SDK workspace package refactoring (Plan 2 of 4). See the
[refactoring plan](../../docs/refactors/plans/extract-sdk-workspace-package.md)
for details.

## License

MIT
