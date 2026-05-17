//! Per-loader manifest fetching and metadata extraction.

pub mod loaders;
pub mod utils;
pub mod types;

#[cfg(feature = "fabric")]
pub use loaders::fabric;
#[cfg(feature = "forge")]
pub use loaders::forge;
#[cfg(feature = "lighty_updater")]
pub use loaders::lighty_updater;
#[cfg(feature = "neoforge")]
pub use loaders::neoforge;
pub use loaders::optifine;
#[cfg(feature = "quilt")]
pub use loaders::quilt;
#[cfg(feature = "vanilla")]
pub use loaders::vanilla;

pub use utils::{
    cache, manifest, query,
};

pub use types::{Loader, LoaderExtensions, VersionInfo, version_metadata};
