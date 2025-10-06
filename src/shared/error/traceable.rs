//! Traceable trait for error formatting and trace generation
//!
//! The `Traceable` trait allows errors to provide detailed trace information
//! without requiring them to implement `Serialize`. This decouples error types
//! from serialization constraints and allows custom formatting per error type.

/// Trait for errors that can generate detailed traces
///
/// This trait enables errors to provide custom formatted trace entries and
/// maintain error chain information. Unlike requiring `Serialize`, this approach
/// allows errors to contain non-serializable data while still generating
/// comprehensive trace files.
///
/// # Example
///
/// ```rust
/// use torrust_tracker_deploy::shared::error::Traceable;
///
/// #[derive(Debug, thiserror::Error)]
/// enum MyError {
///     #[error("Operation failed: {reason}")]
///     OperationFailed {
///         reason: String,
///         #[source]
///         source: std::io::Error,
///     },
/// }
///
/// impl Traceable for MyError {
///     fn trace_format(&self) -> String {
///         match self {
///             Self::OperationFailed { reason, .. } => {
///                 format!("MyError: Operation failed - {}", reason)
///             }
///         }
///     }
///
///     fn trace_source(&self) -> Option<&dyn Traceable> {
///         match self {
///             Self::OperationFailed { source, .. } => {
///                 // Would return Some if source implemented Traceable
///                 None
///             }
///         }
///     }
/// }
/// ```
pub trait Traceable: std::error::Error {
    /// Generate a formatted trace entry for this error
    ///
    /// This method should return a human-readable string describing the error
    /// with relevant context. It will be used to build the error chain in
    /// trace files.
    ///
    /// # Returns
    ///
    /// A formatted string representing this error in the trace
    fn trace_format(&self) -> String;

    /// Get the underlying source error that implements Traceable, if any
    ///
    /// This method enables walking the error chain to capture complete
    /// error information in trace files. Return `Some` if the source error
    /// implements `Traceable`, `None` otherwise.
    ///
    /// # Returns
    ///
    /// An optional reference to the source error as a `Traceable` trait object
    fn trace_source(&self) -> Option<&dyn Traceable>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, thiserror::Error)]
    enum TestError {
        #[error("Root error: {message}")]
        Root { message: String },

        #[error("Wrapped error: {context}")]
        Wrapped {
            context: String,
            #[source]
            source: Box<TestError>,
        },
    }

    impl Traceable for TestError {
        fn trace_format(&self) -> String {
            match self {
                Self::Root { message } => format!("TestError::Root - {message}"),
                Self::Wrapped { context, .. } => format!("TestError::Wrapped - {context}"),
            }
        }

        fn trace_source(&self) -> Option<&dyn Traceable> {
            match self {
                Self::Root { .. } => None,
                Self::Wrapped { source, .. } => Some(source.as_ref()),
            }
        }
    }

    #[test]
    fn it_should_format_root_error() {
        let error = TestError::Root {
            message: "test message".to_string(),
        };
        assert_eq!(error.trace_format(), "TestError::Root - test message");
    }

    #[test]
    fn it_should_format_wrapped_error() {
        let root = TestError::Root {
            message: "root cause".to_string(),
        };
        let wrapped = TestError::Wrapped {
            context: "additional context".to_string(),
            source: Box::new(root),
        };
        assert_eq!(
            wrapped.trace_format(),
            "TestError::Wrapped - additional context"
        );
    }

    #[test]
    fn it_should_provide_source_for_wrapped_error() {
        let root = TestError::Root {
            message: "root cause".to_string(),
        };
        let wrapped = TestError::Wrapped {
            context: "context".to_string(),
            source: Box::new(root),
        };
        assert!(wrapped.trace_source().is_some());
    }

    #[test]
    fn it_should_not_provide_source_for_root_error() {
        let error = TestError::Root {
            message: "test".to_string(),
        };
        assert!(error.trace_source().is_none());
    }
}
