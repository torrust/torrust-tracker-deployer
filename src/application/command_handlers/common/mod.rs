//! Common utilities for command handlers
//!
//! This module provides shared functionality used across multiple command handlers
//! to reduce code duplication and improve maintainability.

pub mod endpoint_builder;
pub mod failure_context;

/// Result type for step execution in command handlers
///
/// This type alias captures the common pattern used across all command handlers
/// where step execution can fail with both an error and the step that was being
/// executed when the failure occurred.
///
/// # Type Parameters
///
/// * `T` - The success value type (e.g., `Environment<Provisioned>` or `()`)
/// * `E` - The error type (e.g., `ProvisionCommandHandlerError`)
/// * `S` - The step type (e.g., `ProvisionStep`)
///
/// # Example
///
/// ```rust,ignore
/// fn execute_with_tracking(
///     &self,
///     environment: &Environment<Provisioning>,
/// ) -> StepResult<Environment<Provisioned>, ProvisionCommandHandlerError, ProvisionStep> {
///     let current_step = ProvisionStep::RenderTemplates;
///     self.render_templates()
///         .map_err(|e| (e, current_step))?;
///     // ... more steps
/// }
/// ```
pub type StepResult<T, E, S> = Result<T, (E, S)>;
