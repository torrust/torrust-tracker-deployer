//! Command Handlers Module
//!
//! This module provides command execution and error handling for CLI commands.
//!
//! **Note**: The main execution logic has been moved to the Dispatch Layer.
//! See `crate::presentation::dispatch` for the current command routing implementation.

// Re-export command modules
pub mod constants;
pub mod context;
pub mod destroy;
pub mod factory;

// Shared test utilities
#[cfg(test)]
pub mod tests;

// Future command modules will be added here:
// pub mod provision;
// pub mod configure;
// pub mod release;
