# Decision: Remove UserOutput Mutex for Simplified Architecture

## Status

Accepted

## Date

2025-11-11

## Context

The current `UserOutput` implementation uses `Arc<Mutex<UserOutput>>` throughout the codebase to prevent mixed output during operations. However, this approach has introduced several significant problems:

### Deadlock Issues

We recently encountered a critical reentrancy deadlock in `CreateTemplateCommandController` where:

1. `display_success_and_guidance()` acquired the `UserOutput` mutex
2. While holding the lock, it called `self.progress.complete()`
3. `ProgressReporter.complete()` attempted to acquire the same mutex again
4. **Deadlock**: Tests hung indefinitely requiring timeout mechanisms

This pattern is fragile and could easily recur as the codebase grows, especially when using `UserOutput` through multiple wrapper types (like `ProgressReporter`).

### Application Characteristics

The Torrust Tracker Deployer has specific characteristics that make complex concurrency unnecessary:

- **Single-user tool**: Intended for individual developers deploying on their machines
- **Short execution time**: Total deployment time < 5 minutes
- **Sequential operations**: Most steps must execute sequentially (provision → configure → deploy)
- **Error recovery priority**: Clear error states more important than minor performance gains

### Current Complexity vs Benefits

The mutex introduces significant complexity:

- Timeout mechanisms needed to prevent deadlocks
- Complex error handling for mutex poisoning
- Difficult debugging when deadlocks occur
- `Arc<Mutex<>>` patterns throughout the codebase
- Multiple access paths to the same mutex (direct + through wrappers)

The benefits (preventing mixed output) don't justify this complexity for a sequential, single-user deployment tool.

## Decision

**Remove the `Arc<Mutex<UserOutput>>` pattern entirely and use direct ownership/borrowing of `UserOutput`.**

### Core Changes

1. **Container Architecture**: Store `UserOutput` directly instead of `Arc<Mutex<UserOutput>>`
2. **Function Signatures**: Change from `&Arc<Mutex<UserOutput>>` to `&mut UserOutput`
3. **ProgressReporter**: Simplify to either extend `UserOutput` directly or use lightweight borrowing patterns
4. **ExecutionContext**: Return mutable references instead of shared mutex references

### Output Ordering Strategy

For cases where ordered output is desired:

- **Default**: Rely on sequential execution (natural for this application)
- **Future**: Implement explicit buffering only when/where needed
- **Flexibility**: Allow developers to choose their output strategy explicitly

## Consequences

### Positive

- **Deadlock Impossible**: No mutexes means no deadlock scenarios
- **Simplified Codebase**: Remove `.lock().unwrap()` throughout codebase
- **Better Performance**: Zero mutex overhead, simpler memory layout
- **Easier Debugging**: Stack traces show actual logic, not mutex contention
- **Contributor Friendly**: Standard Rust ownership patterns, easier to understand
- **Compilation Safety**: Move from runtime deadlocks to compile-time ownership errors

### Negative

- **Breaking Changes**: Requires updating all function signatures using `UserOutput`
- **Output Responsibility**: Developers must be mindful of output ordering (but natural in sequential tool)
- **Migration Effort**: Significant refactoring required across the codebase

### Risks

- **Mixed Output**: Possible if future parallel operations don't use explicit ordering
- **API Changes**: All consumers of `UserOutput` need updates

## Alternatives Considered

### 1. Keep Mutex with Improved Patterns

- **Approach**: Better documentation, lint rules to prevent reentrancy
- **Rejected**: Still fragile, complexity remains, doesn't address root issue

### 2. Channel-Based Architecture

- **Approach**: Message-passing for all output operations
- **Rejected**: Over-engineered for this sequential application, adds complexity

### 3. Token-Based Access Control

- **Approach**: Runtime tokens to prevent conflicts
- **Rejected**: Runtime overhead, still complex, adds new failure modes

### 4. Phantom Locks (Compile-Time Tracking)

- **Approach**: Zero-cost compile-time lock tracking with lifetimes
- **Rejected**: Complex lifetime management, doesn't work well with async patterns

### 5. Linear Types/Move Semantics

- **Approach**: `UserOutput` can only be owned by one component at a time
- **Rejected**: Too restrictive, makes some patterns difficult

## Alternatives Considered Extended

This section provides detailed code examples for each architectural alternative considered, illustrating the implementation patterns and trade-offs.

### 1. Keep Mutex with Improved Patterns - Code Example

**Current problematic pattern:**

