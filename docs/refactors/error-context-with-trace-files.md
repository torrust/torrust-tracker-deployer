# Refactor: Error Context with Trace Files

## 📋 Overview

This refactor improves error handling in commands by replacing string-based error context with structured, type-safe context and independent trace files.

**Key Insight**: Instead of forcing errors to implement `Serialize`, we create a custom `Traceable` trait for error formatting. This decouples error types from serialization constraints.

**Scope**: This refactor focuses on the currently active commands: `provision` and `configure`. The `release` and `run` commands (and their corresponding failed states) will be addressed when those commands are implemented.

## 🎯 Goals

1. **Type Safety**: Replace `String` with enums for pattern matching
2. **Complete Information**: Capture full error chains via `Traceable` trait
3. **Independent Traces**: Decouple trace generation from state management
4. **Error History**: Preserve all failure attempts
5. **User Actionability**: Provide detailed troubleshooting guidance (basic implementation first)

## 🚫 Non-Goals (For This Refactor)

1. **Backward Compatibility**: Not needed - not used in production yet
2. **Suggested Actions**: Future improvement - focus on basic trace implementation first
3. **Automatic Cleanup**: Not needed - entire data directory is removed after successful tracker installation
4. **Release/Run Commands**: Not implemented yet - will be addressed when those commands are added

## ❌ Current Problems

### What We Have Now

**In `ProvisionFailed` state**:

```rust
pub struct ProvisionFailed {
    failed_step: String,  // e.g., "render_opentofu_templates"
}
```

**In command**:

```rust
fn extract_failed_step(&self, error: &ProvisionCommandError) -> String {
    match error {
        ProvisionCommandError::OpenTofuTemplateRendering(_) => {
            "render_opentofu_templates".to_string()
        }
        // ... more string conversions
    }
}
```

**Problems**:

- ❌ String-based (typo-prone, no compile-time safety)
- ❌ No error details beyond step name
- ❌ No error history
- ❌ Limited actionability

## 🚀 Target Implementation

### New Structure

**Error Context (Serializable, stored in state)**:

```rust
use std::path::PathBuf;
use uuid::Uuid;

/// Unique identifier for error traces (newtype pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceId(Uuid);

impl TraceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionFailureContext {
    /// Which step failed (enum for pattern matching)
    pub failed_step: ProvisionStep,

    /// Error category (enum for type-safe handling)
    pub error_kind: ProvisionErrorKind,

    /// Human-readable summary
    pub error_summary: String,

    /// When the failure occurred
    pub failed_at: chrono::DateTime<chrono::Utc>,

    /// Timing information
    pub execution_started_at: chrono::DateTime<chrono::Utc>,
    pub execution_duration: std::time::Duration,

    /// Unique trace identifier (UUID)
    pub trace_id: TraceId,

    /// Path to the trace file (independent of state)
    pub trace_file_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvisionStep {
    RenderTemplates,
    OpenTofuInit,
    OpenTofuApply,
    WaitSshConnectivity,
    // ... more steps
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvisionErrorKind {
    TemplateRendering,
    InfrastructureProvisioning,
    NetworkConnectivity,
    ConfigurationTimeout,
}
```

**Trace Files (Complete error chain, independent of state)**:

```text
data/e2e-full/
├── state.json                            # Current state with context
└── traces/                               # Error trace history (failure-only)
    ├── 20251003-103045-provision.log     # Provision failed - First attempt
    ├── 20251003-105230-provision.log     # Provision failed - Retry #1
    ├── 20251003-110015-configure.log     # Configure failed - First attempt
    └── 20251003-111200-provision.log     # Provision failed - Retry #2
```

**Note**: Trace files are:

- **Location**: `data/{env}/traces/` (not `build/` - data contains internal state, build contains generated artifacts)
- **Naming**: `{timestamp}-{command}.log` (timestamp first for easy chronological sorting)
- **Generation**: **Only when commands fail** (not on success)
- **Independence**: Separate from state management (separate repository/concern)
- **Universal**: Any command can generate traces (provision, configure, deploy, etc.)

**Trace File Format (Basic Implementation)**:

```text
=== Provision Command Failed ===
Timestamp: 2025-10-03T10:30:45.123Z
Trace ID: 550e8400-e29b-41d4-a716-446655440000
Failed Step: OpenTofuApply
Error Kind: InfrastructureProvisioning
Error Summary: Infrastructure provisioning failed

=== Error Chain ===

1. ProvisionCommandError: Infrastructure provisioning failed
   Location: src/application/commands/provision.rs:145
   Context: Environment 'e2e-full', step 'opentofu_apply'

2. OpenTofuError: OpenTofu apply failed with exit code 1
   Location: src/infrastructure/opentofu/client.rs:89
   Command: tofu apply -auto-approve
   Working Dir: /path/to/build/e2e-full/tofu

3. CommandExecutionError: Process exited with non-zero status
   Location: src/infrastructure/command/executor.rs:56
   Exit Code: 1
   Stderr: Error creating instance: Quota 'INSTANCES' exceeded

=== Future Enhancements ===

Suggested actions section will be added in a future iteration.
```

**Note**: The "Suggested Actions" section is planned for future implementation to keep this refactor focused.

### The `Traceable` Trait

**Key innovation**: Custom trait for error formatting (not `Serialize`):

