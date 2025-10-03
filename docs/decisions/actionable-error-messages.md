# Decision Record: Actionable Error Messages

**Date**: October 3, 2025  
**Status**: ✅ Accepted  
**Context**: Implementing Proposal #8 - Improve Error Context  
**Decision Makers**: Development Team

## 📋 Problem Statement

When implementing Proposal #8 to improve error messages in the file lock module, we faced a design challenge: how to make errors actionable and helpful without overwhelming users with verbose messages or requiring external infrastructure.

### Requirements

1. **Actionability**: Errors must guide users toward solutions (development principle)
2. **Brevity**: Error messages shouldn't be overwhelming or cluttered
3. **Accessibility**: Help must be available when users need it
4. **Maintainability**: Solution should be easy to maintain and update
5. **No External Dependencies**: Avoid requiring web hosting or internet access
6. **Runtime Availability**: Help must be accessible at runtime, not just in documentation

### Initial Proposal

The initial Proposal #8 suggested embedding extensive troubleshooting steps directly in error messages:

```rust
#[error("Failed to acquire lock for '{path}' within {timeout:?}

The lock is currently held by process {holder_pid}.

To resolve this issue:
1. Check if process {holder_pid} is still running:
   ps -p {holder_pid}

2. If the process is running and should release the lock:
   - Wait for the process to complete its operation
   - Or increase the timeout duration

3. If the process is stuck or hung:
   - Terminate it: kill {holder_pid}
   - Or force terminate: kill -9 {holder_pid}

4. If the process doesn't exist (stale lock):
   - This should be handled automatically
   - If you see this error repeatedly, report a bug

Current timeout: {timeout:?}
Lock file: {path}.lock")]
```

**Problems identified:**

- ❌ Too verbose - error messages become documentation
- ❌ Maintenance burden - commands may become outdated
- ❌ Not DRY - duplicates publicly available information
- ❌ Clutters logs and terminal output

## 🔍 Alternatives Considered

### Option 1: Error Codes + External Web Documentation

