//! Service URL Views
//!
//! This module provides compact views for rendering service URLs.
//! These views are shared between commands that need to display service URLs
//! (e.g., `run` and `show` commands).
//!
//! # Module Structure
//!
//! - `compact`: Compact view for run command (URLs only, no SSH details)
//! - `dns_hint`: DNS configuration hints for TLS environments

mod compact;
mod dns_hint;

pub use compact::CompactServiceUrlsView;
pub use dns_hint::DnsHintView;
