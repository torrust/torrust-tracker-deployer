# Implementation Plan

> **📋 Roadmap**  
> Implementation plan for Environment State Management feature with detailed phase breakdowns.

## 🏗️ Implementation Phases Overview

### ✅ Phase 1: Foundation (COMPLETED)

**Goal**: Establish compile-time type-safe state management using the type-state pattern.

**Status**: ✅ All 3 subtasks completed

**Key Achievements**:

- 13 state marker types (9 success states + 4 error states)
- Generic `Environment<S>` with compile-time state validation
- Type-safe state transition methods
- 533 tests passing
- Full backward compatibility maintained

**📄 [View Detailed Phase 1 Plan →](./phase-1-foundation.md)**

**Commits**:

- `a7317f5` - feat: add state marker types for environment state machine
- `6b57708` - refactor: convert Environment to generic type-state struct
- `f8cd563` - feat: implement type-safe state transition methods

---

### ⏳ Phase 2: Serialization & Type Erasure (IN PROGRESS)

**Goal**: Enable runtime handling of typed states through type erasure for serialization and storage.

**Status**: ⏳ Planning complete, ready for implementation

**Key Deliverables**:

- `AnyEnvironmentState` enum for type erasure
- Bidirectional type conversion (typed ↔ erased)
- State introspection helper methods
- Full serialization/deserialization support

**📄 [View Detailed Phase 2 Plan →](./phase-2-serialization.md)**

**Subtasks**:

1. ⏳ Create `AnyEnvironmentState` enum
2. ⏳ Implement type conversion methods
3. ⏳ Add state introspection helpers

---

### 📅 Phase 3: Persistence (PLANNED)

**Goal**: Implement repository pattern for state persistence with atomic writes.

**Status**: 📅 Planned for future implementation

**Key Deliverables**:

- `StateRepository` trait for persistence operations
- JSON file-based repository implementation
- Atomic write operations (temp file + rename)
- Error handling for storage operations
- File locking for concurrent access safety

---

### 📅 Phase 4: Command Integration (PLANNED)

**Goal**: Update commands to use type-safe state transitions and orchestration.

**Status**: 📅 Planned for future implementation

**Key Deliverables**:

- Commands accept and return specific state types
- Type-safe state transitions in command execution
- Orchestration layer for chaining commands
- Error state handling with compile-time guarantees
- State persistence during command execution

---

### 📅 Phase 5: Testing & Documentation (ONGOING)

**Goal**: Comprehensive testing and documentation for all features.

**Status**: 🔄 Ongoing throughout all phases

**Key Deliverables**:

- Unit tests for all state machine operations
- Integration tests for repository operations
- E2E tests for command integration
- State recovery and error handling tests
- Updated architecture documentation
- Troubleshooting guides

---

## 📊 Progress Tracking

### Overall Progress

- ✅ Phase 1: Foundation - **100% Complete** (3/3 subtasks)
- ⏳ Phase 2: Serialization & Type Erasure - **0% Complete** (0/3 subtasks)
- 📅 Phase 3: Persistence - **Not Started**
- 📅 Phase 4: Command Integration - **Not Started**
- 🔄 Phase 5: Testing & Documentation - **Ongoing**

### Test Coverage

- **Current Tests**: 533 tests passing
- **Phase 1 Tests Added**: +15 tests
- **Expected Phase 2 Tests**: +100 tests
- **Target Total**: ~700+ tests

---

## 🔧 Technical Approach

### Type-State Pattern (Phase 1)

Use Rust's type system to encode state as a type parameter, making invalid state transitions impossible to compile:

```rust
// ✅ This compiles - valid transition
let env = Environment::new(name, creds);  // Environment<Created>
let env = env.start_provisioning();       // Environment<Provisioning>
let env = env.provisioned();              // Environment<Provisioned>

// ❌ This doesn't compile - invalid transition
let env = Environment::new(name, creds);
let env = env.configured();  // ERROR: method not found
```

### Type Erasure (Phase 2)

Use an enum to hold any typed `Environment<S>` at runtime for serialization and storage:

```rust
pub enum AnyEnvironmentState {
    Created(Environment<Created>),
    Provisioning(Environment<Provisioning>),
    // ... all 13 state variants
}

// Convert: typed → erased
let any_env = env.into_any();

// Convert: erased → typed
let env: Environment<Created> = any_env.try_into_created()?;
```

### Repository Pattern (Phase 3)

Persist type-erased environments to JSON files with atomic writes:

```rust
pub trait StateRepository {
    fn save(&self, env: &AnyEnvironmentState) -> Result<()>;
    fn load(&self, name: &EnvironmentName) -> Result<Option<AnyEnvironmentState>>;
}
```

### Command Integration (Phase 4)

Commands enforce correct state types at compile time:

```rust
// Configure command only accepts Provisioned environments
impl ConfigureCommand {
    pub async fn execute(
        &self,
        environment: Environment<Provisioned>
    ) -> Result<Environment<Configured>, ConfigureError> {
        // ...
    }
}
```

---

## 📚 Related Documentation

- [Feature Specification](../README.md) - Overall feature goals and motivation
- [Phase 1 Details](./phase-1-foundation.md) - Type-state pattern implementation
- [Phase 2 Details](./phase-2-serialization.md) - Serialization & type erasure
- [Error Handling Guide](../../../contributing/error-handling.md) - Error handling principles
- [Testing Conventions](../../../contributing/testing.md) - Testing best practices

---

## 🚀 Getting Started

### For Phase 1 Review

Phase 1 is complete. Review the implementation:

```bash
# View Phase 1 commits
git log --oneline a7317f5..f8cd563

# Run tests
cargo test

# Run linters
cargo run --bin linter all
```

### For Phase 2 Implementation

Phase 2 is ready to start. Follow the detailed plan:

1. Read [Phase 2 Plan](./phase-2-serialization.md)
2. Start with Subtask 1: Create `AnyEnvironmentState` enum
3. Test, lint, and commit after each subtask
4. Verify all existing tests continue to pass

---

## 🎯 Success Criteria

### Phase Completion

Each phase is considered complete when:

- ✅ All subtasks are implemented and tested
- ✅ All linters pass (`cargo run --bin linter all`)
- ✅ All tests pass (`cargo test`)
- ✅ Backward compatibility is maintained
- ✅ Documentation is updated
- ✅ Changes are committed with conventional commit messages

### Feature Completion

The entire feature is considered complete when:

- ✅ All 5 phases are complete
- ✅ E2E tests demonstrate end-to-end state management
- ✅ Commands integrate with typed states
- ✅ State persistence works reliably
- ✅ Documentation is comprehensive
- ✅ User feedback is incorporated
