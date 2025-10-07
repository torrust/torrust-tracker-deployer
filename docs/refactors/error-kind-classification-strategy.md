# Refactor Plan: Error Kind Classification Strategy

## 📅 Date

2025-10-07

## 🎯 Goal

Simplify and unify error kind classification across commands by moving error kind determination into the error types themselves through the `Traceable` trait, eliminating manual classification in command code.

---

## 📖 Context

### Original Motivation for Error Kinds

Error kinds were introduced in commit `635ef9e` (October 6, 2025) as part of the structured error context refactor. The **primary purpose** was to provide an easy way to understand what type of error occurred in an environment **without having to parse detailed trace log files**.

From the user's feedback:

> "I think the reason we introduced those errors is because we wanted to have an easy way to know what was the last error that occurred in the environments without having to parse trace log files."

This aligns with the Phase 5 command integration goals:

- **State Visibility**: Track command progress and detect interrupted operations
- **Error Recovery Guidance**: Know exactly which step failed and provide actionable advice
- **Quick Error Classification**: At-a-glance understanding of failure type for debugging

Error kinds serve as a **high-level summary** in the failure context that can be:

- Displayed to users without technical details
- Used for filtering/grouping errors
- Foundation for future retry/recovery strategies based on error category

### Current Architecture

**Error Flow:**

1. Command executes steps and captures errors
2. On error, command calls `build_failure_context()`
3. `build_failure_context()` manually classifies error using pattern matching:

```rust
// In ProvisionCommand::build_failure_context()
let error_kind = match error {
    ProvisionCommandError::OpenTofuTemplateRendering(_)
    | ProvisionCommandError::AnsibleTemplateRendering(_) => {
        ProvisionErrorKind::TemplateRendering
    }
    ProvisionCommandError::OpenTofu(_) => ProvisionErrorKind::InfrastructureProvisioning,
    ProvisionCommandError::SshConnectivity(_) => ProvisionErrorKind::NetworkConnectivity,
    ProvisionCommandError::Command(_) => ProvisionErrorKind::ConfigurationTimeout,
};
```

**Existing Components:**

- **Traceable Trait** (`src/shared/error/traceable.rs`):
  - `trace_format()`: Formats error for trace files
  - `trace_source()`: Provides source error for chain traversal
- **Error Kinds per Command**:

  - `ProvisionErrorKind`: 4 variants (TemplateRendering, InfrastructureProvisioning, NetworkConnectivity, ConfigurationTimeout)
  - `ConfigureErrorKind`: 2 variants (InstallationFailed, CommandExecutionFailed)

- **Error Context**:
  - `ProvisionFailureContext`: Contains `failed_step` and `error_kind`
  - `ConfigureFailureContext`: Contains `failed_step` and `error_kind`
- **Usage Locations**:
  - Commands: Classification in `build_failure_context()`
  - Trace Files: Written to trace files with `Error Kind: {:?}`
  - State Persistence: Serialized in environment state JSON
  - Tests: Validated in unit tests

### The Problem

1. **Manual Classification**: Each command must manually map error types to error kinds
2. **Duplication**: Classification logic duplicated in every command
3. **Command-Specific Kinds**: Each command defines its own `ErrorKind` enum with different variants
4. **Maintenance Burden**: Adding new error types requires updating classification logic
5. **Inconsistency Risk**: Different commands might classify similar errors differently
6. **Unclear Purpose**: The actual purpose and audience of error kinds is ambiguous

### Purpose Analysis

**Where Error Kinds Are Used:**

1. **Trace Files**: Written as `Error Kind: {:?}` (debug format) - for debugging
2. **Environment State**: Serialized in `ProvisionFailureContext` and `ConfigureFailureContext`
3. **Tests**: Asserted in unit tests to verify correct classification
4. **Not Used**: No evidence of user-facing display or CLI output using error kinds

**Inferred Purpose:**

- **High-level categorization** for debugging and error analysis
- **Post-mortem analysis** via trace files
- **Test validation** to ensure errors are properly categorized
- **Future feature foundation** (possibly retry logic, recovery strategies)

**Not for:**

- Direct user-facing error messages (those use `error.to_string()`)
- CLI output or interactive error handling (no evidence found)

---

## 🤔 Analysis

### Key Questions

1. **Should error kinds be generic or command-specific?**

   - Command-specific: More precise but requires separate enums per command
   - Generic: One enum for all but may not capture command-specific nuances

