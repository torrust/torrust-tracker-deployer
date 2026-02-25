---
name: organize-rust-modules
description: Guide for organizing Rust modules and imports in this project. Covers import ordering (std → external crates → internal crate), preferring short imported names over fully-qualified paths, top-down module organization (public items first, high-level abstractions before low-level details), and placing error types at the end. Use when creating new Rust files, organizing module contents, adding imports, or reviewing code structure. Triggers on "module organization", "import order", "use statement", "Rust imports", "organize module", "code structure", or "module structure".
metadata:
  author: torrust
  version: "1.0"
---

# Organizing Rust Modules

## Rule 1: Imports Always First

All `use` statements go at the top of the file, in three ordered groups:

```rust
// Group 1: Standard library
use std::path::{Path, PathBuf};
use std::sync::Arc;

// Group 2: External crates
use anyhow::Result;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// Group 3: Internal crate (absolute paths first, then relative)
use crate::domain::Environment;
use crate::shared::Clock;
use super::config::Config;
```

## Rule 2: Prefer Short Imported Names

Always import types and use short names. Never use long inline paths.

```rust
// ✅ Good
use std::sync::Arc;
use crate::presentation::views::UserOutput;

pub struct Handler {
    output: Arc<UserOutput>,
}
```

```rust
// ❌ Bad: verbose, hard to read
pub struct Handler {
    output: std::sync::Arc<crate::presentation::views::UserOutput>,
}
```

**Exception**: Full paths only when disambiguating same-named types:

```rust
use crate::domain::Environment as DomainEnvironment;
// use full path for the other one to disambiguate
fn compare(a: &DomainEnvironment, b: &crate::config::Environment) { ... }
```

## Rule 3: Top-Down Module Organization

Order items within a module:

1. **Public items first** — what consumers care about
2. **High-level abstractions before low-level details**
3. **Main responsibilities before secondary concerns**
4. **Error types last** — `pub enum MyError { ... }`

```rust
// ✅ Good ordering in a module file
pub struct MyCommand { ... }          // 1. Main public struct

impl MyCommand {                      // 2. Primary implementation
    pub fn new(...) -> Self { ... }
    pub fn execute(...) -> Result<...> { ... }
}

fn helper_fn() { ... }                // 3. Private helpers

#[derive(Debug, thiserror::Error)]    // 4. Error type last
pub enum MyCommandError { ... }
```

## Quick Checklist

- [ ] All `use` statements at top of file before any other items
- [ ] `use` groups separated by blank lines (std, external, internal)
- [ ] No inline full paths like `std::sync::Arc<crate::...>` in function bodies
- [ ] Public items declared before private items in modules
- [ ] Error types defined at the bottom of the module

## Reference

Full guide: [`docs/contributing/module-organization.md`](../../docs/contributing/module-organization.md)
