//! Logging configuration for the dependency installer

/// Initialize tracing with default configuration
pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_level(true)
        .with_max_level(tracing::Level::INFO)
        .init();
}