2. **What granularity is appropriate?**

   - Too broad: "Failed" (not useful)
   - Too narrow: Duplicates error variants (redundant)
   - Right level: Categories that group related failures for recovery strategies

3. **Who is the audience for error kinds?**

   - Developers debugging issues (via trace files)
   - Automated systems (future retry/recovery logic)
   - Not end users (they see the full error message)

4. **Should error types determine their own kind?**
   - Yes: Each error knows its category (self-describing)
   - No: Commands classify based on context (current approach)

### Design Considerations

#### Consideration 1: Generic vs. Command-Specific Kinds

##### Option A: Keep Command-Specific Enums

- ✅ Precise categorization per command domain
- ✅ Clear separation of concerns
- ❌ Duplication across commands
- ❌ Different vocabularies for similar concepts

##### Option B: Unified Generic Enum

- ✅ Consistent categorization across all commands
- ✅ No duplication
- ✅ Simpler to understand
- ❌ May not fit all command types perfectly
- ❌ Could become too broad or too narrow

#### Consideration 2: Error Kind Location

##### Current: Classified by Command

```rust
// Command determines kind based on error type
let error_kind = match error {
    CommandError::Type1(_) => ErrorKind::Category1,
    CommandError::Type2(_) => ErrorKind::Category2,
};
```

##### Proposed: Self-Describing Errors

```rust
// Error knows its own kind
trait Traceable {
    fn error_kind(&self) -> ErrorKind;
}
```

#### Consideration 3: Granularity Analysis

##### Current Provision Kinds

- `TemplateRendering`: Clear, specific to templating operations
- `InfrastructureProvisioning`: Infrastructure-related (OpenTofu operations)
- `NetworkConnectivity`: Network and connectivity issues
- `ConfigurationTimeout`: Timeouts and delays (somewhat unclear - actually maps to Command errors)

##### Current Configure Kinds

- `InstallationFailed`: Software installation errors
- `CommandExecutionFailed`: Generic command failures (never actually used in code!)

##### Observations

- Some overlap potential: "CommandExecutionFailed" could apply to both commands
- "ConfigurationTimeout" doesn't accurately describe command failures
- Two variants in `ConfigureErrorKind` but only one is used (`InstallationFailed`)

---

## 💡 Proposals

### Proposal 1: Generic Error Kinds with Traceable Integration (RECOMMENDED)

**Description**: Create a single, unified `ErrorKind` enum that covers all commands and add a `error_kind()` method to the `Traceable` trait.

**Implementation:**

```rust
// In src/shared/error/traceable.rs or src/shared/error/kind.rs

/// Generic error categories for command failures
///
/// These categories provide high-level classification for debugging
/// and potential recovery strategies. They are used in trace files
/// and failure context but are not directly user-facing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    /// Template rendering or generation failed
    TemplateRendering,

    /// Infrastructure operations failed (OpenTofu, LXD, etc.)
    InfrastructureOperation,

    /// Network connectivity or communication failed
    NetworkConnectivity,

    /// External tool or command execution failed
    CommandExecution,

    /// Timeout or deadline exceeded
    Timeout,

    /// File system or I/O operation failed
    FileSystem,

    /// Configuration validation or parsing failed
    Configuration,
}

pub trait Traceable: std::error::Error {
    fn trace_format(&self) -> String;
    fn trace_source(&self) -> Option<&dyn Traceable>;

    /// Get the error kind for categorization
    ///
    /// Returns a high-level category for this error, used in trace files
    /// and failure context for debugging and potential recovery strategies.
    fn error_kind(&self) -> ErrorKind;
}
```

**Changes Required:**

1. Remove `ProvisionErrorKind` and `ConfigureErrorKind` enums
2. Add `error_kind()` to `Traceable` trait
3. Implement `error_kind()` for all error types:
   - `ProvisionCommandError`
   - `ConfigureCommandError`
   - `OpenTofuError`
   - `CommandError`
   - `SshError`
   - etc.
4. Update failure contexts to use generic `ErrorKind`
5. Remove classification logic from `build_failure_context()` methods
6. Update trace writers to use generic `ErrorKind`
7. Update all tests

**Example Implementation:**

