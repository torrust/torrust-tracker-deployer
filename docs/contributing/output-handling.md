# Output Handling Guide

## 📋 Overview

This guide explains how to properly handle user-facing output in this application. Following these conventions ensures consistent, testable, and user-friendly output across all commands.

## 🎯 Golden Rule

**NEVER write directly to stdout/stderr using standard library functions.**

❌ **Wrong**:

```rust
// NEVER DO THIS
println!("Processing...");
eprintln!("Error occurred");
std::io::stdout().write_all(b"data").unwrap();
std::io::stderr().write_all(b"error").unwrap();
```

✅ **Correct**:

```rust
// ALWAYS USE UserOutput
let user_output = ctx.user_output();
user_output.lock().borrow_mut().progress("Processing...");
user_output.lock().borrow_mut().error("Error occurred");
user_output.lock().borrow_mut().result("data");
```

## 🏗️ Architecture Overview

The application uses a **sink-based output architecture** in the Views layer (`src/presentation/views/`):

```text
Controllers → UserOutput → OutputSink → stdout/stderr
```

### Why This Architecture?

1. **Testability**: Output can be captured and asserted in tests
2. **Flexibility**: Multiple output destinations (console, file, telemetry)
3. **Consistency**: All output follows the same formatting rules
4. **Channel Separation**: Automatic routing to stdout vs stderr
5. **Verbosity Control**: Centralized verbosity filtering
6. **Theme Support**: Consistent visual appearance with emoji/plain/ASCII themes

## 📺 Channel Strategy (Unix Conventions)

The application follows Unix conventions with dual-channel output:

| Channel    | Purpose                                   | Examples                                  |
| ---------- | ----------------------------------------- | ----------------------------------------- |
| **stdout** | Final results, structured data for piping | JSON output, deployment results           |
| **stderr** | Progress, status, warnings, errors        | "Destroying environment...", "✅ Success" |

This enables clean piping:

```bash
# Progress goes to stderr, result goes to stdout
torrust-tracker-deployer destroy env | jq .status

# Separate output streams
torrust-tracker-deployer create env > results.json 2> logs.txt
```

## 🎨 Output in Controllers: `ProgressReporter`

### Higher-Level Abstraction for Controllers

**Controllers use `ProgressReporter`**, not `UserOutput` directly. `ProgressReporter` is a higher-level abstraction built on top of `UserOutput` that provides:

- **Step Tracking**: Numbered progress through multi-step operations (e.g., "[1/5] Loading configuration...")
- **Timing Information**: Automatic timing for each step
- **Sub-step Support**: Detailed progress within major steps
- **Consistent Format**: Standardized output across all commands

### Using `ProgressReporter` in Controllers

Controllers receive `UserOutput` via dependency injection and create a `ProgressReporter`:

```rust
use crate::presentation::views::progress::ProgressReporter;
use crate::presentation::views::UserOutput;

pub struct ConfigureCommandController {
    repository: Arc<dyn EnvironmentRepository + Send + Sync>,
    clock: Arc<dyn Clock>,
    progress: ProgressReporter,  // Use this, not UserOutput directly
}

impl ConfigureCommandController {
    pub fn new(
        repository: Arc<dyn EnvironmentRepository + Send + Sync>,
        clock: Arc<dyn Clock>,
        user_output: Arc<ReentrantMutex<RefCell<UserOutput>>>,
    ) -> Self {
        // Create ProgressReporter with total step count
        let progress = ProgressReporter::new(user_output, 3);

        Self {
            repository,
            clock,
            progress,
        }
    }

    pub fn execute(&mut self, environment_name: &str) -> Result<Environment<Configured>, Error> {
        // Step 1: Validation
        self.progress.start_step("Validating environment")?;
        let env_name = validate_name(environment_name)?;
        self.progress.complete_step(Some(&format!("Environment name validated: {}", env_name)))?;

        // Step 2: Create handler
        self.progress.start_step("Creating command handler")?;
        let handler = create_handler()?;
        self.progress.complete_step(None)?;

        // Step 3: Configure
        self.progress.start_step("Configuring infrastructure")?;
        self.progress.sub_step("Creating virtual machine")?;
        self.progress.sub_step("Configuring network")?;
        let result = handler.execute()?;
        self.progress.complete_step(Some("Instance configured"))?;

        // Complete workflow
        self.progress.complete("Environment configured successfully")?;

        Ok(result)
    }
}
```

### Direct `UserOutput` Usage

**Only use `UserOutput` directly** in simple scenarios where step tracking isn't needed:

````rust
// For simple messages without step tracking
let user_output = ctx.user_output();
user_output.lock().borrow_mut().warn("Using default configuration");
user_output.lock().borrow_mut().error("Failed to connect");
```### Message Types

Use the appropriate message method based on what you're communicating:

#### 1. Progress Messages (stderr, Normal+)

For ongoing operations and status updates:

```rust
user_output.lock().borrow_mut().progress("Destroying environment...");
user_output.lock().borrow_mut().progress("Waiting for instance to be ready...");
````

