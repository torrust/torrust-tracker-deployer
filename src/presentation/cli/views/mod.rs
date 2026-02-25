//! Views Layer - User Interface Output
//!
//! This module implements the Views layer of the MVC presentation architecture, handling
//! user-facing output formatting and presentation. It provides clean separation between
//! internal logging and user interface output, implementing a dual-channel strategy
//! following Unix conventions and modern CLI best practices (similar to cargo, docker, npm):
//!
//! - **stdout (Results Channel)**: Final results, structured data, output for piping/redirection
//! - **stderr (Progress/Operational Channel)**: Progress updates, status messages, warnings, errors
//!
//! This separation enables:
//! - Clean piping: `torrust-tracker-deployer destroy env | jq .status` works correctly
//! - Automation friendly: Scripts can redirect progress to /dev/null while capturing results
//! - Unix convention compliance: Follows established patterns from modern CLI tools
//! - Better UX: Progress feedback doesn't interfere with result data
//!
//! ## Type-Safe Channel Routing
//!
//! The module uses newtype wrappers (`StdoutWriter` and `StderrWriter`) to provide compile-time
//! guarantees that messages are routed to the correct output channel. This prevents accidental
//! channel confusion and makes the code more maintainable by catching routing errors at compile
//! time rather than runtime.
//!
//! The newtype pattern is a zero-cost abstraction - it has the same memory layout and performance
//! characteristics as the wrapped type, but provides type safety benefits.
//!
//! ## Buffering Behavior
//!
//! Output is line-buffered by default. Messages are typically flushed automatically
//! after each newline. For cases where immediate output is critical (e.g., before
//! long-running operations), call `flush()` explicitly:
//!
//! ```rust,ignore
//! output.progress("Starting long operation...");
//! output.flush()?; // Ensure message appears before operation starts
//! perform_long_operation();
//! ```
//!
//! ## Example Usage
//!
//! ```rust
//! use torrust_tracker_deployer_lib::presentation::cli::views::{UserOutput, VerbosityLevel};
//!
//! let mut output = UserOutput::new(VerbosityLevel::Normal);
//!
//! // Progress messages go to stderr
//! output.progress("Destroying environment...");
//!
//! // Success status goes to stderr
//! output.success("Environment destroyed successfully");
//!
//! // Results go to stdout for piping
//! output.result(r#"{"status": "destroyed"}"#);
//! ```
//!
//! ## Channel Strategy
//!
//! Based on research from [`docs/research/UX/console-app-output-patterns.md`](../../docs/research/UX/console-app-output-patterns.md):
//!
//! - **stdout**: Deployment results, configuration summaries, structured data (JSON)
//! - **stderr**: Step progress, status updates, warnings, error messages with actionable guidance
//!
//! ## Progress Indicators
//!
//! The [`progress`] module provides components for displaying real-time progress during long operations.
//!
//! See also: [`docs/research/UX/user-output-vs-logging-separation.md`](../../docs/research/UX/user-output-vs-logging-separation.md)

// Re-export core types and traits for backward compatibility
pub use channel::Channel;
pub use formatters::JsonFormatter;
pub use messages::{
    DebugDetailMessage, DetailMessage, ErrorMessage, InfoBlockMessage, InfoBlockMessageBuilder,
    ProgressMessage, ResultMessage, StepsMessage, StepsMessageBuilder, SuccessMessage,
    WarningMessage,
};
pub use sinks::{CompositeSink, FileSink, StandardSink, TelemetrySink};
pub use theme::Theme;
pub use traits::{FormatterOverride, OutputMessage, OutputSink};
pub use user_output::UserOutput;
pub use verbosity::VerbosityLevel;

// Internal modules
mod channel;
mod formatters;
mod messages;
mod sinks;
mod theme;
mod traits;
mod user_output;
mod verbosity;

// Progress indicators module (moved from presentation root for clear ownership)
pub mod progress;

// Command-specific views (organized by command)
pub mod commands;

// Testing utilities module (public for use in tests across the codebase)
pub mod testing;