```rust
impl Traceable for ProvisionCommandError {
    fn trace_format(&self) -> String {
        // ... existing code ...
    }

    fn trace_source(&self) -> Option<&dyn Traceable> {
        // ... existing code ...
    }

    fn error_kind(&self) -> ErrorKind {
        match self {
            Self::OpenTofuTemplateRendering(_)
            | Self::AnsibleTemplateRendering(_) => ErrorKind::TemplateRendering,
            Self::OpenTofu(_) => ErrorKind::InfrastructureOperation,
            Self::SshConnectivity(_) => ErrorKind::NetworkConnectivity,
            Self::Command(_) => ErrorKind::CommandExecution,
        }
    }
}
```

**Benefits:**

- ✅ Errors self-describe their category
- ✅ No manual classification in commands
- ✅ Consistent categorization across all commands
- ✅ Errors delegate to source when appropriate
- ✅ Easy to add new commands without new enums
- ✅ Simple to maintain and extend

**Drawbacks:**

- ⚠️ Generic categories might not capture command-specific nuances
- ⚠️ Breaking change to existing state JSON (migration needed)

---

### Proposal 2: Keep Command-Specific Kinds, Add to Traceable (Alternative)

**Description**: Keep separate `ProvisionErrorKind` and `ConfigureErrorKind` enums but add a `error_kind()` method to `Traceable` that returns a command-specific kind.

**Implementation:**

```rust
pub trait Traceable: std::error::Error {
    fn trace_format(&self) -> String;
    fn trace_source(&self) -> Option<&dyn Traceable>;

    /// Get the error kind as a generic trait object
    ///
    /// Returns a trait object that can be downcasted to specific
    /// error kind enums per command.
    fn error_kind(&self) -> Box<dyn std::any::Any>;
}

impl Traceable for ProvisionCommandError {
    fn error_kind(&self) -> Box<dyn std::any::Any> {
        let kind: ProvisionErrorKind = match self {
            Self::OpenTofuTemplateRendering(_)
            | Self::AnsibleTemplateRendering(_) => {
                ProvisionErrorKind::TemplateRendering
            }
            // ... other variants ...
        };
        Box::new(kind)
    }
}
```

**Benefits:**

- ✅ Maintains precise command-specific categorization
- ✅ No breaking changes to existing enums
- ✅ Errors self-describe their category

**Drawbacks:**

- ❌ Type erasure with `Any` is complex and unergonomic
- ❌ Loss of type safety
- ❌ Commands still need command-specific error kind enums
- ❌ More complex to work with
- ❌ Duplication across commands continues

---

### Proposal 3: Error Kinds as Associated Types (Alternative)

**Description**: Make error kinds an associated type on commands or error types.

**Implementation:**

```rust
pub trait Traceable: std::error::Error {
    type Kind: Debug + Clone + Copy + Serialize;

    fn trace_format(&self) -> String;
    fn trace_source(&self) -> Option<&dyn Traceable>;
    fn error_kind(&self) -> Self::Kind;
}

impl Traceable for ProvisionCommandError {
    type Kind = ProvisionErrorKind;

    fn error_kind(&self) -> Self::Kind {
        match self {
            // ... classification ...
        }
    }
}
```

**Benefits:**

- ✅ Type-safe per error type
- ✅ Flexible per command
- ✅ Errors self-describe

**Drawbacks:**

