// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! The loader dispatch seam. [`Loader`] is the built-in implementation; a
//! consumer brings its own loader by implementing [`LoaderKind`] for its own
//! type (used as [`VersionInfo::LoaderType`]), without extending the `Loader`
//! enum.

use std::sync::Arc;

use async_trait::async_trait;
use lighty_core::QueryError;

use super::loader::Loader;
use crate::types::VersionInfo;
use crate::types::version::resolved_instance::ResolvedInstance;
use crate::types::version_metadata::VersionMetaData;

#[cfg(feature = "lighty_updater")]
use crate::loaders::lighty_updater::lighty_updater::{LIGHTY_UPDATER, LightyQuery, revalidate};
#[cfg(feature = "neoforge")]
use crate::loaders::neoforge::neoforge::{NEOFORGE, NeoForgeQuery};
#[cfg(feature = "forge")]
use crate::loaders::forge::forge::{FORGE, ForgeQuery};
#[cfg(feature = "quilt")]
use crate::loaders::quilt::quilt::{QUILT, QuiltQuery};
#[cfg(feature = "fabric")]
use crate::loaders::fabric::fabric::{FABRIC, FabricQuery};
#[cfg(feature = "vanilla")]
use crate::loaders::vanilla::vanilla::{VANILLA, VanillaQuery};

type Result<T> = std::result::Result<T, QueryError>;

/// A loader kind: resolves an instance's full metadata and the coordinates it
/// installs under. Implemented by the built-in [`Loader`] and by any
/// consumer-defined loader type used as [`VersionInfo::LoaderType`].
#[async_trait]
pub trait LoaderKind: Clone + Send + Sync + std::fmt::Debug + 'static {
    /// Resolves the full metadata for an instance of this loader. The one async method a
    /// consumer's loader must provide: fetch its own manifest and build the `Version`.
    async fn resolve_metadata<V>(&self, version: &V) -> Result<Arc<VersionMetaData>>
    where
        V: VersionInfo<LoaderType = Self>;

    /// The built-in loader this kind installs as. Drives mod resolution and the
    /// forge/neoforge post-install steps; a custom fabric-based loader returns
    /// [`Loader::Fabric`].
    fn base_loader(&self) -> Loader;

    /// Resolves the coordinates the instance really installs under. The launch pipeline only
    /// calls this when the base loader may need a Forge/NeoForge post-install (LightyUpdater
    /// included, as it may resolve to one). The built-in [`Loader`] overrides it; a custom loader
    /// needs it ONLY if its [`Self::base_loader`] is Forge/NeoForge — otherwise it is never
    /// called, so the default fails loudly rather than fabricating coordinates from a builder
    /// whose `loader_version` may hold something else entirely (a server URL, say).
    async fn resolved_instance<V>(&self, _version: &V) -> Result<ResolvedInstance>
    where
        V: VersionInfo<LoaderType = Self>,
    {
        Err(QueryError::OperationUnsupported {
            operation: "resolved_instance",
            loader: format!("{:?}", self.base_loader()),
        })
    }

    /// Drops any cached manifest held for the named instance. Defaults to a no-op for
    /// loaders that keep no cache of their own.
    async fn invalidate(&self, _version_name: &str) {}
}

#[async_trait]
impl LoaderKind for Loader {
    async fn resolve_metadata<V>(&self, version: &V) -> Result<Arc<VersionMetaData>>
    where
        V: VersionInfo<LoaderType = Loader>,
    {
        match self {
            #[cfg(feature = "vanilla")]
            Loader::Vanilla => VANILLA.get(version, VanillaQuery::VanillaBuilder).await,

            #[cfg(feature = "fabric")]
            Loader::Fabric => FABRIC.get(version, FabricQuery::FabricBuilder).await,

            #[cfg(feature = "quilt")]
            Loader::Quilt => QUILT.get(version, QuiltQuery::QuiltBuilder).await,

            #[cfg(feature = "neoforge")]
            Loader::NeoForge => NEOFORGE.get(version, NeoForgeQuery::NeoForgeBuilder).await,

            #[cfg(feature = "forge")]
            Loader::Forge => FORGE.get(version, ForgeQuery::ForgeBuilder).await,

            #[cfg(feature = "lighty_updater")]
            Loader::LightyUpdater => {
                revalidate(version).await?;
                LIGHTY_UPDATER.get(version, LightyQuery::LightyBuilder).await
            }

            _ => Err(QueryError::OperationUnsupported {
                operation: "get_metadata()",
                loader: format!("{self:?}"),
            }),
        }
    }

    async fn resolved_instance<V>(&self, version: &V) -> Result<ResolvedInstance>
    where
        V: VersionInfo<LoaderType = Loader>,
    {
        match self {
            #[cfg(feature = "lighty_updater")]
            Loader::LightyUpdater => {
                let manifest = LIGHTY_UPDATER.get_raw(version).await?;
                let info = manifest
                    .server_info
                    .as_ref()
                    .ok_or(QueryError::InvalidMetadata)?;

                Ok(ResolvedInstance::new(
                    version.name().to_string(),
                    Loader::from_server_name(info.loader())?,
                    info.loader_version().to_string(),
                    info.minecraft_version().to_string(),
                    version.game_dirs().to_path_buf(),
                    version.java_dirs().to_path_buf(),
                ))
            }

            _ => Ok(ResolvedInstance::of(version)),
        }
    }

    async fn invalidate(&self, version_name: &str) {
        match self {
            #[cfg(feature = "vanilla")]
            Loader::Vanilla => VANILLA.invalidate(version_name).await,

            #[cfg(feature = "fabric")]
            Loader::Fabric => FABRIC.invalidate(version_name).await,

            #[cfg(feature = "quilt")]
            Loader::Quilt => QUILT.invalidate(version_name).await,

            #[cfg(feature = "neoforge")]
            Loader::NeoForge => NEOFORGE.invalidate(version_name).await,

            #[cfg(feature = "forge")]
            Loader::Forge => FORGE.invalidate(version_name).await,

            #[cfg(feature = "lighty_updater")]
            Loader::LightyUpdater => LIGHTY_UPDATER.invalidate(version_name).await,

            _ => {}
        }
    }

    fn base_loader(&self) -> Loader {
        self.clone()
    }
}
