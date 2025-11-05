pub mod app;
pub mod cli;
pub mod command;
pub mod detector;
pub mod handlers;
pub mod installer;
pub mod logging;
pub mod manager;
pub mod verification;

pub use detector::{DependencyDetector, DetectionError};
pub use installer::{DependencyInstaller, InstallationError};
pub use logging::*;
pub use manager::*;
pub use verification::{verify_dependencies, DependencyVerificationError};
