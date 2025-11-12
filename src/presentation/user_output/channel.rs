//! Output channel routing for user-facing messages
//!
//! This module defines the channel enum that determines whether messages
//! should be written to stdout or stderr.

/// Output channel for routing messages
///
/// Determines whether a message should be written to stdout or stderr.
/// Following Unix conventions:
/// - **stdout**: Final results and structured data for piping/redirection
/// - **stderr**: Progress updates, status messages, operational info, errors
///
/// # Examples
///
/// ```rust
/// use torrust_tracker_deployer_lib::presentation::user_output::Channel;
///
/// let channel = Channel::Stdout;
/// assert_eq!(channel, Channel::Stdout);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    /// Standard output stream for final results and data
    Stdout,
    /// Standard error stream for progress and operational messages
    Stderr,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_enum_should_support_equality() {
        assert_eq!(Channel::Stdout, Channel::Stdout);
        assert_eq!(Channel::Stderr, Channel::Stderr);
        assert_ne!(Channel::Stdout, Channel::Stderr);
    }
}