```rust
// Problem: Nested mutex acquisition through wrappers
pub struct CreateTemplateCommandController {
    output: Arc<Mutex<UserOutput>>,
    progress: ProgressReporter,
}

impl CreateTemplateCommandController {
    fn display_success_and_guidance(&self) -> Result<(), Error> {
        // DEADLOCK: Acquires mutex here
        let mut output = self.output.lock().unwrap();
        output.display_success("Template created successfully!");

        // Still holding mutex, calls progress.complete()
        self.progress.complete()?; // ← This tries to acquire same mutex!
        Ok(())
    }
}

pub struct ProgressReporter {
    output: Arc<Mutex<UserOutput>>,
}

impl ProgressReporter {
    pub fn complete(&self) -> Result<(), Error> {
        // DEADLOCK: Tries to acquire already-held mutex
        let mut output = self.output.lock().unwrap(); // ← Hangs here
        output.display_info("Operation completed.");
        Ok(())
    }
}
```

**Improved mutex pattern (rejected approach):**

```rust
// Better mutex patterns, but still complex
pub struct CreateTemplateCommandController {
    output: Arc<Mutex<UserOutput>>,
    progress: ProgressReporter,
}

impl CreateTemplateCommandController {
    fn display_success_and_guidance(&self) -> Result<(), Error> {
        // Scoped lock to prevent deadlock
        {
            let mut output = self.output.lock().unwrap();
            output.display_success("Template created successfully!");
        } // Lock released here

        // Now safe to call progress.complete()
        self.progress.complete()?;
        Ok(())
    }
}

// Or with timeout mechanisms
impl ProgressReporter {
    pub fn complete(&self) -> Result<(), Error> {
        let timeout = Duration::from_secs(5);
        let start = Instant::now();

        // Polling with timeout to prevent infinite hangs
        loop {
            match self.output.try_lock() {
                Ok(mut output) => {
                    output.display_info("Operation completed.");
                    return Ok(());
                }
                Err(_) if start.elapsed() > timeout => {
                    return Err(Error::UserOutputMutexTimeout);
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
            }
        }
    }
}
```

**Why rejected**: Still fragile, requires careful lock management, complex error handling.

### 2. Channel-Based Architecture - Code Example

```rust
// Message-passing approach
#[derive(Debug)]
enum OutputMessage {
    Success(String),
    Info(String),
    Error(String),
    Progress(String),
}

pub struct UserOutputChannel {
    sender: mpsc::UnboundedSender<OutputMessage>,
}

impl UserOutputChannel {
    pub fn display_success(&self, message: &str) {
        let _ = self.sender.send(OutputMessage::Success(message.to_string()));
    }

    pub fn display_info(&self, message: &str) {
        let _ = self.sender.send(OutputMessage::Info(message.to_string()));
    }
}

// Background task processes messages
async fn output_processor(mut receiver: mpsc::UnboundedReceiver<OutputMessage>) {
    while let Some(message) = receiver.recv().await {
        match message {
            OutputMessage::Success(msg) => println!("✅ {}", msg),
            OutputMessage::Info(msg) => println!("ℹ️ {}", msg),
            OutputMessage::Error(msg) => eprintln!("❌ {}", msg),
            OutputMessage::Progress(msg) => print!("\r{}", msg),
        }
    }
}

// Usage
pub struct CreateTemplateCommandController {
    output: UserOutputChannel,
    progress: ProgressReporter,
}

impl CreateTemplateCommandController {
    fn display_success_and_guidance(&self) -> Result<(), Error> {
        // No deadlock possible - just sends messages
        self.output.display_success("Template created successfully!");
        self.progress.complete()?;
        Ok(())
    }
}
```

**Why rejected**: Over-engineered for sequential operations, adds complexity without benefits.

### 3. Token-Based Access Control - Code Example

```rust
// Runtime token system
pub struct UserOutputToken(Uuid);

pub struct UserOutput {
    current_holder: Option<Uuid>,
    data: OutputData,
}

impl UserOutput {
    pub fn acquire_exclusive(&mut self) -> Result<UserOutputToken, AccessError> {
        if self.current_holder.is_some() {
            return Err(AccessError::AlreadyInUse);
        }

        let token = UserOutputToken(Uuid::new_v4());
        self.current_holder = Some(token.0);
        Ok(token)
    }

    pub fn display_success(&mut self, token: &UserOutputToken, message: &str) -> Result<(), AccessError> {
        if self.current_holder != Some(token.0) {
            return Err(AccessError::InvalidToken);
        }

        println!("✅ {}", message);
        Ok(())
    }

    pub fn release(&mut self, token: UserOutputToken) {
        if self.current_holder == Some(token.0) {
            self.current_holder = None;
        }
    }
}

// Usage
impl CreateTemplateCommandController {
    fn display_success_and_guidance(&mut self, output: &mut UserOutput) -> Result<(), Error> {
        let token = output.acquire_exclusive()?;
        output.display_success(&token, "Template created successfully!")?;

        // Must explicitly pass token to progress reporter
        self.progress.complete(output, &token)?;
        output.release(token);
        Ok(())
    }
}
```