```rust
/// Trait for errors that can generate detailed traces
pub trait Traceable {
    /// Generate a formatted trace entry for this error
    fn trace_format(&self) -> String;

    /// Get the underlying source error, if any
    fn trace_source(&self) -> Option<&dyn Traceable>;
}
```

**Benefits**:

- ✅ No `Serialize` constraints on error types
- ✅ Custom formatting per error type
- ✅ Errors can contain non-serializable data
- ✅ Independent of state management

## 📝 Implementation Tasks

### Task 1: Define Error Context Types and Traceable Trait

**Files**:

- `src/domain/environment/state/error_context.rs` (new)
- `src/shared/error/traceable.rs` (new)

**Steps**:

1. Create `ProvisionFailureContext` and `ConfigureFailureContext` structs (separate per command)
2. Create `ProvisionStep`, `ConfigureStep`, `ProvisionErrorKind`, and `ConfigureErrorKind` enums
3. Define `Traceable` trait
4. Implement `Traceable` for provision and configure command errors
5. Add tests for context serialization

**Note**: Separate context structs per command because they have command-specific enums (`ProvisionStep` vs `ConfigureStep`). Common fields are shared but type-specific fields require separate structs.

**Estimated**: 3 hours

### Task 2: Update Failed States

**Files**:

- `src/domain/environment/state/provision_failed.rs`
- `src/domain/environment/state/configure_failed.rs`

**Steps**:

1. Replace `String` field with `ProvisionFailureContext` / `ConfigureFailureContext`
2. Update transition methods to accept context
3. Update serialization tests

**Note**: No backward compatibility needed - not used in production yet.

**Estimated**: 1 hour

### Task 3: Implement Trace File Writer

**Files**:

- `src/infrastructure/trace/writer.rs` (new)
- `src/infrastructure/trace/repository.rs` (new)

**Steps**:

1. Create `TraceWriter` that walks error chain via `Traceable`
2. Implement trace file naming: `{timestamp}-{command}.log`
3. Create trace directory: `data/{env}/traces/`
4. Format trace with error chain and context (no suggested actions yet)
5. Add tests for trace generation

**Note**: Basic implementation without suggested actions section. That will be a future enhancement.

**Estimated**: 2.5 hours

### Task 4: Update Commands to Generate Traces

**Files**:

- `src/application/commands/provision.rs`
- `src/application/commands/configure.rs`

**Steps**:

1. Replace `extract_failed_step()` with context builder
2. Add trace generation on failure
3. Store trace file reference in context
4. Update error handling to use enums
5. Add integration tests

**Estimated**: 3 hours

### Task 5: Documentation and Examples

**Files**:

- `docs/contributing/error-handling.md`

**Steps**:

1. Document `Traceable` trait usage
2. Add examples of trace file reading
3. Update error handling guide

**Estimated**: 0.5 hours

## ⏱️ Total Estimated Effort

**10 hours** (~1.5 days of focused work)

**Reduced from original 13 hours due to**:

- No backward compatibility needed (saves 2 hours)
- No suggested actions in traces yet (saves 0.5 hours)
- Focused on provision/configure only (saves 0.5 hours)

## ✅ Success Criteria

- [ ] Error contexts use enums for provision and configure commands
- [ ] Trace files generated **only** on command failure
- [ ] Trace files stored in `data/{env}/traces/`
- [ ] Trace files named `{timestamp}-{command}.log`
- [ ] `Traceable` trait implemented for provision and configure errors
- [ ] Complete error chains captured in trace files
- [ ] Basic trace format (without suggested actions)
- [ ] All tests pass (703+)
- [ ] Error handling documentation updated

## 🔄 Implementation Path

1. **Current Phase 5**: String-based context (working, tested)
2. **Implement on main branch**: Direct implementation (no branch needed for internal refactor)
3. **Implement in stages**: One task at a time
4. **Test thoroughly**: Unit + integration + E2E after each task
5. **Commit**: Single commit with all changes after verification

**Note**: No backward compatibility or cleanup needed. This is a breaking change to internal state representation, but acceptable since not in production.

## 📚 Related Documentation

- [Error Handling Guide](../contributing/error-handling.md)
- [ADR: Error Context Strategy](../decisions/error-context-strategy.md)

## 🎯 Benefits

### Before (Current)

```rust
// Limited information
match failed_env.state().failed_step().as_str() {
    "opentofu_apply" => { /* ... */ }  // String matching, typo-prone
    _ => {}
}
```

### After (Refactored)

```rust
// Rich, type-safe information
match failed_env.state().context() {
    ProvisionFailureContext {
        failed_step: ProvisionStep::OpenTofuApply,
        error_kind: ProvisionErrorKind::InfrastructureProvisioning,
        trace_file_path: Some(ref path),
        ..
    } => {
        println!("Provisioning failed at OpenTofu apply step");
        println!("Error kind: Infrastructure provisioning");
        println!("Trace ID: {}", context.trace_id.0);
        println!("Full trace: {}", path.display());

        // Type-safe, compile-time checked!
    }
}
```

**Key Improvements**:

- ✅ Compile-time safety (enums, not strings)
- ✅ Complete error information (via `Traceable` trait)
- ✅ Error history preserved (multiple trace files)
- ✅ No serialization constraints (custom `Traceable` trait)
- ✅ Better user experience (detailed guidance)
- ✅ Independent trace management (separate concern)
- ✅ Proper storage location (`data/`, not `build/`)
- ✅ Chronological organization (timestamp-first naming)
- ✅ Failure-focused (traces only when needed)