- ❌ Makes `Traceable` not object-safe (can't use `dyn Traceable`)
- ❌ Complex type system implications
- ❌ Breaks existing trace file infrastructure
- ❌ Not worth the complexity for this use case

---

### Proposal 4: No Error Kinds - Remove Entirely (Radical Alternative)

**Description**: Remove error kinds entirely since they're only used in trace files and don't provide significant value over the error type itself.

**Rationale:**

- Trace files already include full error chain with types
- Error kinds are debug-formatted (`{:?}`), not user-facing
- `failed_step` already provides high-level context
- Full error message provides detailed context

**Changes:**

1. Remove `ProvisionErrorKind` and `ConfigureErrorKind`
2. Remove `error_kind` field from failure contexts
3. Update trace files to show error type name instead
4. Update tests to not check error kinds

**Benefits:**

- ✅ Simpler architecture
- ✅ Less code to maintain
- ✅ No classification needed
- ✅ One less thing to keep in sync

**Drawbacks:**

- ❌ Loses high-level categorization
- ❌ Harder to build recovery strategies later
- ❌ Breaking change to state JSON
- ❌ May need to add back later for features like retry

---

## 🎯 Final Recommendation

### Adopt Proposal 1: Generic Error Kinds with Traceable Integration

#### Rationale

1. **Simplicity**: Single `ErrorKind` enum is easier to understand and maintain
2. **Self-Describing Errors**: Errors know their own category, eliminating manual classification
3. **Consistency**: All commands use the same vocabulary for error categories
4. **Extensibility**: Easy to add new commands without defining new error kind enums
5. **Foundation for Features**: Provides categorization for future retry/recovery logic
6. **Clean Integration**: Natural fit with existing `Traceable` trait

#### Implementation Phases

##### Phase 1: Add Generic ErrorKind Enum

**Goal**: Create unified error kind classification

**Tasks**:

1. Create `src/shared/error/kind.rs` with generic `ErrorKind` enum
2. Export from `src/shared/error/mod.rs`
3. Document each variant's purpose and usage

**Files**:

- New: `src/shared/error/kind.rs`
- Modified: `src/shared/error/mod.rs`

##### Phase 2: Extend Traceable Trait

**Goal**: Add `error_kind()` method to trait

**Tasks**:

1. Add `error_kind()` method signature to `Traceable` trait
2. Document the method's purpose and usage
3. Update trait documentation

**Files**:

- Modified: `src/shared/error/traceable.rs`

##### Phase 3: Implement error_kind() for All Errors

**Goal**: Make all errors self-describing

**Tasks**:

1. Implement `error_kind()` for `ProvisionCommandError`
2. Implement `error_kind()` for `ConfigureCommandError`
3. Implement `error_kind()` for `OpenTofuError`
4. Implement `error_kind()` for `CommandError`
5. Implement `error_kind()` for `SshError`
6. Implement for other error types as needed

**Files**:

- Modified: `src/application/commands/provision.rs`
- Modified: `src/application/commands/configure.rs`
- Modified: `src/infrastructure/external_tools/tofu/adapter/client.rs`
- Modified: `src/shared/command/error.rs`
- Modified: `src/shared/ssh/error.rs` (if exists)
- Modified: Other error implementation files

##### Phase 4: Update Failure Contexts

**Goal**: Use generic `ErrorKind` in contexts

**Tasks**:

1. Replace `ProvisionErrorKind` with generic `ErrorKind` in `ProvisionFailureContext`
2. Replace `ConfigureErrorKind` with generic `ErrorKind` in `ConfigureFailureContext`
3. Update serialization/deserialization
4. Update all context builders

**Files**:

- Modified: `src/domain/environment/state/provision_failed.rs`
- Modified: `src/domain/environment/state/configure_failed.rs`
- Modified: `src/domain/environment/state/mod.rs`

##### Phase 5: Update Commands

**Goal**: Remove manual classification from commands

**Tasks**:

1. Update `ProvisionCommand::build_failure_context()` to call `error.error_kind()`
2. Update `ConfigureCommand::build_failure_context()` to call `error.error_kind()`
3. Remove pattern matching for error kind classification

**Files**:

- Modified: `src/application/commands/provision.rs`
- Modified: `src/application/commands/configure.rs`

##### Phase 6: Update Trace Writers

**Goal**: Use generic `ErrorKind` in trace files

**Tasks**:

1. Update `ProvisionTraceWriter` imports and types
2. Update `ConfigureTraceWriter` imports and types
3. Verify trace file format still works

**Files**:

- Modified: `src/infrastructure/trace/writer/commands/provision.rs`
- Modified: `src/infrastructure/trace/writer/commands/configure.rs`

##### Phase 7: Update Tests

**Goal**: Fix all broken tests

**Tasks**:

1. Update provision command tests to use generic `ErrorKind`
2. Update configure command tests to use generic `ErrorKind`
3. Update trace writer tests
4. Update state serialization tests
5. Run full test suite and fix any remaining issues

**Files**:

- Modified: `src/application/commands/provision.rs` (test module)
- Modified: `src/application/commands/configure.rs` (test module)
- Modified: `src/infrastructure/trace/writer/commands/provision.rs` (test module)
- Modified: `src/infrastructure/trace/writer/commands/configure.rs` (test module)
- Modified: State test files

##### Phase 8: Remove Old Enums

**Goal**: Clean up deprecated code

**Tasks**:

1. Remove `ProvisionErrorKind` enum from `provision_failed.rs`
2. Remove `ConfigureErrorKind` enum from `configure_failed.rs`
3. Remove any remaining references
4. Update documentation

**Files**:

- Modified: `src/domain/environment/state/provision_failed.rs`
- Modified: `src/domain/environment/state/configure_failed.rs`
- Modified: `docs/refactors/` (update this document with results)

---

## ⚠️ Migration Considerations

### Breaking Changes

1. **State JSON Format**: The `error_kind` field in serialized states will change

   - Old: `"error_kind": "TemplateRendering"` (command-specific enum)
   - New: `"error_kind": "TemplateRendering"` (generic enum)

2. **Enum Variant Names**: Some variants will change:
   - `InfrastructureProvisioning` → `InfrastructureOperation`
   - `ConfigurationTimeout` → `Timeout` or `CommandExecution`

### Compatibility Strategy

#### Option A: Accept Breaking Change (PoC Phase)

- Since this is a proof-of-concept with no production users
- No migration needed
- Clean slate with new design

#### Option B: Maintain Backward Compatibility

- Add custom deserializer that maps old variants to new ones
- More complex but preserves existing state files

**Recommendation**: Option A (accept breaking change) since we're in PoC phase.

---

## 📊 Success Criteria

- [ ] Single `ErrorKind` enum used across all commands
- [ ] All error types implement `error_kind()` method
- [ ] No manual classification logic in commands
- [ ] All tests pass
- [ ] Trace files still generate correctly
- [ ] State serialization/deserialization works
- [ ] Documentation updated
- [ ] Code is cleaner and easier to maintain

---

## 📚 Discarded Alternatives Summary

| Proposal                               | Reason for Rejection                                               |
| -------------------------------------- | ------------------------------------------------------------------ |
| **Proposal 2**: Command-specific + Any | Type erasure too complex, loses type safety, still has duplication |
| **Proposal 3**: Associated types       | Makes Traceable non-object-safe, breaks existing infrastructure    |
| **Proposal 4**: Remove kinds entirely  | Loses useful categorization for future features like retry logic   |

---

## � Implementation Progress

### Status: 🚧 IN PROGRESS

**Started**: October 7, 2025  
**Target Completion**: TBD

### Phase Completion Tracker

| Phase                            | Status      | Commit | Tests | Notes                                                                 |
| -------------------------------- | ----------- | ------ | ----- | --------------------------------------------------------------------- |
| Phase 1: Generic ErrorKind Enum  | ✅ Complete | -      | 758   | Created src/shared/error/kind.rs with 7 variants, 5 tests             |
| Phase 2: Extend Traceable Trait  | ✅ Complete | -      | 758   | Added error_kind() method to Traceable trait                          |
| Phase 3: Implement error_kind()  | ✅ Complete | -      | 758   | All 9 error types + 3 test errors now have error_kind()               |
| Phase 4: Update Failure Contexts | ✅ Complete | -      | 758   | Replaced ProvisionErrorKind/ConfigureErrorKind with generic ErrorKind |
| Phase 5: Update Commands         | ✅ Complete | -      | 758   | Commands now call error.error_kind() instead of manual classification |
| Phase 6-8: Cleanup               | ✅ Complete | -      | 758   | Updated all tests, trace writers, removed old enum exports            |

**Legend**: ⏳ Not Started | 🚧 In Progress | ✅ Complete

**Note**: Phases 6-8 were combined as they were all part of updating tests and removing references to old enums. All work completed in single pass.

### Test Count Tracking

- **Baseline**: 753 tests passing (before refactor)
- **Current**: 758 tests passing (Phases 1-5 complete)
- **Target**: All tests passing with updated error kinds ✅

---

## �🔗 Related Documentation

- [Error Handling Guide](../contributing/error-handling.md) - Error handling principles
- [Traceable Trait](../../src/shared/error/traceable.rs) - Current trait definition
- [Step Tracking Refactor](./step-tracking-for-failure-context.md) - Recent related refactor
- [Development Principles](../development-principles.md) - Observability and user friendliness
- [Phase 5 Command Integration](../features/environment-state-management/implementation-plan/phase-5-command-integration.md) - Original error context implementation

---

## 📝 Notes

- This refactor builds on the recent step tracking refactor
- The combination of direct step tracking + self-describing error kinds eliminates all reverse engineering in commands
- Future commands will automatically integrate by implementing `Traceable::error_kind()`
- Error kinds are for debugging/categorization, not user-facing messages
- This lays foundation for future retry/recovery strategies based on error category
- Error kinds provide quick classification without parsing detailed trace files (original motivation)
