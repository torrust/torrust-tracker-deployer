# Console Applications: stdout and stderr Channel Handling

This document analyzes how console applications handle stdout and stderr output channels and their implications for user experience in the Torrust Tracker Deploy project.

## 📋 Overview

Console applications typically use two primary output channels:

- **stdout** (standard output): Used for normal program output and data
- **stderr** (standard error): Used for error messages, warnings, and diagnostic information

## 🔧 How Applications Use These Channels

### Standard Practices

#### stdout (File Descriptor 1)

- **Purpose**: Primary output data, results, and normal program flow information
- **Examples**: Command results, structured data, success messages
- **Redirection**: Can be redirected independently (`command > output.txt`)

#### stderr (File Descriptor 2)

- **Purpose**: Error messages, warnings, debug information, and diagnostic output
- **Examples**: Error messages, warnings, progress indicators, debug logs
- **Redirection**: Can be redirected separately (`command 2> errors.txt`)

### Why Applications Mix Warning and Error Information

Many applications write various types of information to stderr, not just fatal errors:

1. **Warnings**: Non-fatal issues that users should be aware of
2. **Progress Information**: Status updates that shouldn't interfere with data output
3. **Debug Information**: Diagnostic information for troubleshooting
4. **Configuration Messages**: Information about settings or environment

## 🐛 SSH-Specific Behavior

SSH is a prime example of an application that uses stderr for both errors and informational messages:

### SSH Warning Messages to stderr

```bash
# SSH writes host key warnings to stderr even when connection succeeds
ssh -o StrictHostKeyChecking=no user@host command
# stderr: "Warning: Permanently added 'host' (ED25519) to the list of known hosts."
# stdout: command output
```

### Why SSH Does This

1. **Data Integrity**: Keeps actual command output clean on stdout
2. **User Awareness**: Ensures security warnings are visible
3. **Scriptability**: Allows scripts to capture command output without warnings
4. **Standard Practice**: Follows Unix convention of using stderr for diagnostics

## 🔍 Impact on Error Detection

### Challenge: Detecting Real Errors

When applications write warnings to stderr, it becomes challenging to distinguish between:

- **Real Errors**: Actual failures that should stop execution
- **Warnings**: Information that should be noted but doesn't indicate failure
- **Informational Messages**: Status updates and diagnostics

### Current Behavior in Torrust Deploy

Our current implementation treats any stderr output as an error:

```rust
// From the error logs - this treats SSH warnings as errors
ERROR torrust_tracker_deploy::shared::executor: Command produced stderr output
```

### SSH Examples from Our Logs

```text
stderr: Warning: Permanently added '10.140.190.144' (ED25519) to the list of known hosts.
```

This is **not an error** - it's SSH informing the user that it's adding the host to known_hosts file.

## 💡 Common Solutions

### 1. Exit Code Checking

The most reliable way to detect actual errors:

```bash
# Exit code 0 = success, regardless of stderr content
ssh user@host command
if [ $? -eq 0 ]; then
    echo "Command succeeded (stderr may contain warnings)"
else
    echo "Command failed"
fi
```

### 2. Stderr Content Filtering

Filter known warning patterns:

```rust
fn is_ssh_warning(stderr: &str) -> bool {
    stderr.contains("Warning: Permanently added") ||
    stderr.contains("Host key verification") ||
    stderr.contains("known_hosts")
}
```

### 3. Separate Warning and Error Handling

Distinguish between warning and error log levels:

```rust
if exit_code != 0 {
    log::error!("Command failed: {}", stderr);
} else if !stderr.is_empty() {
    log::warn!("Command succeeded with warnings: {}", stderr);
}
```

### 4. Application-Specific Handling

Different applications may need different stderr interpretation:

```rust
match command_type {
    CommandType::Ssh => handle_ssh_stderr(stderr, exit_code),
    CommandType::Docker => handle_docker_stderr(stderr, exit_code),
    CommandType::Ansible => handle_ansible_stderr(stderr, exit_code),
}
```

## 🎯 Recommendations for Torrust Deploy

### Short Term (Current State)

1. **Document Known Warnings**: Clearly document that SSH host key warnings are expected
2. **User Education**: Explain that red error messages may include harmless warnings
3. **Log Level Adjustment**: Consider using WARN level for stderr when exit code is 0

### Medium Term (Improvement)

1. **Smart Error Detection**: Implement exit code checking before treating stderr as error
2. **Warning Filtering**: Filter out known harmless warnings from error logs
3. **Contextual Handling**: Different stderr handling per application type

### Long Term (Enhanced UX)

1. **Structured Logging**: Separate warning and error channels in our logging system
2. **User-Friendly Output**: Hide technical warnings from end users unless in verbose mode
3. **Progress Indicators**: Replace stderr monitoring with proper progress reporting

## 📚 References

- [Unix Standard Streams](https://en.wikipedia.org/wiki/Standard_streams)
- [SSH Manual - Host Key Verification](https://man.openbsd.org/ssh#Host_key_verification)
- [Rust std::process::Command](https://doc.rust-lang.org/std/process/struct.Command.html)
- [POSIX Exit Codes](https://www.gnu.org/software/libc/manual/html_node/Exit-Status.html)

## 🔗 Related Documentation

- [Known Issues](../contributing/known-issues.md) - Specific known warnings and errors
- [Error Handling Guide](../contributing/error-handling.md) - Error handling principles
- [Development Principles](../development-principles.md) - Observability and user-friendliness principles
