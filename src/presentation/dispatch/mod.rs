//! Dispatch Layer - Presentation Layer Component
//!
//! The Dispatch Layer is responsible for routing parsed commands to their appropriate handlers
//! and providing execution context for command processing. This is the second layer in the
//! presentation layer's four-layer architecture: Input → Dispatch → Controllers → Views.
//!
//! ## Purpose
//!
//! The Dispatch Layer establishes clear separation between:
//! - **Command routing** (determining which handler to execute)
//! - **Execution context** (providing dependencies to handlers)
//! - **Command execution** (actual command logic in Controllers layer)
//! - **Result presentation** (handled by Views layer)
//!
//! This separation provides several benefits:
//! - **Single Responsibility**: Routing logic isolated from command execution
//! - **Testability**: Router can be tested independently of command handlers
//! - **Dependency Injection**: Clean pattern for providing services to commands
//! - **Scalability**: Easy to add new commands without modifying existing handlers
//!
//! ## Module Structure
//!
//! ```text
//! dispatch/
//! ├── mod.rs       # This file - layer exports and documentation
//! ├── router.rs    # Command routing logic (route_command function)
//! └── context.rs   # ExecutionContext wrapper around Container
//! ```
//!
//! ## ExecutionContext Design
//!
//! The Dispatch Layer uses an `ExecutionContext` wrapper around the `Container` rather than
//! passing the `Container` directly to command handlers. This design choice provides several
//! important benefits:
//!
//! ### 1. Future-Proof Command Signatures
//!
//! By using `ExecutionContext`, we can extend execution context in the future without
//! breaking existing command handler signatures:
//!
//! ```rust
//! pub struct ExecutionContext {
//!     container: Arc<Container>,
//!     // Future additions without breaking changes:
//!     // request_id: RequestId,
//!     // execution_metadata: ExecutionMetadata,
//!     // tracing_context: TracingContext,
//!     // user_permissions: UserPermissions,
//! }
//! ```
//!
//! ### 2. Clear Abstraction and Intent
//!
//! `ExecutionContext` represents "everything a command needs to execute" rather than
//! exposing dependency injection mechanics directly:
//!
//! ```rust
//! // Clear: This is specifically for command execution
//! fn handle_command(context: &ExecutionContext)
//!
//! // Less clear: Could be for bootstrapping, testing, or execution
//! fn handle_command(container: &Container)
//! ```
//!
//! ### 3. Command-Specific Service Access
//!
//! ExecutionContext can provide command-specific convenience methods and service
//! aggregations without exposing the entire Container interface.
//!
//! For the complete rationale, see the architectural decision record:
//! [`docs/decisions/execution-context-wrapper.md`](../../../docs/decisions/execution-context-wrapper.md)
//!
//! ## Design Principles
//!
//! - **Route, Don't Execute**: This layer only routes commands, doesn't execute them
//! - **Dependency Injection**: Provide clean access to services via ExecutionContext
//! - **Type Safety**: Use strongly-typed routing with match statements
//! - **Error Propagation**: Pass routing errors up to caller
//!
//! ## Integration with Presentation Layer
//!
//! The Dispatch Layer integrates with the broader presentation layer architecture:
//!
//! 1. **Input Layer** (`input/`) - Parses user input into Commands enum
//! 2. **Dispatch Layer** (this module) - Routes commands and provides context
//! 3. **Controller Layer** (`commands/`) - Executes command logic with context
//! 4. **View Layer** (`user_output/`, `progress.rs`) - Presents results to users
//!
//! ## Usage Pattern
//!
//! ```rust
//! use crate::presentation::dispatch::{route_command, ExecutionContext};
//! use crate::presentation::input::Commands;
//! use crate::bootstrap::Container;
//!
//! // Create execution context from dependency injection container
//! let container = Container::new(/* ... */);
//! let context = ExecutionContext::new(container);
//!
//! // Route command to appropriate handler
//! match route_command(&command, &context).await {
//!     Ok(()) => println!("Command completed successfully"),
//!     Err(e) => eprintln!("Command failed: {}", e),
//! }
//! ```
//!
//! ## Future Enhancements
//!
//! As part of the presentation layer reorganization (Issue #154), this Dispatch Layer
//! will serve as the foundation for:
//!
//! - Middleware support (logging, timing, authentication)
//! - Command validation and preprocessing
//! - Async command execution patterns
//! - Command composition and chaining
//!
//! ## Related Documentation
//!
//! - [Presentation Layer Reorganization Plan](../../docs/refactors/plans/presentation-layer-reorganization.md)
//! - [DDD Layer Placement Guide](../../docs/contributing/ddd-layer-placement.md)
//! - [Container Pattern Documentation](../../docs/analysis/presentation-layer/design-proposal.md#dependency-injection-with-container)

// Command routing module
pub mod router;

// Execution context module
pub mod context;

// Re-export main types for convenience
pub use context::ExecutionContext;
pub use router::route_command;
