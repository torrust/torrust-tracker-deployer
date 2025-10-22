# Console Output & Logging Strategy Research

> **📋 Research Document Status**  
> This document contains research findings and exploration of console output patterns. These findings may differ from the current implementation approach described in the main project documentation. The final implementation strategy is still being decided.

## Overview

This document summarizes research on how console applications handle logging and output that is intended for end-users. The goal is to establish patterns for separating user-facing output from internal application logs.

## Relationship to Current Project Design

This research explores logging and output strategies independently of the current command structure. The project currently follows a **granular command approach** with individual commands for each deployment step (see [Console Commands](../../console-commands.md) and [Deployment Overview](../../deployment-overview.md)):

```bash
# Current planned approach (individual commands)
torrust-tracker-deployer create <env>
torrust-tracker-deployer provision <env>
torrust-tracker-deployer configure <env>
torrust-tracker-deployer release <env>
torrust-tracker-deployer run <env>
```

The logging and output principles researched here can be applied to **any command structure** - whether using individual step commands, a unified wizard approach, or a hybrid model that combines both.

## Types of Console Applications

We analyzed different styles of console applications to understand their output patterns:

- **Single-command tools** (e.g., `ls`, `cat`) - Simple output to stdout
- **Long-lived services** (daemons) - Structured logging to files/syslog
- **Interactive assistants** - Mix of user prompts and feedback
- **Rich TUIs** (Text User Interfaces) - Panels, widgets, and interactive components

## Main Challenge

The primary challenge identified is how to handle **user-facing output** vs. **logs**, since both often compete for stdout space and can create confusion for users.

## Key Design Decisions

### 1. Separate Concerns

- **User output** (progress, status, results) → `stdout`
- **Logs** (info, debug, trace, warnings, errors) → `stderr` (and optionally a log file)

This separation allows users to:

- Pipe user output to other tools without noise from logs
- Redirect logs separately for debugging
- Maintain clean, readable user experience

### 2. Verbosity Control System

Implement a graduated verbosity system similar to other CLI tools:

- **Default** (no `-v`) → Only warnings and errors to stderr
- **`-v`** → Show `INFO` level logs to stderr
- **`-vv`** → Show `DEBUG` level logs to stderr
- **`-vvv`** → Show `TRACE` level logs to stderr

### 3. Cargo-like Approach

Follow the successful pattern used by Cargo:

- **Normal run**: Clean, concise summary for users
- **Verbose run**: Detailed logs for debugging and troubleshooting

### 4. Future TUI Possibility

While keeping the door open for future TUI enhancements:

- Progress bars and spinners
- Task lists with status indicators
- Collapsible log panels
- Real-time status updates

For now, focus on the simpler stdout + stderr separation model.

## Implementation Strategy

### Core Components

1. **Logging Framework**: Use the [`tracing`](https://crates.io/crates/tracing) crate for structured logging
2. **User Output Abstraction**: Custom `UserOutput` trait for user-facing messages
3. **Separation of Concerns**: Clear distinction between logs and user communication

### Rust Design Pattern

```rust
// UserOutput trait for user-facing messages
pub trait UserOutput {
    fn msg(&self, message: &str);
    fn success(&self, message: &str);
    fn warning(&self, message: &str);
    fn error(&self, message: &str);
}

// Production implementation
pub struct StdoutOutput;
impl UserOutput for StdoutOutput {
    fn msg(&self, message: &str) {
        println!("{}", message);
    }
    // ... other methods
}

// Test implementation
pub struct MockOutput {
    messages: Vec<String>,
}
impl UserOutput for MockOutput {
    fn msg(&self, message: &str) {
        self.messages.push(message.to_string());
    }
    // ... other methods
}
```

### Usage Patterns

```rust
// User-facing messages
user_output.msg("Starting deployment...");
user_output.success("✅ Provisioning complete");
user_output.error("❌ Configuration failed");

// Internal logs (to stderr when verbose)
info!("Loading configuration from {}", path);
debug!("Ansible inventory generated: {:?}", inventory);
trace!("Raw HTTP response: {}", response_body);
warn!("Using default SSH key path");
error!("Failed to connect to instance: {}", error);
```

## Benefits

### Testability

- **Mock user output** in tests for asserting user-facing messages
- **Capture logs** separately for testing internal behavior
- **Isolated concerns** make unit testing straightforward

### User Experience

- **Clean output** by default for normal operations
- **Verbose mode** available when troubleshooting
- **Familiar patterns** similar to Cargo, Ansible, and Terraform

### Flexibility

- **Easy migration** to TUI components later
- **Configurable output** (colors, timestamps, formatting)
- **Multiple backends** (stdout, files, network, etc.)

### Developer Experience

- **Clear separation** between user communication and logging
- **Consistent patterns** across the entire application
- **Easy debugging** with structured, filterable logs

## Implementation Notes

- All user-facing messages should go through the `UserOutput` abstraction
- Never mix `println!` with `tracing` macros for the same type of information
- Use appropriate log levels to enable useful verbose modes
- Consider future extensibility when designing the user output interface
- Maintain consistency with established CLI tool patterns

This approach provides a solid foundation for console output that can evolve with the project's needs while maintaining clear separation of concerns and excellent testability.
