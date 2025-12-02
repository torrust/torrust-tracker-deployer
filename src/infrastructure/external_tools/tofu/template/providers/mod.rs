//! Provider-specific `OpenTofu` template functionality.
//!
//! This module contains template implementations that are specific to
//! individual infrastructure providers (LXD, Hetzner, etc.).
//!
//! Each provider has its own independent template wrappers for:
//! - `cloud_init` - Cloud-init configuration templates
//! - `variables` - `OpenTofu` variables templates
//!
//! Templates are not shared between providers to allow provider-specific customization.

pub mod hetzner;
pub mod lxd;
