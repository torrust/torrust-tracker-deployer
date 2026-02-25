---
name: handle-user-output
description: Guide for producing user-facing output in this Rust project. NEVER use println!, eprintln!, print!, eprint!, or direct stdout/stderr access. Always use UserOutput via the execution context or ProgressReporter in controllers. Covers the sink-based output architecture, stdout vs stderr channel strategy, ProgressReporter for multi-step operations, verbosity control, and themes. Use when writing any user-visible output, progress messages, errors, or results. Triggers on "println", "print output", "user output", "UserOutput", "ProgressReporter", "display result", "output message", or "progress message".
metadata:
  author: torrust
  version: "1.0"
---

# Handling User Output

## Golden Rule

**NEVER write directly to stdout/stderr.**

```rust
// ❌ FORBIDDEN
println!("Processing...");
eprintln!("Error occurred");
std::io::stdout().write_all(b"data").unwrap();
```

```rust
// ✅ CORRECT — use UserOutput
let user_output = ctx.user_output();
user_output.lock().borrow_mut().progress("Processing...");
user_output.lock().borrow_mut().error("Error occurred");
user_output.lock().borrow_mut().result("data");
```

## In Controllers: Use ProgressReporter

Controllers use `ProgressReporter` (not `UserOutput` directly):

```rust
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;

pub struct ConfigureCommandController {
    progress: ProgressReporter,
}

impl ConfigureCommandController {
    pub fn new(user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>) -> Self {
        Self {
            progress: ProgressReporter::new(user_output, 3), // 3 total steps
        }
    }

    pub fn execute(&mut self) -> Result<(), Error> {
        self.progress.start_step("Validating environment")?;
        // ... do work ...
        self.progress.complete_step(Some("Environment name validated: my-env"))?;

        self.progress.start_step("Configuring infrastructure")?;
        // ... do work ...
        self.progress.complete_step(None)?;

        Ok(())
    }
}
```

## Channel Strategy (stdout vs stderr)

| Channel    | Use for                                         |
| ---------- | ----------------------------------------------- |
| **stdout** | Final results, structured data (JSON), pipeline |
| **stderr** | Progress, status updates, warnings, errors      |

This enables piping: `deployer create env | jq .status`

## Why This Architecture?

- **Testability**: Output captured and asserted in tests
- **Verbosity control**: Centralized filtering
- **Theme support**: emoji/plain/ASCII themes
- **Channel routing**: Automatic stdout vs stderr

## Reference

Full guide: [`docs/contributing/output-handling.md`](../../docs/contributing/output-handling.md)
