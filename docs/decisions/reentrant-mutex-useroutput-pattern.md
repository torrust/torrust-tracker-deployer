# Decision: Use ReentrantMutex Pattern for UserOutput Reentrancy

## Status

Accepted

## Date

2025-11-11

## Context

The Torrust Tracker Deployer application was experiencing reentrancy deadlocks with `Arc<std::sync::Mutex<UserOutput>>` when the same thread attempted to acquire the UserOutput lock multiple times. This occurred in scenarios where:

### The Deadlock Problem

In `CreateTemplateCommandController`, we encountered a critical deadlock where:

1. **Controller Level**: `display_success_and_guidance()` acquired the `UserOutput` mutex for error handling
2. **Progress Reporting**: While holding the lock, it called `self.progress.complete()`
3. **Same Thread Deadlock**: `ProgressReporter.complete()` tried to acquire the same mutex on the same thread
4. **Result**: Tests hung indefinitely requiring timeout mechanisms

### Problem Pattern

```rust
// Deadlock scenario with std::sync::Mutex
let user_output = Arc::new(Mutex::new(UserOutput::new(VerbosityLevel::Info)));

// First acquisition in controller
let lock1 = user_output.lock().unwrap();
// ... controller work that calls ProgressReporter ...

// Second acquisition in ProgressReporter (same thread)
let lock2 = user_output.lock().unwrap(); // ❌ DEADLOCK!
```

This pattern was particularly problematic because:

- Error handling and progress reporting are legitimate concurrent concerns
- Both need access to UserOutput for displaying messages to users
- The call stack naturally leads from controllers → progress reporters → user output
- Standard Rust mutexes are not reentrant

### GitHub Issue

