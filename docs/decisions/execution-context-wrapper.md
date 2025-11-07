# Decision: ExecutionContext Wrapper Pattern

## Status

Accepted

## Date

2025-11-07

## Context

During the implementation of the Dispatch Layer (Proposal 2 from the presentation layer reorganization epic #154), we needed to decide how command handlers should access application services. We had two main options:

1. **Direct Container Access**: Pass the `Container` directly to command handlers
2. **ExecutionContext Wrapper**: Create an `ExecutionContext` wrapper around the `Container`

### Current Architecture

The Dispatch Layer routes commands to handlers and needs to provide access to application services (user output, repositories, external tool clients, etc.). These services are managed by the dependency injection `Container`.

### Design Options Considered

#### Option 1: Direct Container Access

```rust
pub fn route_command(
    command: Commands,
    working_dir: &Path,
    container: &Container,
) -> Result<(), CommandError>

// In handlers:
fn handle_create_command(container: &Container) {
    let user_output = container.user_output();
    // ...
}
```

#### Option 2: ExecutionContext Wrapper

```rust
pub struct ExecutionContext {
    container: Arc<Container>,
}

pub fn route_command(
    command: Commands,
    working_dir: &Path,
    context: &ExecutionContext,
) -> Result<(), CommandError>

// In handlers:
fn handle_create_command(context: &ExecutionContext) {
    let user_output = context.user_output();
    // ...
}
```

## Decision

We chose **Option 2: ExecutionContext Wrapper** for the following reasons:

### 1. Future-Proof Command Signatures

By introducing `ExecutionContext`, we can add execution-related data in the future without breaking existing command handler signatures:

```rust
pub struct ExecutionContext {
    container: Arc<Container>,
    // Future additions without breaking changes:
    // request_id: RequestId,
    // execution_metadata: ExecutionMetadata,
    // tracing_context: TracingContext,
    // user_permissions: UserPermissions,
    // execution_timeout: Duration,
}
```

If we used `Container` directly, adding any execution context would require changing every command handler signature.

### 2. Clear Abstraction and Intent

`ExecutionContext` provides a logical abstraction for "everything a command needs to execute" rather than exposing the dependency injection container directly:

- **Container**: Implementation detail for dependency injection
- **ExecutionContext**: Execution abstraction for command handlers

This makes the intent clearer and separates concerns properly.

### 3. Type Safety and Interface Clarity

```rust
// Less clear: What is this container for? Bootstrapping? Testing? Execution?
fn handle_command(container: &Container)

// Clear: This is specifically for command execution
fn handle_command(context: &ExecutionContext)
```

### 4. Command-Specific Service Aggregation

ExecutionContext can provide command-specific convenience methods and service aggregations:

```rust
impl ExecutionContext {
    // Direct service access
    pub fn user_output(&self) -> &Arc<Mutex<UserOutput>> {
        self.container.user_output()
    }

    // Future: Command-specific aggregated services
    pub fn deployment_services(&self) -> DeploymentServices {
        DeploymentServices {
            provisioner: self.container.provisioner(),
            configurator: self.container.configurator(),
            validator: self.container.validator(),
        }
    }
}
```

### 5. Enhanced Testability

Different execution contexts can be created for different scenarios:

```rust
// Production context
let context = ExecutionContext::new(container);

// Test context with mocks
let context = TestExecutionContext::new(mock_container);

// Both can implement the same interface
trait ExecutionContextTrait {
    fn user_output(&self) -> &Arc<Mutex<UserOutput>>;
}
```

### 6. Industry Pattern Alignment

Most frameworks use execution context patterns:

- **Spring Framework**: `ApplicationContext`
- **ASP.NET Core**: `HttpContext`
- **Express.js**: Request/Response context
- **Go**: `context.Context`

This aligns with established patterns for managing execution state.

## Consequences

### Positive

- **Future-Proof**: Can extend execution context without breaking command signatures
- **Clear Intent**: ExecutionContext clearly indicates its purpose for command execution
- **Better Abstraction**: Separates execution concerns from dependency injection mechanics
- **Enhanced Testability**: Enables different contexts for different testing scenarios
- **Industry Alignment**: Follows established patterns from major frameworks

### Negative

- **Initial Overhead**: Currently just a thin wrapper around Container
- **Additional Indirection**: One extra layer between commands and services
- **Learning Curve**: New developers need to understand the wrapper pattern

### Migration Path

If needed, migration from Container to ExecutionContext (or vice versa) is straightforward:

```rust
// From Container to ExecutionContext
fn handle_command(container: &Container) -> fn handle_command(context: &ExecutionContext)

// From ExecutionContext to Container
fn handle_command(context: &ExecutionContext) -> fn handle_command(container: &Container)
```

## Implementation Details

### Current Implementation

```rust
pub struct ExecutionContext {
    container: Arc<Container>,
}

impl ExecutionContext {
    pub fn new(container: Arc<Container>) -> Self {
        Self { container }
    }

    pub fn container(&self) -> &Container {
        &self.container
    }

    pub fn user_output(&self) -> &Arc<Mutex<UserOutput>> {
        self.container.user_output()
    }
}
```

### Usage Pattern

```rust
// In bootstrap/app.rs
let container = Arc::new(Container::new());
let context = ExecutionContext::new(container);

// In dispatch layer
route_command(command, working_dir, &context)?;

// In command handlers
fn handle_create_command(context: &ExecutionContext) {
    let user_output = context.user_output();
    // Command implementation
}
```

## Related Decisions

- [Presentation Layer Reorganization](../decisions/README.md) - Overall context for the four-layer architecture
- [Command State Return Pattern](./command-state-return-pattern.md) - How commands return typed states

## References

- [Epic #154: Presentation Layer Reorganization](https://github.com/torrust/torrust-tracker-deployer/issues/154)
- [Issue #156: Create Dispatch Layer](https://github.com/torrust/torrust-tracker-deployer/issues/156)
- [Spring Framework ApplicationContext](https://docs.spring.io/spring-framework/docs/current/reference/html/core.html#beans-introduction)
- [ASP.NET Core HttpContext](https://docs.microsoft.com/en-us/aspnet/core/fundamentals/http-context)
- [Go Context Package](https://pkg.go.dev/context)