#### 2. Success Messages (stderr, Normal+)

For successful completion notifications:

```rust
user_output.lock().borrow_mut().success("Environment destroyed successfully");
user_output.lock().borrow_mut().success("Configuration applied");
```

#### 3. Warning Messages (stderr, Normal+)

For non-critical issues:

```rust
user_output.lock().borrow_mut().warn("Using default SSH port");
user_output.lock().borrow_mut().warn("Infrastructure may already be destroyed");
```

#### 4. Error Messages (stderr, all levels)

For errors (always shown regardless of verbosity):

```rust
user_output.lock().borrow_mut().error("Failed to connect to instance");
user_output.lock().borrow_mut().error("Invalid environment name");
```

#### 5. Result/Data Messages (stdout, Normal+)

For final results and structured data that users might pipe:

```rust
user_output.lock().borrow_mut().result(r#"{"status": "destroyed"}"#);
user_output.lock().borrow_mut().data(json_output);
```

#### 6. Info Blocks (stderr, Normal+)

For grouped information with a title and multiple lines:

```rust
user_output.lock().borrow_mut().info_block(
    "Configuration options:",
    &[
        "  - username: 'torrust' (default)",
        "  - port: 22 (default SSH port)",
        "  - key_path: path/to/key",
    ]
);
```

#### 7. Steps Instructions (stderr, Normal+)

For multi-step instructions or guides:

```rust
use crate::presentation::views::StepsMessageBuilder;

let steps = StepsMessageBuilder::new("Next steps:")
    .add_step("Run: torrust-tracker-deployer provision my-env")
    .add_step("Run: torrust-tracker-deployer configure my-env")
    .add_step("Run: torrust-tracker-deployer release my-env")
    .build();

user_output.lock().borrow_mut().write(&steps);
```

#### 8. Blank Lines (stderr, Normal+)

For visual spacing between sections:

```rust
user_output.lock().borrow_mut().blank_line();
```

## 📊 Verbosity Levels

Messages are automatically filtered based on verbosity level:

| Level           | CLI Flag  | What's Shown                                 |
| --------------- | --------- | -------------------------------------------- |
| **Quiet**       | `-q`      | Only errors and essential results            |
| **Normal**      | (default) | Progress, success, warnings, errors, results |
| **Verbose**     | `-v`      | Detailed progress information                |
| **VeryVerbose** | `-vv`     | Including decisions and retries              |
| **Debug**       | `-vvv`    | Maximum detail for troubleshooting           |

The verbosity level is set when creating `UserOutput` and filtering is automatic. You don't need to check verbosity manually - just use the appropriate message method.

## 🎭 Themes

The application supports multiple visual themes:

- **Emoji** (default): `✅ ❌ ⚠️ ⏳ 🔍`
- **Plain**: `[OK] [ERROR] [WARN] [...]`
- **ASCII**: `[+] [-] [!] [*]`

Themes are applied automatically. You don't need to include emoji or symbols in your messages - the theme system handles this:

```rust
// Just provide the message text
user_output.lock().borrow_mut().success("Operation completed");

// Theme system outputs:
// Emoji theme:  ✅ Operation completed
// Plain theme:  [OK] Operation completed
// ASCII theme:  [+] Operation completed
```

## 🚫 Common Anti-Patterns

### ❌ Anti-Pattern 1: Direct stdout/stderr Access

```rust
// WRONG - bypasses architecture
println!("Starting...");
eprintln!("Error: {}", error);
std::io::stdout().write_all(b"data").unwrap();
```

**Why it's wrong**: Breaks testability, bypasses verbosity control, inconsistent formatting, can't be captured.

**Fix**: Use `UserOutput` methods.

### ❌ Anti-Pattern 2: Manual Channel Selection

```rust
// WRONG - don't manually route to channels
writeln!(std::io::stderr(), "Progress message").unwrap();
```

**Why it's wrong**: `OutputMessage` trait handles routing automatically.

**Fix**: Use the appropriate message method.

### ❌ Anti-Pattern 3: Manual Verbosity Checks

```rust
// WRONG - don't check verbosity manually
if verbosity >= VerbosityLevel::Verbose {
    println!("Detailed info");
}
```

**Why it's wrong**: Verbosity filtering is automatic.

**Fix**: Use the message method - filtering happens automatically.

### ❌ Anti-Pattern 4: Including Symbols in Messages

```rust
// WRONG - don't include emoji or symbols
user_output.lock().borrow_mut().success("✅ Operation completed");
```

**Why it's wrong**: Theme system handles symbols.

**Fix**: Provide plain text - theme adds symbols:

```rust
// CORRECT
user_output.lock().borrow_mut().success("Operation completed");
```

### ❌ Anti-Pattern 5: Mixing `println!` with Logging

```rust
// WRONG - don't mix output systems
println!("Starting operation");
info!("Operation started");  // tracing log
```

**Why it's wrong**: User output and internal logging serve different audiences.

**Fix**: Use `UserOutput` for users, `tracing` for developers:

