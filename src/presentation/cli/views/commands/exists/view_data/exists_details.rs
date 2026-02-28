//! View data for the exists command.
//!
//! Re-exports the application-layer DTO as the canonical view input type.
//! The presentation layer references this module rather than importing directly
//! from the application layer.

pub use crate::application::command_handlers::exists::handler::ExistsResult;
