pub mod command;
pub mod detector;
pub mod errors;
pub mod manager;

pub use detector::*;
pub use errors::*;
pub use manager::*;

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
