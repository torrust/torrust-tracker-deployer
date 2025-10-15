//! Network diagnostic tool adapters
//!
//! This module provides adapters for network diagnostic command-line tools
//! like `netstat` and `ss`.

mod error;
mod netstat;
mod ss;

pub use error::NetworkError;
pub use netstat::NetstatClient;
pub use ss::SsClient;
