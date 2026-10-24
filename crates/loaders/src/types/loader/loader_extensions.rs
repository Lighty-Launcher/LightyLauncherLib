use crate::types::version_metadata::VersionMetaData;
use crate::types::{Loader, LoaderKind, ResolvedInstance, VersionInfo};
use lighty_core::QueryError;
#[cfg(feature = "neoforge")]
use crate::loaders::neoforge::neoforge::{NeoForgeQuery, NEOFORGE};
#[cfg(feature = "forge")]
use crate::loaders::forge::forge::{ForgeQuery, FORGE};
#[cfg(feature = "quilt")]
use crate::loaders::quilt::quilt::{QuiltQuery, QUILT};
#[cfg(feature = "fabric")]
use crate::loaders::fabric::fabric::{FabricQuery, FABRIC};
#[cfg(feature = "vanilla")]
use crate::loaders::vanilla::vanilla::{VanillaQuery, VANILLA};
use async_trait::async_trait;
use std::sync::Arc;

pub type Result<T> = std::result::Result<T, QueryError>;

/// Generic interface for fetching metadata from different mod loaders.
///
/// [`Self::get_metadata`] dispatches to the loader via [`LoaderKind`], so a
/// consumer's own loader type plugs in without touching the `Loader` enum.
/// Specialized accessors fetch specific parts of the metadata.
#[async_trait]
pub trait LoaderExtensions {
    /// Get complete metadata for the current loader.
    async fn get_metadata(&self) -> Result<Arc<VersionMetaData>>;

    /// Get only libraries metadata.
    async fn get_libraries(&self) -> Result<Arc<VersionMetaData>>;

    /// Get main class information, already merged with the wrapped
    /// Minecraft version's the way the full builder merges it.
    async fn get_main_class(&self) -> Result<Arc<VersionMetaData>>;

    /// Get native libraries. No loader overrides them, so they always come
    /// from the wrapped Minecraft version.
    async fn get_natives(&self) -> Result<Arc<VersionMetaData>>;

    /// Get Java version requirement. No loader overrides it either.
    async fn get_java_version(&self) -> Result<Arc<VersionMetaData>>;

    /// Get assets information.
    async fn get_assets(&self) -> Result<Arc<VersionMetaData>>;
    async fn invalidate_cache(&self);

    /// The coordinates the instance really installs under.
    async fn resolved_instance(&self) -> Result<ResolvedInstance>;
}

#[async_trait]
impl<T> LoaderExtensions for T
where
    T: VersionInfo + Send + Sync,
    T::LoaderType: LoaderKind,
{
    async fn get_metadata(&self) -> Result<Arc<VersionMetaData>> {
        self.loader().resolve_metadata(self).await
    }

    async fn resolved_instance(&self) -> Result<ResolvedInstance> {
        self.loader().resolved_instance(self).await
    }

    async fn invalidate_cache(&self) {
        self.loader().invalidate(self.name()).await;
    }

    // Specialized accessors run against the base loader (the loader a custom
    // kind installs as); the full `get_metadata` is the path loaders override.
    async fn get_libraries(&self) -> Result<Arc<VersionMetaData>> {
        match self.loader().base_loader() {
            #[cfg(feature = "vanilla")]
            Loader::Vanilla => VANILLA.get(self, VanillaQuery::Libraries).await,

            #[cfg(feature = "fabric")]
            Loader::Fabric => FABRIC.get(self, FabricQuery::Libraries).await,

            #[cfg(feature = "quilt")]
            Loader::Quilt => QUILT.get(self, QuiltQuery::Libraries).await,

            #[cfg(feature = "neoforge")]
            Loader::NeoForge => NEOFORGE.get(self, NeoForgeQuery::NeoForgeBuilder).await,

            #[cfg(feature = "forge")]
            Loader::Forge => FORGE.get(self, ForgeQuery::ForgeBuilder).await,

            other => Err(QueryError::OperationUnsupported {
                operation: "get_libraries()",
                loader: format!("{other:?}"),
            }),
        }
    }

    async fn get_main_class(&self) -> Result<Arc<VersionMetaData>> {
        match self.loader().base_loader() {
            #[cfg(feature = "vanilla")]
            Loader::Vanilla => VANILLA.get(self, VanillaQuery::MainClass).await,

            #[cfg(feature = "fabric")]
            Loader::Fabric => FABRIC.get(self, FabricQuery::MainClass).await,

            #[cfg(feature = "quilt")]
            Loader::Quilt => QUILT.get(self, QuiltQuery::MainClass).await,

            #[cfg(feature = "neoforge")]
            Loader::NeoForge => NEOFORGE.get(self, NeoForgeQuery::MainClass).await,

            #[cfg(feature = "forge")]
            Loader::Forge => FORGE.get(self, ForgeQuery::MainClass).await,

            other => Err(QueryError::OperationUnsupported {
                operation: "get_main_class()",
                loader: format!("{other:?}"),
            }),
        }
    }

    async fn get_natives(&self) -> Result<Arc<VersionMetaData>> {
        #[cfg(feature = "vanilla")]
        {
            VANILLA.get(self, VanillaQuery::Natives).await
        }

        #[cfg(not(feature = "vanilla"))]
        {
            Err(QueryError::FeatureRequired {
                operation: "get_natives()",
                feature: "vanilla",
            })
        }
    }

    async fn get_java_version(&self) -> Result<Arc<VersionMetaData>> {
        #[cfg(feature = "vanilla")]
        {
            VANILLA.get(self, VanillaQuery::JavaVersion).await
        }

        #[cfg(not(feature = "vanilla"))]
        {
            Err(QueryError::FeatureRequired {
                operation: "get_java_version()",
                feature: "vanilla",
            })
        }
    }

    async fn get_assets(&self) -> Result<Arc<VersionMetaData>> {
        #[cfg(feature = "vanilla")]
        {
            VANILLA.get(self, VanillaQuery::Assets).await
        }

        #[cfg(not(feature = "vanilla"))]
        {
            Err(QueryError::FeatureRequired {
                operation: "get_assets()",
                feature: "vanilla",
            })
        }
    }
}
