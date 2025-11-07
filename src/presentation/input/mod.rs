//! Input Layer - Presentation Layer Component
//!
//! The Input Layer is responsible for parsing and validating user input from various sources,
//! primarily the command-line interface. This is the first layer in the presentation layer's
//! four-layer architecture: Input → Dispatch → Controllers → Views.
//!
//! ## Purpose
//!
//! The Input Layer establishes a clear separation between:
//! - **Raw user input** (command-line arguments, configuration files)
//! - **Validated input data** ready for command dispatch
//! - **Command execution logic** (handled by other presentation layers)
//!
//! This separation provides several benefits:
//! - **Single Responsibility**: Input parsing is isolated from command execution
//! - **Testability**: Input validation can be tested independently
//! - **Flexibility**: Easy to add new input sources (web UI, API, config files)
//! - **Error Handling**: Input validation errors are handled at the appropriate layer
//!
//! ## Module Structure
//!
//! ```text
//! input/
//! ├── mod.rs     # This file - layer exports and documentation
//! └── cli/       # Command-line interface parsing (moved from presentation/cli)
//!     ├── mod.rs     # Main CLI structure and parsing logic
//!     ├── args.rs    # Global CLI arguments (logging config)
//!     └── commands.rs # Subcommand definitions
//! ```
//!
//! ## Design Principles
//!
//! - **Parse, Don't Execute**: This layer only parses and validates input
//! - **Early Validation**: Catch input errors as soon as possible
//! - **Clean Data Structures**: Provide well-typed data to subsequent layers
//! - **User-Friendly Errors**: Generate helpful error messages for invalid input
//!
//! ## Integration with Presentation Layer
//!
//! The Input Layer integrates with the broader presentation layer architecture:
//!
//! 1. **Input Layer** (this module) - Parses user input
//! 2. **Dispatch Layer** (`commands/mod.rs`) - Routes commands to handlers
//! 3. **Controller Layer** (`commands/*/handler.rs`) - Executes command logic
//! 4. **View Layer** (`user_output/`, `progress.rs`) - Presents results to users
//!
//! ## Future Enhancements
//!
//! As part of the presentation layer reorganization (Issue #154), this Input Layer
//! will serve as the foundation for:
//! - Configuration file input parsing
//! - Environment variable input handling
//! - Potential future input sources (API, web interface)
//!
//! ## Related Documentation
//!
//! - [Presentation Layer Reorganization Plan](../../docs/refactors/plans/presentation-layer-reorganization.md)
//! - [DDD Layer Placement Guide](../../docs/contributing/ddd-layer-placement.md)
//! - [Module Organization](../../docs/contributing/module-organization.md)

// CLI input parsing module
pub mod cli;

// Re-export CLI types for convenience
pub use cli::{Cli, Commands, GlobalArgs};
