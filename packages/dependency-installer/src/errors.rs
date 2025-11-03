use thiserror::Error;

/// Error types for detection operations
#[derive(Debug, Error)]
pub enum DetectionError {
    #[error("Failed to detect tool '{tool}': {source}")]
    DetectionFailed {
        tool: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Command execution failed for tool '{tool}': {message}")]
    CommandFailed { tool: String, message: String },
}

/// Error types for command execution utilities
#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Failed to execute command '{command}': {source}")]
    ExecutionFailed {
        command: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Command '{command}' not found in PATH")]
    CommandNotFound { command: String },
}
