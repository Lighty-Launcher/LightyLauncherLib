//! Caching and querying primitives shared by every loader implementation.

pub mod manifest;
pub mod cache;
pub mod query;
pub mod maven;
#[cfg(any(feature = "neoforge", feature = "forge"))]
pub mod forge_installer;