**Approach**: Assign error codes (like Rust's E0001) and link to web documentation.

```rust
#[error("Failed to acquire lock for '{path}' (error code: FL001)
See: https://docs.torrust.com/errors/FL001 for troubleshooting")]
```

**Pros:**

- ✅ Clean separation of concerns
- ✅ Detailed help without cluttering errors
- ✅ Easy to update documentation independently
- ✅ Similar to Rust's own error codes

**Cons:**

- ❌ **Stability problem**: Internal errors change frequently, making code numbering unstable
- ❌ Requires infrastructure (hosting docs, numbering scheme)
- ❌ Users need internet access to see help
- ❌ Extra cognitive load (see error → look up code → read docs)
- ❌ Version synchronization issues between binary and docs

**Decision**: ❌ Rejected due to stability concerns and infrastructure requirements.

---

### Option 2: Rustdoc Only

**Approach**: Document errors in Rustdoc, accessible via `cargo doc`.

```rust
/// Failed to acquire lock within timeout period
///
/// ## Troubleshooting
///
/// 1. Check if process is running: ps -p <pid>
/// 2. Wait or increase timeout
/// 3. Terminate stuck processes
#[error("Failed to acquire lock for '{path}'")]
AcquisitionTimeout { /* ... */ }
```

**Pros:**

- ✅ Documentation lives with the code
- ✅ Easy to maintain
- ✅ Accessible via `cargo doc`

**Cons:**

- ❌ **Target audience mismatch**: Rustdoc is for developers, not end-users
- ❌ Not accessible at runtime when users need it
- ❌ Users running binaries won't have access to docs
- ❌ Doesn't help with production issues

**Decision**: ❌ Rejected because help isn't available when users need it most (at runtime).

---

### Option 3: Brief Tips Only

**Approach**: Include only one-line hints in error messages.

```rust
#[error("Failed to acquire lock for '{path}' (held by process {holder_pid})
Tip: Check if process is running or increase timeout")]
```

**Pros:**

- ✅ Balances brevity with actionability
- ✅ No external infrastructure
- ✅ Aligns with development principles

**Cons:**

- ❌ Limited guidance for complex scenarios
- ❌ Users may still struggle without detailed steps
- ❌ Platform-specific commands not easily conveyed

**Decision**: ⚠️ Good but insufficient on its own - incorporated into final solution.

---

### Option 4: External Tools/Generic References

**Approach**: Point to existing tools and generic documentation.

```rust
#[error("Failed to acquire lock for '{path}' (held by process {holder_pid})
Check process status with: ps -p {holder_pid}
See: https://docs.torrust.com/troubleshooting#file-locks")]
```

**Pros:**

- ✅ Leverages existing documentation
- ✅ Can provide deep troubleshooting
- ✅ Documentation evolves independently

**Cons:**

- ❌ Requires maintaining external docs
- ❌ Requires internet access
- ❌ Link rot over time
- ❌ Version synchronization issues

**Decision**: ❌ Rejected due to external dependency and maintenance burden.

---

### Option 5: Tiered Help System (RECOMMENDED)

**Approach**: Provide concise errors with brief tips, plus an optional `.help()` method for detailed guidance.

```rust
#[derive(Debug, Error)]
pub enum FileLockError {
    /// Failed to acquire lock within timeout period
    ///
    /// Use `.help()` for detailed troubleshooting steps.
    #[error("Failed to acquire lock for '{path}' within {timeout:?} (held by process {holder_pid})
Tip: Use 'ps -p {holder_pid}' to check if process is running")]
    AcquisitionTimeout {
        path: PathBuf,
        holder_pid: ProcessId,
        timeout: Duration,
    },
}

impl FileLockError {
    /// Get detailed troubleshooting guidance
    pub fn help(&self) -> &'static str {
        match self {
            Self::AcquisitionTimeout { .. } => {
                "Lock Acquisition Timeout - Detailed Troubleshooting:

1. Check if holder process is running:
   Unix: ps -p <pid>
   Windows: tasklist /FI \"PID eq <pid>\"

2. If running: wait or increase timeout
3. If stuck: terminate process
4. If stale: report bug

See docs for more info."
            }
        }
    }
}
```

**Usage:**

```rust
// Basic: just show error
eprintln!("Error: {e}");

// Advanced: show help when needed
if verbose {
    eprintln!("\n{}", e.help());
}
```

**Pros:**

- ✅ Clean separation: error vs help
- ✅ No external infrastructure needed
- ✅ Help always available at runtime
- ✅ Users get help when they need it
- ✅ No version stability concerns
- ✅ Easy to maintain (help with error definition)
- ✅ Balances brevity with actionability
- ✅ Platform-specific guidance possible
- ✅ Aligns with all development principles

**Cons:**

- ⚠️ Help strings embedded in binary (small overhead)
- ⚠️ Still some duplication of publicly available info

**Decision**: ✅ **ACCEPTED** - Best balance of all considerations.

---

## 🎯 Final Decision

**We adopt the Tiered Help System approach (Option 5)** combining:

1. **Base error message**: Concise with essential context
2. **Brief tip**: One-liner actionable hint (from Option 3)
3. **`.help()` method**: Detailed troubleshooting on-demand
4. **Rustdoc**: Developer documentation (from Option 2)

### Implementation Pattern

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileLockError {
    /// Brief description for developers
    ///
    /// Longer context and when this error occurs.
    /// Use `.help()` for user-facing troubleshooting.
    #[error("Concise error with context
Tip: Brief actionable hint")]
    VariantName {
        // Error fields with #[source] where appropriate
    },
}

impl FileLockError {
    /// Get detailed troubleshooting guidance
    ///
    /// Returns platform-specific steps to resolve the error.
    pub fn help(&self) -> &'static str {
        match self {
            Self::VariantName { .. } => {
                "Detailed troubleshooting with:
1. Step-by-step instructions
2. Platform-specific commands
3. Multiple resolution approaches
4. Links to report bugs"
            }
        }
    }
}
```

## 📊 Comparison Matrix

| Criterion             | Error Codes         | Rustdoc Only | Brief Tips | External Refs | **Tiered Help**      |
| --------------------- | ------------------- | ------------ | ---------- | ------------- | -------------------- |
| **Brevity**           | ✅ Excellent        | ✅ Excellent | ✅ Good    | ✅ Good       | ✅ **Excellent**     |
| **Actionability**     | ✅ High             | ❌ Low       | ⚠️ Medium  | ✅ High       | ✅ **Very High**     |
| **Runtime Access**    | ❌ No               | ❌ No        | ✅ Yes     | ❌ Partial    | ✅ **Yes**           |
| **No Infrastructure** | ❌ Needs web        | ✅ Yes       | ✅ Yes     | ❌ Needs web  | ✅ **Yes**           |
| **Maintainability**   | ⚠️ Medium           | ✅ Easy      | ✅ Easy    | ⚠️ Medium     | ✅ **Easy**          |
| **Stability**         | ❌ Numbering issues | ✅ Stable    | ✅ Stable  | ⚠️ Link rot   | ✅ **Stable**        |
| **User Control**      | ❌ No               | ❌ No        | ❌ No      | ❌ No         | ✅ **Yes (verbose)** |

## 🎬 Implementation Guidelines

### For Error Definitions

1. Add Rustdoc comment explaining when/why error occurs
2. Use `#[error]` with concise message + brief tip
3. Include `#[source]` for underlying errors
4. Implement `.help()` method with detailed guidance

### For Application Code

```rust
// Let users control verbosity
match operation() {
    Ok(result) => { /* success */ }
    Err(e) => {
        eprintln!("Error: {e}");

        if args.verbose {
            eprintln!("\n{}", e.help());
        } else {
            eprintln!("\nRun with --verbose for detailed troubleshooting");
        }
    }
}
```

### For Testing

- Test error messages contain tips
- Test `.help()` returns non-empty strings for all variants
- Test help content includes key troubleshooting terms

## 📚 Related Documentation

- [Error Handling Guide](../contributing/error-handling.md) - Full implementation guidance
- [Development Principles](../development-principles.md) - Actionability principle
- Proposal #8 implementation details - See git history for `docs/refactors/file-lock-improvements.md` (completed October 3, 2025)

## 🔄 Future Considerations

If we find the tiered help system insufficient, we could:

1. **Generate HTML docs**: Build-time generation from `.help()` content
2. **Error catalog**: Maintain a searchable error database
3. **Telemetry integration**: Track which errors need better help

For now, the tiered help system provides the best balance of simplicity and effectiveness.

---

**Last Updated**: October 3, 2025  
**Review Date**: After Proposal #8 implementation and user feedback
