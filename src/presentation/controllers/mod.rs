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
//! ## Controller Maturity Levels
//!
//! ### ✅ Reference Implementation: Destroy Controller
//! 
//! The `destroy` controller demonstrates the target architecture:
//! - **Clean Handler Pattern**: Single `handler.rs` with focused responsibility
//! - **Dedicated Error Types**: Command-specific errors with help methods
//! - **Minimal Dependencies**: Takes ExecutionContext, delegates to application layer
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
//! ### 🚧 Needs Refactoring: Create Controller
//!
//! The `create` controller needs refactoring to match the destroy pattern.
//! Currently has subcommands that should be separate controllers:
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
//! ```text
//! create_environment/  # NEW: Dedicated environment controller
//! ├── handler.rs      # Environment creation handler
//! ├── errors.rs       # Environment-specific errors
//! ├── tests/          # Environment tests
//! └── mod.rs          # Module exports
//!
//! create_template/     # NEW: Dedicated template controller
//! ├── handler.rs      # Template generation handler
//! ├── errors.rs       # Template-specific errors
//! ├── tests/          # Template tests
//! └── mod.rs          # Module exports
//! ```
//!
//! ## 🎯 Controller Design Principles
//!
//! Based on the destroy controller reference implementation:
//!
//! ### 1. Single Responsibility
//! - Each controller handles **one specific command** (destroy, create environment, create template)
//! - No routing logic within controllers - handled by dispatch layer
//! - No presentation logic - controllers coordinate, views handle output
//!
//! ### 2. Clean Handler Pattern
//! - Main logic in `handler.rs` with descriptive function name (`handle_destroy_command`)
//! - Take `ExecutionContext` for dependencies
//! - Return command-specific error types
//! - Focus on orchestrating application layer services
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
//! ## 📋 Refactoring Plan for Create Controller
//!
//! To complete Proposal #3, the create controller needs to be split:
//!
//! ### Step 1: Create Environment Controller
//! 1. Create `src/presentation/controllers/create_environment/`
//! 2. Move environment logic from `create/subcommands/environment/`
//! 3. Create clean handler following destroy pattern
//! 4. Move environment tests to new controller
//!
//! ### Step 2: Create Template Controller  
//! 1. Create `src/presentation/controllers/create_template/`
//! 2. Move template logic from `create/subcommands/template/`
//! 3. Create clean handler following destroy pattern
//! 4. Move template tests to new controller
//!
//! ### Step 3: Update Dispatch Router
//! 1. Remove create router (no more subcommand routing needed)
//! 2. Update dispatch router to call separate controllers directly
//! 3. Update error handling for separate error types
//!
//! ### Step 4: Remove Old Create Structure
//! 1. Remove `create/` directory entirely
//! 2. Update imports throughout codebase
//! 3. Update documentation and tests
//!
//! ## 🧪 Testing Strategy
//!
//! Each controller should have comprehensive test coverage:
//! - **Unit Tests**: Handler function behavior with various inputs
//! - **Error Tests**: All error variants and help text validation
//! - **Integration Tests**: End-to-end command execution with ExecutionContext
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
pub mod constants;
pub mod create;
pub mod destroy;

// Shared test utilities
#[cfg(test)]
pub mod tests;

// Future command modules will be added here:
// pub mod provision;
// pub mod configure;
// pub mod release;
