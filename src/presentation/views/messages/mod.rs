//! Message type implementations for user output
//!
//! This module contains all concrete message types that implement the `OutputMessage` trait.

pub use blank_line::BlankLineMessage;
pub use debug_detail::DebugDetailMessage;
pub use detail::DetailMessage;
pub use error::ErrorMessage;
pub use info_block::{InfoBlockMessage, InfoBlockMessageBuilder};
pub use progress::ProgressMessage;
pub use result::ResultMessage;
pub use steps::{StepsMessage, StepsMessageBuilder};
pub use success::SuccessMessage;
pub use warning::WarningMessage;

mod blank_line;
mod debug_detail;
mod detail;
mod error;
mod info_block;
mod progress;
mod result;
mod steps;
mod success;
mod warning;