This decision addresses [GitHub Issue #164](https://github.com/torrust/torrust-tracker-deployer/issues/164): "Eliminate potential deadlock with Arc<Mutex<UserOutput>> reentrancy"

## Decision

**Replace `Arc<std::sync::Mutex<UserOutput>>` with `Arc<parking_lot::ReentrantMutex<RefCell<UserOutput>>>` throughout the codebase.**

### Pattern Components

1. **`parking_lot::ReentrantMutex`**: Allows the same thread to acquire the lock multiple times safely
2. **`RefCell`**: Provides interior mutability since `ReentrantMutex<T>` only gives `&T` but UserOutput methods need `&mut self`
3. **`Arc`**: Maintains shared ownership across components

### New Pattern

```rust
use std::cell::RefCell;
use std::sync::Arc;
use parking_lot::ReentrantMutex;

// New pattern that eliminates deadlocks
let user_output = Arc::new(ReentrantMutex::new(RefCell::new(
    UserOutput::new(VerbosityLevel::Info)
)));

// First acquisition in controller
let lock1 = user_output.lock();
{
    let mut output1 = lock1.borrow_mut();
    output1.info("Controller processing...");
}

// Second acquisition in ProgressReporter (same thread) - NO DEADLOCK!
let lock2 = user_output.lock(); // ✅ Works correctly
{
    let mut output2 = lock2.borrow_mut();
    output2.progress("50% complete");
}
```

### Usage Pattern

```rust
// Standard usage in components
let lock = user_output.lock();
{
    let mut output = lock.borrow_mut();
    output.success("Operation completed");
    output.data(&format!("Details: {}", details));
} // RefCell borrow dropped here
// ReentrantMutex lock can be held longer if needed
```

## Consequences

### Positive

- **Eliminates Deadlocks**: Same-thread reentrancy is now safe and supported
- **Maintains Thread Safety**: Still protects UserOutput from concurrent access between different threads
- **Preserves API**: UserOutput methods continue to work with `&mut self` through RefCell
- **Clear Intent**: The type signature documents that reentrancy is expected and supported
- **Performance**: `parking_lot::ReentrantMutex` has similar performance characteristics to `std::sync::Mutex`
- **Minimal Changes**: UserOutput API remains unchanged, only synchronization mechanism updated

### Negative

- **Additional Dependency**: Introduces dependency on `parking_lot` crate
- **Runtime Borrow Checking**: RefCell uses runtime borrow checking instead of compile-time (though panics are highly unlikely in our usage pattern)
- **Slightly More Complex**: The type signature is more complex: `Arc<ReentrantMutex<RefCell<UserOutput>>>` vs `Arc<Mutex<UserOutput>>`
- **Learning Curve**: Developers need to understand the ReentrantMutex + RefCell pattern

### Migration Impact

- **Components Updated**: Container, ExecutionContext, all controllers, ProgressReporter
- **API Compatibility**: No changes to UserOutput public API - all methods continue to work identically
- **Test Verification**: Integration tests verify the deadlock scenario is resolved

### Thread Safety Limitations

The current solution using `RefCell` has important thread safety limitations:

- **Single-Thread Only**: `RefCell` does not implement `Sync`, making the pattern unsuitable for multi-threaded access
- **Same-Thread Reentrancy**: The solution only addresses reentrancy within a single thread, not concurrent access from multiple threads
- **Current Use Case**: This limitation is acceptable for the current codebase, which accesses `UserOutput` from a single thread during command execution

**Multi-Threading Alternative**: If future requirements need multi-threaded access to `UserOutput`, consider:

```rust
Arc<ReentrantMutex<Arc<Mutex<UserOutput>>>>
```

However, this would reintroduce the original deadlock problem and require a different architectural solution (e.g., message passing or separate UserOutput instances per thread).

## Alternatives Considered

### Option 1: Arc<RwLock<UserOutput>> (Rejected)

**Pros:**

- Standard library solution
- Multiple readers possible

**Cons:**

- Still not reentrant - would deadlock on write → write from same thread
- UserOutput methods need `&mut self`, so read access isn't useful
- More complex than needed for single-writer use case

### Option 2: Remove Mutex Entirely (Considered in Previous ADR)

**Pros:**

- Completely eliminates synchronization complexity
- Simple ownership patterns

**Cons:**

- Loses thread safety guarantees
- Requires extensive architectural changes
- Breaks shared UserOutput patterns needed for progress reporting
- Would require passing UserOutput explicitly through all layers

**Status**: This approach was documented in a previous ADR but ultimately rejected in favor of the ReentrantMutex solution.

### Option 3: Message-Passing Architecture (Rejected)

**Pros:**

- Completely eliminates shared mutable state
- Natural async boundaries

**Cons:**

- Massive architectural change requiring redesign of entire presentation layer
- Overkill for this specific reentrancy issue
- Would require rewriting all UserOutput usage patterns
- Significantly more complex than the focused ReentrantMutex solution

## Error Handling

RefCell will panic if:

- Attempting to borrow mutably while already borrowed mutably (same thread)
- Attempting to borrow mutably while borrowed immutably

In our usage pattern, this is extremely unlikely because:

- We only use mutable borrows (never immutable)
- Borrows are immediately dropped after UserOutput method calls
- No long-lived borrows across function calls

## Testing Strategy

The solution includes integration tests that verify:

1. Same-thread multiple acquisitions work without deadlock
2. RefCell interior mutability enables `&mut self` methods
3. Production code patterns work correctly

See `tests/user_output_reentrancy.rs` for complete test verification.

## Related Decisions

This decision **supersedes**:

- [Remove UserOutput Mutex for Simplified Architecture](./user-output-mutex-removal.md) - Marked as superseded by this ReentrantMutex approach

This decision enables:

- Safer progress reporting patterns
- More natural error handling in nested call stacks
- Future expansion of UserOutput usage without deadlock concerns

## References

- [GitHub Issue #164](https://github.com/torrust/torrust-tracker-deployer/issues/164)
- [parking_lot ReentrantMutex Documentation](https://docs.rs/parking_lot/latest/parking_lot/struct.ReentrantMutex.html)
- [RefCell Documentation](https://doc.rust-lang.org/std/cell/struct.RefCell.html)
- [Integration Test: tests/user_output_reentrancy.rs](../../tests/user_output_reentrancy.rs)