**Why rejected**: Runtime complexity for compile-time problem, new failure modes.

### 4. Phantom Locks (Compile-Time Tracking) - Code Example

```rust
// Zero-cost compile-time exclusivity
use std::marker::PhantomData;

pub struct Locked;
pub struct Unlocked;

pub struct UserOutput<State = Unlocked> {
    data: OutputData,
    _state: PhantomData<State>,
}

impl UserOutput<Unlocked> {
    pub fn lock(self) -> UserOutput<Locked> {
        UserOutput {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl UserOutput<Locked> {
    pub fn display_success(&mut self, message: &str) {
        println!("✅ {}", message);
    }

    pub fn unlock(self) -> UserOutput<Unlocked> {
        UserOutput {
            data: self.data,
            _state: PhantomData,
        }
    }
}

// Usage
impl CreateTemplateCommandController {
    fn display_success_and_guidance(&self, output: UserOutput<Unlocked>) -> Result<UserOutput<Unlocked>, Error> {
        let mut locked_output = output.lock();
        locked_output.display_success("Template created successfully!");

        // Problem: How to share with progress reporter?
        // Would need complex lifetime management
        let output = locked_output.unlock();
        // self.progress.complete(output)?; // Borrow checker issues

        Ok(output)
    }
}
```

**Why rejected**: Complex type machinery, doesn't work well with async patterns.

### 5. Linear Types/Move Semantics - Code Example

```rust
// UserOutput can only be owned by one component at a time
pub struct UserOutput {
    data: OutputData,
}

impl UserOutput {
    pub fn display_success(mut self, message: &str) -> Self {
        println!("✅ {}", message);
        self
    }

    pub fn display_info(mut self, message: &str) -> Self {
        println!("ℹ️ {}", message);
        self
    }
}

// Threading pattern
impl CreateTemplateCommandController {
    fn display_success_and_guidance(&self, output: UserOutput) -> Result<UserOutput, Error> {
        let output = output.display_success("Template created successfully!");

        // Must thread output through progress reporter
        let output = self.progress.complete(output)?;
        Ok(output)
    }
}

impl ProgressReporter {
    pub fn complete(&self, output: UserOutput) -> Result<UserOutput, Error> {
        let output = output.display_info("Operation completed.");
        Ok(output)
    }
}
```

**Why rejected**: Too restrictive, makes borrowing patterns difficult.

### 6. Chosen Approach: Direct Ownership - Code Example

```rust
// Simple, direct ownership model
pub struct UserOutput {
    data: OutputData,
}

impl UserOutput {
    pub fn display_success(&mut self, message: &str) {
        println!("✅ {}", message);
    }

    pub fn display_info(&mut self, message: &str) {
        println!("ℹ️ {}", message);
    }
}

// Clean, simple usage
impl CreateTemplateCommandController {
    fn display_success_and_guidance(&self, output: &mut UserOutput) -> Result<(), Error> {
        output.display_success("Template created successfully!");

        // Direct access - no deadlock possible
        self.progress.complete(output)?;
        Ok(())
    }
}

impl ProgressReporter {
    pub fn complete(&self, output: &mut UserOutput) -> Result<(), Error> {
        output.display_info("Operation completed.");
        Ok(())
    }
}

// Application layer coordinates ownership
pub struct ExecutionContext {
    output: UserOutput,
    // other fields...
}

impl ExecutionContext {
    pub fn run_command<C>(&mut self, command: C) -> Result<(), Error>
    where
        C: Command,
    {
        command.execute(&mut self.output)
    }
}
```

**Why chosen**: Simple, matches usage patterns, eliminates deadlock risk, clear ownership.

## Related Decisions

- [Error Context Strategy](./error-context-strategy.md) - Error handling will be simplified without mutex errors
- [ExecutionContext Wrapper Pattern](./execution-context-wrapper.md) - ExecutionContext will need updates for new UserOutput pattern

## References

- GitHub Issue: [#164 - Refactor UserOutput to remove Arc<Mutex<>> pattern](https://github.com/torrust/torrust-tracker-deployer/issues/164)
- ProgressReporter Deadlock Investigation (2025-11-11)
- [Rust Book: Ownership and Borrowing](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [Rust Anti-patterns: Shared Mutability](https://rust-unofficial.github.io/patterns/anti_patterns/borrow_clone.html)
