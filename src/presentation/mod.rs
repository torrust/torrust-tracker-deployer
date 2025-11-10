//! Presentation Layer
//!
//! This layer handles user-facing output and presentation concerns following DDD architecture.
//! It manages how information is presented to users, separate from internal logging and
//! application logic.
//!
//! ## 🏗️ Current Architecture (Proposal #3 in Progress)
//!
//! The presentation layer is being reorganized following a four-layer MVC architecture.
//! This is part of [Presentation Layer Reorganization](../../docs/refactors/plans/presentation-layer-reorganization.md).
//!
//! **Progress**: 3/6 proposals completed (50%), currently implementing Proposal #3
//!
//! ### Layer Architecture
//!
//! ```text
//! Input → Dispatch → Controllers → Views
//!   ✅       ✅         🚧         ⏳
//! ```
//!
//! | Layer        | Status           | Purpose                                        |
//! |-------------|------------------|------------------------------------------------|
//! | **Input**    | ✅ Complete      | CLI argument parsing and validation           |
//! | **Dispatch** | ✅ Complete      | Command routing and execution context         |
//! | **Controllers** | 🚧 In Progress | Command handling and business logic coordination |
//! | **Views**    | ⏳ Planned       | Output formatting and presentation            |
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
//! ├── controllers/      # 🚧 Controllers Layer - Command handlers (IN PROGRESS)
//! │   ├── create/       # Create command controller (🚧 Needs subcontroller refactor)
//! │   │   ├── errors.rs       # Unified create command errors
//! │   │   ├── router.rs       # Create subcommand routing
//! │   │   ├── subcommands/    # Subcommand implementations
//! │   │   │   ├── environment/ # Environment creation logic
//! │   │   │   └── template/    # Template generation logic
//! │   │   └── tests/          # Create command tests
//! │   │       ├── environment.rs # Environment creation tests
//! │   │       └── template.rs    # Template generation tests
//! │   ├── destroy/      # ✅ Destroy command controller (REFERENCE IMPLEMENTATION)
//! │   │   ├── handler.rs      # Clean handler implementation
//! │   │   ├── errors.rs       # Command-specific errors
//! │   │   └── tests/          # Destroy command tests
//! │   └── mod.rs        # Controller layer exports
//! │
//! ├── user_output/      # ⏳ Future Views Layer (will be renamed to views/)
//! │   └── ...           # Output formatting and presentation
//! ├── progress.rs       # ⏳ Will move to views/progress/
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
//! - **Execution Context**: Providing dependencies through ExecutionContext wrapper
//! - **Service Location**: Bridge between CLI and business logic
//!
//! ### 🚧 Controllers Layer (`controllers/`) - IN PROGRESS
//! - **Command Handling**: Business logic coordination for each command
//! - **Error Management**: Command-specific error types and handling
//! - **Application Integration**: Calling application layer services
//!
//! #### Controller Maturity Levels:
//! - **✅ Destroy Controller**: Reference implementation with clean handler pattern
//! - **🚧 Create Controller**: Needs refactoring to match destroy pattern:
//!   - Split environment and template into separate controllers
//!   - Create dedicated handlers for each subcommand
//!   - Align with destroy's clean architecture
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
//! ## 🔄 Next Steps (Proposal #3 Completion)
//!
//! To complete the Controllers layer refactoring:
//!
//! 1. **Create Environment Controller**: Extract environment creation into dedicated controller
//! 2. **Create Template Controller**: Extract template generation into dedicated controller
//! 3. **Align with Destroy Pattern**: Follow the clean handler pattern established by destroy
//! 4. **Update Router**: Modify create router to delegate to separate controllers
//! 5. **Update Tests**: Ensure all tests pass with new controller structure
//!
//! After Proposal #3, the next steps will be:
//! - **Proposal #4**: Rename `user_output/` to `views/` with organized submodules
//! - **Proposal #5**: Move `progress.rs` into `views/progress/`
//! - **Proposal #6**: Remove vestigial old command structures
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
pub mod progress;
pub mod user_output;

// Re-export commonly used presentation types for convenience
pub use controllers::create::CreateCommandError;
pub use controllers::destroy::DestroySubcommandError;

// Re-export error handling function from error module
pub use error::handle_error;

pub use errors::CommandError;
pub use input::{Cli, Commands, GlobalArgs};
pub use progress::ProgressReporter;
pub use user_output::{Theme, UserOutput, VerbosityLevel};
