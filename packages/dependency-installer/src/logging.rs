//! Logging configuration for the dependency installer

/// Initialize tracing with the specified log level
///
/// If `level` is `None`, logging is disabled completely.
pub fn init_tracing(level: Option<tracing::Level>) {
    if let Some(max_level) = level {
        tracing_subscriber::fmt()
            .with_target(true)
            .with_thread_ids(false)
            .with_thread_names(false)
            .with_level(true)
            .with_max_level(max_level)
            .init();
    }
    // If level is None (Off), don't initialize tracing at all
}
