//! Shared value objects and traits for the Torrust Tracker Deployer
//!
//! This package provides foundational types shared between the root crate and
//! the SDK package (`torrust-tracker-deployer-sdk`). These are validated value
//! objects and cross-cutting traits with no business logic.
//!
//! ## Types Provided
//!
//! - [`Clock`] / [`SystemClock`] — time abstraction for testability
//! - [`DomainName`] / [`DomainNameError`] — validated DNS-like domain name
//! - [`Email`] / [`EmailError`] — validated email address
//! - [`EnvironmentName`] / [`EnvironmentNameError`] — validated environment identifier
//! - [`Username`] / [`UsernameError`] — validated username
//! - [`ServiceEndpoint`] / [`InvalidServiceEndpointUrl`] — validated URL + port
//! - [`secrets`] — secret wrappers: `ApiToken`, `Password`, `PlainApiToken`, `PlainPassword`
//! - [`error`] — error infrastructure: `ErrorKind`, `Traceable`

pub mod clock;
pub mod domain_name;
pub mod email;
pub mod environment_name;
pub mod error;
pub mod secrets;
pub mod service_endpoint;
pub mod username;

// Top-level re-exports for convenience
pub use clock::{Clock, SystemClock};
pub use domain_name::{DomainName, DomainNameError};
pub use email::{Email, EmailError};
pub use environment_name::{EnvironmentName, EnvironmentNameError};
pub use error::{ErrorKind, Traceable};
pub use secrets::{ApiToken, ExposeSecret, Password, PlainApiToken, PlainPassword};
pub use service_endpoint::{InvalidServiceEndpointUrl, ServiceEndpoint};
pub use username::{Username, UsernameError};
