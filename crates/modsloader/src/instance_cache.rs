// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! `instance.clear_cache().await` — purges every cache entry tied to an
//! instance in a single call.

use lighty_loaders::types::{Loader, LoaderExtensions, VersionInfo};

use crate::with_mods::WithMods;

/// Cache invalidation surface for a launcher instance.
pub trait InstanceCache {
    /// Invalidates the loader manifest cache plus the Modrinth and
    /// CurseForge mod caches scoped to `(minecraft_version, loader)`.
    /// Idempotent.
    fn clear_cache(&self) -> impl std::future::Future<Output = ()> + Send;
}

impl<T> InstanceCache for T
where
    T: VersionInfo<LoaderType = Loader> + LoaderExtensions + WithMods + Send + Sync,
{
    async fn clear_cache(&self) {
        self.invalidate_cache().await;

        #[cfg(feature = "modrinth")]
        {
            let mc = self.minecraft_version().to_string();
            let loader = self.loader().clone();
            crate::modrinth::MODRINTH_CACHE
                .retain(move |k| !(k.mc == mc && k.loader == loader))
                .await;
        }
        #[cfg(feature = "curseforge")]
        {
            let mc = self.minecraft_version().to_string();
            let loader = self.loader().clone();
            crate::curseforge::CURSEFORGE_CACHE
                .retain(move |k| !(k.mc == mc && k.loader == loader))
                .await;
        }
    }
}
