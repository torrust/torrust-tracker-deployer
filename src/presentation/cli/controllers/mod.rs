//! Controllers Layer - Presentation Layer Component
//!
//! The Controllers Layer handles command execution and business logic coordination.
//! This is the third layer in the presentation layer's four-layer architecture:
//! Input → Dispatch → **Controllers** → Views.
//!
//! ## 🚧 Current Status: Proposal #3 In Progress
//!
//! This layer is being refactored to establish consistent controller patterns.
//! The goal is to align all command controllers with the clean architecture
//! established by the destroy command controller.
//!
//! ## Command Architecture Types
//!
//! There are two types of commands in the system, each with different internal structure:
//!
//! ### 1. Single Commands (Direct Execution)
//!
//! Commands that execute directly without subcommands (e.g., `destroy`).
//! These follow the clean handler pattern with direct execution.
//!
//! ### 2. Commands with Subcommands (Router + Subcontrollers)
//!
//! Commands that route to multiple subcommands (e.g., `create environment`, `create template`).
//! These have an **extra routing layer** but subcommands maintain the same internal
//! structure as single commands.
//!
//! **Key Principle**: Subcommands should have internally the same structure as normal
//! commands, but with an additional routing layer to dispatch between subcommands.
//!
//! ## Controller Maturity Levels
//!
//! ### ✅ Reference Implementation: Destroy Controller (Single Command)
//!
//! The `destroy` controller demonstrates the target architecture for single commands:
//! - **Clean Handler Pattern**: Single `handler.rs` with focused responsibility
//! - **Dedicated Error Types**: Command-specific errors with help methods
//! - **Minimal Dependencies**: Takes `ExecutionContext`, delegates to application layer
//! - **Comprehensive Tests**: Full test coverage with clear test organization
//!
//! ```text
//! destroy/
//! ├── handler.rs      # Main command handler function
//! ├── errors.rs       # DestroySubcommandError with help methods
//! ├── tests/          # Command-specific tests
//! └── mod.rs          # Module exports
//! ```
//!
//! ### 🚧 Needs Refactoring: Create Controller (Command with Subcommands)
//!
//! The `create` controller needs refactoring to match the target pattern for commands
//! with subcommands. It demonstrates the **router + subcontroller** pattern but needs
//! architectural cleanup.
//!
//! **Current Structure (Transitional)**:
//! ```text
//! create/
//! ├── router.rs       # Routes between environment and template subcommands
//! ├── errors.rs       # Unified CreateCommandError wrapper
//! ├── subcommands/    # Temporary: should become separate controllers
//! │   ├── environment/ # Environment creation logic
//! │   └── template/    # Template generation logic
//! ├── tests/          # Tests organized by function
//! │   ├── environment.rs # Environment creation tests
//! │   └── template.rs    # Template generation tests
//! └── mod.rs          # Module exports
//! ```
//!
//! **Target Structure (After Refactoring)**:
//!
//! The refactoring will split this into separate controllers for each subcommand,
//! each following the same clean structure as single commands:
//!
//! ```text
//! create_environment/  # NEW: Dedicated environment controller
//! ├── handler.rs      # Environment creation handler (same structure as destroy)
//! ├── errors.rs       # Environment-specific errors
//! ├── tests/          # Environment tests
//! └── mod.rs          # Module exports
//!
//! create_template/     # NEW: Dedicated template controller
//! ├── handler.rs      # Template generation handler (same structure as destroy)
//! ├── errors.rs       # Template-specific errors
//! ├── tests/          # Template tests
//! └── mod.rs          # Module exports
//! ```
//!
//! **Key Insight**: Each subcommand becomes a separate controller with the same
//! internal structure as single commands. The routing is handled at the dispatch
//! layer, not within controllers.
//!
//! ## 🎯 Controller Design Principles
//!
//! Based on the destroy controller reference implementation:
//!
//! ### 1. Single Responsibility
//! - Each controller handles **one specific command** (destroy, create environment, create template)
//! - No routing logic within controllers - routing handled by dispatch layer for subcommands
//! - No presentation logic - controllers coordinate, views handle output
//!
//! ### 2. Clean Handler Pattern (Universal for All Controllers)
//! - Main logic in `handler.rs` with descriptive function name (`handle_destroy_command`)
//! - Take `ExecutionContext` for dependencies
//! - Return command-specific error types
//! - Focus on orchestrating application layer services
//!
//! **Note**: This pattern applies to both single commands and subcommand controllers.
//! Subcommand controllers have the same internal structure, just different routing.
//!
//! ### 3. Dedicated Error Types
//! - Command-specific error enums (e.g., `DestroySubcommandError`)
//! - Use thiserror for structured errors
//! - Implement `.help()` method for detailed troubleshooting
//! - Include context and actionable guidance
//!
//! ### 4. Application Layer Integration
//! - Controllers call application layer command handlers
//! - Pass through domain entities and value objects
//! - Handle application errors and convert to presentation errors
//! - Don't contain business logic - delegate to application layer
//!
//! ## 🔀 Subcommand Routing Architecture
//!
//! For commands with subcommands (like `create`), the current architecture uses:
//!
//! 1. **Dispatch Layer**: Receives full command (e.g., `create environment`)
//! 2. **Router**: Routes to appropriate subcommand handler based on action type
//! 3. **Subcommand Handler**: Executes specific logic (environment creation or template generation)
//!
//! **Current Implementation Example** (from `create/router.rs`):
//! ```ignore
//! use std::path::Path;
//! use torrust_tracker_deployer_lib::domain::provider::Provider;
//! use torrust_tracker_deployer_lib::presentation::cli::input::cli::commands::CreateAction;
//! use torrust_tracker_deployer_lib::presentation::cli::dispatch::context::ExecutionContext;
//! use torrust_tracker_deployer_lib::presentation::cli::controllers::create::errors::CreateCommandError;
//! use torrust_tracker_deployer_lib::presentation::cli::controllers::create::subcommands;
//!
//! # #[tokio::main]
//! # async fn main() {
//! # let action = todo!();
//! # let working_dir = todo!();
//! # let context = todo!();
//! pub async fn route_command(
//!     action: CreateAction,
//!     working_dir: &Path,
//!     context: &ExecutionContext,
//! ) -> Result<(), CreateCommandError> {
//!     match action {
//!         CreateAction::Environment { env_file } => {
//!             context
//!                 .container()
//!                 .create_environment_controller()
//!                 .execute(&env_file, working_dir)
//!                 .await
//!                 .map(|_| ()) // Convert Environment<Created> to ()
//!                 .map_err(CreateCommandError::Environment)
//!         }
//!         CreateAction::Template { output_path, provider } => {
//!             let template_path = output_path.unwrap_or_else(CreateAction::default_template_path);
//!             context
//!                 .container()
//!                 .create_template_controller()
//!                 .execute(&template_path, provider)
//!                 .await
//!                 .map_err(CreateCommandError::Template)
//!         }
//!         CreateAction::Schema { output_path } => {
//!             context
//!                 .container()
//!                 .create_schema_controller()
//!                 .execute(output_path.as_ref())
//!                 .map_err(CreateCommandError::Schema)
//!         }
//!     }
//! }
//! # }
//! ```
//!
//! **Target Architecture**: Move routing to dispatch layer, make each subcommand
//! a separate controller with the same structure as single commands.
//!
//! ## 📋 Refactoring Plan for Create Controller
//!
//! To complete Proposal #3, the create controller needs to be split into separate
//! controllers that each follow the same clean structure as single commands:
//!
//! ### Current Challenge
//! The `create` command currently uses internal routing with subcommands. This needs
//! to be refactored so each subcommand becomes a separate controller with the same
//! internal structure as single commands.
//!
//! ### Step 1: Create Environment Controller
//! 1. Create `src/presentation/controllers/create_environment/`
//! 2. Move environment logic from `create/subcommands/environment/`
//! 3. Create clean handler following destroy pattern (same structure)
//! 4. Move environment tests to new controller
//!
//! ### Step 2: Create Template Controller  
//! 1. Create `src/presentation/controllers/create_template/`
//! 2. Move template logic from `create/subcommands/template/`
//! 3. Create clean handler following destroy pattern (same structure)
//! 4. Move template tests to new controller
//!
//! ### Step 3: Update Dispatch Router
//! 1. Remove create controller router (no more internal subcommand routing)
//! 2. Update dispatch layer to route directly to separate controllers
//! 3. Update error handling for separate error types
//! 4. Each subcommand treated as independent controller
//!
//! ### Step 4: Remove Old Create Structure
//! 1. Remove `create/` directory entirely
//! 2. Update imports throughout codebase
//! 3. Update documentation and tests
//!
//! **Result**: Both `create environment` and `create template` will be handled by
//! separate controllers with identical structure to `destroy` - just different routing
//! at the dispatch level.
//!
//! ## 🧪 Testing Strategy
//!
//! Each controller should have comprehensive test coverage:
//! - **Unit Tests**: Handler function behavior with various inputs
//! - **Error Tests**: All error variants and help text validation
//! - **Integration Tests**: End-to-end command execution with `ExecutionContext`
//!
//! Tests should be **isolated** - each controller's tests run independently
//! without depending on other controllers or external state.
//!
//! ## 🔄 Future Controllers
//!
//! After establishing the controller pattern, additional commands will follow
//! the same structure:
//!
//! ```text
//! controllers/
//! ├── create_environment/  # Environment creation
//! ├── create_template/     # Template generation
//! ├── destroy/            # Environment destruction ✅
//! ├── provision/          # Future: Infrastructure provisioning
//! ├── configure/          # Future: Software configuration
//! ├── release/            # Future: Application deployment
//! └── run/               # Future: Service management
//! ```
//!
//! Each controller will:
//! - Follow the established handler pattern
//! - Have dedicated error types
//! - Integrate cleanly with the dispatch layer
//! - Maintain comprehensive test coverage

// Re-export command modules
pub mod configure;
pub mod constants;
pub mod create;
pub mod destroy;
pub mod docs;
pub mod exists;
pub mod list;
pub mod provision;
pub mod purge;
pub mod register;
pub mod release;
pub mod render;
pub mod run;
pub mod show;
pub mod test;
pub mod validate;

// Shared test utilities
#[cfg(test)]
pub mod tests;
