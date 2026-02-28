//! Render trait and error type for command view rendering.
//!
//! This module defines the shared interface that all command view structs implement.
//! Every `JsonView` and `TextView` across all command modules implements [`Render<T>`],
//! providing compile-time enforcement of a consistent render signature.
//!
//! # Design
//!
//! The [`Render<T>`] trait is generic over the DTO type `T` so that each view module
//! can bind it to its own view-data type:
//!
//! ```rust,ignore
//! impl Render<ConfigureDetailsData> for JsonView { ... }
//! impl Render<ConfigureDetailsData> for TextView { ... }
//! ```
//!
//! Using a dedicated [`ViewRenderError`] type (rather than `serde_json::Error` directly)
//! decouples the presentation layer from the serialization backend. If the backend ever
//! changes, only this module and its `From` impls need updating — not every call site.
//!
//! # Error handling
//!
//! Text renderers always return `Ok` — they do pure string formatting and never fail.
//! JSON renderers call `serde_json::to_string_pretty`, which is expected to succeed for
//! the plain `#[derive(Serialize)]` DTOs used in this project, but serialization errors
//! are still possible (e.g. non-finite floats, non-string map keys, custom `Serialize`
//! impls) and are propagated as [`ViewRenderError`].

/// Error produced by a [`Render`] implementation.
///
/// Using a dedicated error type decouples the presentation layer from the
/// serialization library. If the backend ever changes, only this type and
/// its `From` impls need updating — not every call site.
#[derive(Debug, thiserror::Error)]
pub enum ViewRenderError {
    /// JSON serialization failed.
    #[error("JSON serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Trait for rendering command output data into a string.
///
/// Implementors transform a DTO (`T`) into a displayable or parseable string.
/// The `Result` return type is required even for infallible renderers (e.g., [`TextView`](super::commands::configure::views::text_view::TextView))
/// so that all renderers share a uniform interface and callers can use `?` unconditionally.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::presentation::cli::views::{Render, ViewRenderError};
///
/// // JSON view — calls serde_json, theoretically fallible
/// impl Render<ConfigureDetailsData> for JsonView {
///     fn render(data: &ConfigureDetailsData) -> Result<String, ViewRenderError> {
///         Ok(serde_json::to_string_pretty(data)?)
///     }
/// }
///
/// // Text view — pure string formatting, always Ok
/// impl Render<ConfigureDetailsData> for TextView {
///     fn render(data: &ConfigureDetailsData) -> Result<String, ViewRenderError> {
///         Ok(format!("Configuration completed for '{}'", data.environment_name))
///     }
/// }
/// ```
pub trait Render<T> {
    /// Render `data` into a string representation.
    ///
    /// # Errors
    ///
    /// Returns a [`ViewRenderError`] if rendering fails. Text renderers always
    /// return `Ok`; JSON renderers return `Err` only if serialization fails
    /// (which is unreachable for plain `#[derive(Serialize)]` types).
    fn render(data: &T) -> Result<String, ViewRenderError>;
}
