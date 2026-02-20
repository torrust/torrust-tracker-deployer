//! Docs Command Controller (Presentation Layer)
//!
//! This module handles the presentation layer concerns for CLI JSON documentation
//! generation, including user output and progress reporting.
//!
//! # Architecture
//!
//! - **Controller**: Coordinates between application handler and user output
//! - **Errors**: Presentation layer error wrapping
//! - **Progress Reporting**: Provides user feedback during generation
//!
//! # Usage
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use std::path::PathBuf;
//! use torrust_tracker_deployer_lib::presentation::controllers::docs::DocsCommandController;
//! use torrust_tracker_deployer_lib::presentation::views::{UserOutput, VerbosityLevel};
//! use std::cell::RefCell;
//! use parking_lot::ReentrantMutex;
//!
//! // Setup user output
//! let user_output = Arc::new(ReentrantMutex::new(RefCell::new(
//!     UserOutput::new(VerbosityLevel::Normal)
//! )));
//! let mut controller = DocsCommandController::new(&user_output);
//!
//! // Generate to file
//! let output_path = PathBuf::from("docs/cli/commands.json");
//! controller.execute(Some(&output_path))?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod errors;
mod handler;

pub use errors::DocsCommandError;
pub use handler::DocsCommandController;