```rust
// CORRECT
user_output.lock().borrow_mut().progress("Starting operation");
info!("Operation started with parameters: {:?}", params);  // Internal log
```

## 🧪 Testing Output

### Capturing Output in Tests

Use test helpers to capture and assert on output:

```rust
use std::io::Cursor;
use crate::presentation::views::{UserOutput, VerbosityLevel, Theme};

#[test]
fn it_should_display_success_message() {
    // Create buffers for capturing output
    let stdout_buf = Vec::new();
    let stderr_buf = Vec::new();

    // Create UserOutput with buffers
    let mut output = UserOutput::with_theme_and_writers(
        VerbosityLevel::Normal,
        Theme::plain(),
        Box::new(Cursor::new(stdout_buf)),
        Box::new(Cursor::new(stderr_buf)),
    );

    // Write message
    output.success("Operation completed");

    // Get captured stderr
    let stderr = output.get_stderr_content();

    // Assert
    assert!(stderr.contains("[OK] Operation completed"));
}
```

### Testing with Mock Sinks

For integration tests, use mock sinks:

```rust
use crate::presentation::views::testing::MockSink;

#[test]
fn it_should_write_to_sink() {
    let sink = Arc::new(Mutex::new(MockSink::new()));
    let mut output = UserOutput::with_sink(
        VerbosityLevel::Normal,
        Box::new(sink.clone())
    );

    output.progress("Processing...");

    let messages = sink.lock().unwrap().messages();
    assert_eq!(messages.len(), 1);
}
```

## 📚 Related Documentation

- **Architecture Overview**: [`docs/codebase-architecture.md`](../codebase-architecture.md)
- **User Output vs Logging**: [`docs/research/UX/user-output-vs-logging-separation.md`](../research/UX/user-output-vs-logging-separation.md)
- **Console Output Strategy**: [`docs/research/UX/console-output-logging-strategy.md`](../research/UX/console-output-logging-strategy.md)
- **Presentation Layer Design**: [`docs/analysis/presentation-layer/design-proposal.md`](../analysis/presentation-layer/design-proposal.md)
- **DDD Layer Placement**: [`docs/contributing/ddd-layer-placement.md`](./ddd-layer-placement.md) (see Presentation Layer section)

## ✅ Checklist for Contributors

Before submitting code that produces output:

- [ ] No direct use of `println!`, `eprintln!`, `print!`, `eprint!`
- [ ] No direct access to `std::io::stdout()` or `std::io::stderr()`
- [ ] Controllers use `ProgressReporter` for multi-step operations
- [ ] Direct `UserOutput` only for simple, non-step messages
- [ ] Used appropriate message type (progress, success, error, etc.)
- [ ] Message text is plain (no emoji or symbols)
- [ ] No manual verbosity level checks
- [ ] No manual channel routing
- [ ] Output is tested with captured buffers or mock sinks
- [ ] User-facing output separate from internal logging (`tracing`)

## 💡 Quick Reference

### For Controllers (Multi-Step Operations)

```rust
// Use ProgressReporter for structured workflows
let progress = ProgressReporter::new(user_output, total_steps);

// Step-by-step progress
progress.start_step("Loading configuration")?;
progress.complete_step(Some("Configuration loaded: test-env"))?;

progress.start_step("Provisioning infrastructure")?;
progress.sub_step("Creating virtual machine")?;
progress.sub_step("Configuring network")?;
progress.complete_step(Some("Instance created"))?;

progress.start_step("Finalizing")?;
progress.complete_step(None)?;

progress.complete("Operation completed successfully")?;
```

### For Direct Output (Simple Messages)

```rust
// Get UserOutput for simple, non-step messages
let user_output = ctx.user_output();
let mut output = user_output.lock().borrow_mut();

// Common patterns
output.progress("Starting operation...");           // Ongoing work
output.success("Operation completed");              // Success
output.warn("Using default configuration");         // Warning
output.error("Failed to connect");                  // Error
output.result(r#"{"status": "ok"}"#);              // Final result
output.blank_line();                                // Spacing

// Info blocks
output.info_block("Configuration:", &[
    "  - port: 22",
    "  - user: torrust",
]);

// Multi-step instructions
use crate::presentation::views::StepsMessageBuilder;
let steps = StepsMessageBuilder::new("Next steps:")
    .add_step("Run command 1")
    .add_step("Run command 2")
    .build();
output.write(&steps);
```

## 🎓 Summary

- **Use `ProgressReporter` in controllers** - For multi-step operations with timing and step tracking
- **Use `UserOutput` directly** - For simple messages without step tracking
- **Never write directly to stdout/stderr** - No `println!`, `eprintln!`, or `std::io` functions
- **Let the architecture handle routing** - stdout vs stderr is automatic
- **Provide plain text messages** - Theme system adds symbols
- **Trust the verbosity system** - Filtering is automatic
- **Test your output** - Use captured buffers or mock sinks
- **Separate concerns** - User output ≠ Internal logging

Following these guidelines ensures your code is consistent, testable, and maintainable while providing an excellent user experience.
