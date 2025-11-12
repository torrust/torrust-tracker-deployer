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
//! ## `ExecutionContext` Design
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
//! ```rust,ignore
//! use std::sync::Arc;
//! use torrust_tracker_deployer_lib::bootstrap::Container;
//!
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
//! ```rust,no_run
//! use torrust_tracker_deployer_lib::bootstrap::Container;
//! use torrust_tracker_deployer_lib::presentation::dispatch::ExecutionContext;
//!
//! # fn example() {
//! // Clear: This is specifically for command execution
//! fn handle_command(context: &ExecutionContext) {
//!     // Command execution logic
//! }
//!
//! // Less clear: Could be for bootstrapping, testing, or execution
//! fn handle_command_old(container: &Container) {
//!     // Generic container usage
//! }
//! # }
//! ```
//!
//! ### 3. Command-Specific Service Access
//!
//! `ExecutionContext` can provide command-specific convenience methods and service
//! aggregations without exposing the entire Container interface.
//!
//! For the complete rationale, see the architectural decision record:
//! [`docs/decisions/execution-context-wrapper.md`](../../../docs/decisions/execution-context-wrapper.md)
//!
//! ## Design Principles
//!
//! - **Route, Don't Execute**: This layer only routes commands, doesn't execute them
//! - **Dependency Injection**: Provide clean access to services via `ExecutionContext`
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
//! ```rust,ignore
//! use std::path::Path;
//! use std::sync::Arc;
//! use torrust_tracker_deployer_lib::bootstrap::Container;
//! use torrust_tracker_deployer_lib::presentation::dispatch::{route_command, ExecutionContext};
//! use torrust_tracker_deployer_lib::presentation::views::VerbosityLevel;
//! // Note: Commands enum requires specific action parameters in practice
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let container = Container::new(VerbosityLevel::Normal);
//! let context = ExecutionContext::new(Arc::new(container));
//! let working_dir = Path::new(".");
//!
//! // Execute a command through the dispatch layer
//! // Note: route_command is synchronous, not async
//! // Commands require proper construction with actions
//! # Ok(())
//! # }
//! ```

// Command routing module
pub mod router;

// Execution context module
pub mod context;

// Re-export main types for convenience
pub use context::ExecutionContext;
pub use router::route_command;
