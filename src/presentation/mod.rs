//! Presentation Layer
//!
//! This layer handles user-facing output and presentation concerns following DDD architecture.
//! It manages how information is presented to users, separate from internal logging and
//! application logic.
//!
//! ## 🏗️ Current Architecture (Proposal #3 Complete)
//!
//! The presentation layer follows a four-layer MVC architecture.
//! This is part of [Presentation Layer Reorganization](../../docs/refactors/plans/presentation-layer-reorganization.md).
//!
//! **Progress**: 4/6 proposals completed (67%), Proposal #4 complete - Views layer established
//!
//! ### Layer Architecture
//!
//! ```text
//! Input → Dispatch → Controllers → Views
//!   ✅       ✅         ✅         ✅
//! ```
//!
//! | Layer        | Status           | Purpose                                        |
//! |-------------|------------------|------------------------------------------------|
//! | **Input**    | ✅ Complete      | CLI argument parsing and validation           |
//! | **Dispatch** | ✅ Complete      | Command routing and execution context         |
//! | **Controllers** | ✅ Complete   | Command handling and business logic coordination |
//! | **Views**    | ✅ Complete      | Output formatting and presentation            |
//!
//! ## Current Module Structure
//!
//! ```text
//! presentation/
//! ├── input/            # ✅ Input Layer - CLI parsing and validation
//! │   └── cli/          # Clap-based argument parsing
//! │       ├── args.rs   # Global CLI arguments (logging config)
//! │       ├── commands.rs # Subcommand definitions
//! │       └── mod.rs    # Main Cli struct and parsing logic
//! │
//! ├── dispatch/         # ✅ Dispatch Layer - Routing and execution context
//! │   ├── mod.rs        # Layer exports and documentation
//! │   ├── router.rs     # Command routing logic (route_command function)
//! │   └── context.rs    # ExecutionContext wrapper around Container
//! │
//! ├── controllers/      # ✅ Controllers Layer - Command handlers (COMPLETE)
//! │   ├── create/       # Create command with subcommand controllers
//! │   │   ├── router.rs       # Routes to environment/template subcommands
//! │   │   ├── errors.rs       # Unified create command errors
//! │   │   ├── subcommands/    # Subcommand controller implementations
//! │   │   │   ├── environment/ # Environment creation controller
//! │   │   │   │   ├── handler.rs      # Main command handler
//! │   │   │   │   ├── config_loader.rs # Configuration loading
//! │   │   │   │   ├── errors.rs       # Environment-specific errors
//! │   │   │   │   └── tests.rs        # Integration tests
//! │   │   │   └── template/    # Template generation controller
//! │   │   │       ├── handler.rs      # Main command handler
//! │   │   │       ├── errors.rs       # Template-specific errors
//! │   │   │       └── tests.rs        # Integration tests
//! │   │   └── tests/          # Create command level tests
//! │   │       ├── environment.rs # Environment creation integration tests
//! │   │       └── template.rs    # Template generation integration tests
//! │   ├── destroy/      # ✅ Destroy command controller
//! │   │   ├── handler.rs      # Main command handler
//! │   │   ├── errors.rs       # Command-specific errors
//! │   │   └── tests/          # Destroy command tests
//! │   ├── constants.rs  # Shared constants across controllers
//! │   ├── tests/        # Controller layer integration tests
//! │   └── mod.rs        # Controller layer exports
//! │
//! ├── views/           # ✅ Views Layer - Output formatting and presentation
//! │   ├── progress/     # ✅ Progress indicators (moved from root)
//! │   └── ...           # User interface output and formatting
//! ├── errors.rs         # Unified error types for all commands
//! └── mod.rs            # This file - layer exports and documentation
//! ```
//!
//! ## 📋 Responsibilities by Layer
//!
//! ### ✅ Input Layer (`input/`)
//! - **CLI Parsing**: Command-line argument parsing with Clap
//! - **Input Validation**: Basic validation of user input
//! - **Command Structure**: Definition of available commands and options
//!
//! ### ✅ Dispatch Layer (`dispatch/`)
//! - **Command Routing**: Determining which controller to execute
//! - **Subcommand Routing**: For commands with subcommands (e.g., `create environment` vs `create template`)
//! - **Execution Context**: Providing dependencies through `ExecutionContext` wrapper
//! - **Service Location**: Bridge between CLI and business logic
//!
//! ### ✅ Controllers Layer (`controllers/`) - COMPLETE
//! - **Command Handling**: Business logic coordination for each command
//! - **Two Command Types**:
//!   - Single commands (e.g., `destroy`) - direct execution via handler
//!   - Commands with subcommands (e.g., `create`) - router delegates to subcommand controllers
//! - **Uniform Structure**: All controllers (single or subcommand) follow consistent patterns
//! - **Error Management**: Command-specific error types with actionable help
//! - **Application Integration**: Calling application layer services through `ExecutionContext`
//!
//! #### Command Architecture Patterns:
//!
//! **Single Commands** (Direct execution):
//! ```text
//! destroy/
//! ├── handler.rs    # handle_destroy_command()
//! ├── errors.rs     # DestroySubcommandError
//! └── tests/        # Command-specific tests
//! ```
//!
//! **Commands with Subcommands** (Router + separate controllers):
//! ```text
//! create/
//! ├── router.rs           # Routes to subcommand controllers
//! ├── errors.rs           # Shared create command errors
//! ├── subcommands/
//! │   ├── environment/    # Environment creation controller
//! │   │   ├── handler.rs          # handle_environment_creation()
//! │   │   ├── config_loader.rs    # Configuration loading logic
//! │   │   ├── errors.rs           # Environment-specific errors
//! │   │   └── tests.rs           # Integration tests
//! │   └── template/       # Template generation controller
//! │       ├── handler.rs          # handle_template_generation()
//! │       ├── errors.rs           # Template-specific errors
//! │       └── tests.rs           # Integration tests
//! └── tests/             # Create command level tests
//! ```
//!
//! **Key Insight**: All controllers follow the same internal structure (handler + errors + tests),
//! providing consistency whether they are single commands or subcommand controllers.
//!
//! #### Current Controller Status:
//! - **✅ Create Controller**: Complete with environment and template subcommand controllers
//! - **✅ Destroy Controller**: Complete single command controller
//! - **✅ All Controllers**: Follow consistent `ExecutionContext` pattern for dependency injection
//!
//! ### ⏳ Views Layer (Future)
//! - **Output Formatting**: Structuring output for users
//! - **Channel Management**: stdout/stderr separation
//! - **Progress Indicators**: User feedback during long operations
//! - **Theme Support**: Customizable output appearance
//!
//! ## 🎯 Design Principles
//!
//! - **Layered Architecture**: Clear separation of input, routing, handling, and output
//! - **Single Responsibility**: Each layer has one primary concern
//! - **Dependency Flow**: Dependencies flow inward (controllers don't know about views)
//! - **Testability**: Each layer can be tested independently
//! - **MVC Pattern**: Controllers coordinate between input and views
//! - **Error Handling**: Structured errors with tiered help system
//! - **Unix Conventions**: stdout for results, stderr for operational messages
//!
//! ## 🔄 Next Steps (Proposal #4: Views Layer)
//!
//! With the Controllers layer complete, the next phase focuses on organizing the Views layer:
//!
//! 1. ✅ **Rename `user_output/` to `views/`**: Align with MVC terminology
//! 2. ✅ **Organize view submodules**: Group related presentation concerns
//! 3. ✅ **Move `progress.rs` to `views/progress/`**: Place progress indicators with other views
//! 4. ✅ **Implement theme system**: Structured output formatting and customization
//! 5. ✅ **Channel separation**: Proper stdout/stderr management for Unix conventions
//!
//! After Proposal #4, the final steps will be:
//! - **Proposal #5**: Enhanced error presentation and help system integration
//! - **Proposal #6**: Remove any vestigial structures from the old architecture
//!
//! ## 📚 Related Documentation
//!
//! - [Presentation Layer Reorganization Plan](../../docs/refactors/plans/presentation-layer-reorganization.md)
//! - [Current Structure Analysis](../../docs/analysis/presentation-layer/current-structure.md)
//! - [Design Proposal](../../docs/analysis/presentation-layer/design-proposal.md)
//! - [Error Handling Guide](../../docs/contributing/error-handling.md)

// Core presentation modules
pub mod controllers;
pub mod dispatch;
pub mod error;
pub mod errors;
pub mod input;
pub mod views;

// Re-export commonly used presentation types for convenience
pub use controllers::create::{
    CreateCommandError, CreateEnvironmentCommandError, CreateEnvironmentTemplateCommandError,
};
pub use controllers::destroy::DestroySubcommandError;

// Re-export error handling function from error module
pub use error::handle_error;

pub use errors::CommandError;
pub use input::{Cli, Commands, GlobalArgs};
pub use views::progress::ProgressReporter;
pub use views::{Theme, UserOutput, VerbosityLevel};

#[cfg(test)]
mod tests {
    mod reentrancy_fix_test;
}
