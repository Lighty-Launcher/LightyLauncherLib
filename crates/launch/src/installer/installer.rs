// Copyright (c) 2025 Hamadi
// Licensed under the MIT License

//! Game installation orchestration: libraries, natives, client JAR, assets, and mods.

use super::ressources::{libraries, natives, client, assets, mods, resourcepacks, shaderpacks, datapacks};
use lighty_loaders::types::{Loader, VersionInfo, version_metadata::{Mods, Version}};
use lighty_core::{mkdir, time_it};
use lighty_modsloader::WithMods;
use crate::errors::InstallerResult;

#[cfg(feature = "events")]
use lighty_event::{EventBus, Event, LaunchEvent};

pub use self::installer_trait::Installer;

mod installer_trait {
    use super::*;
    use std::future::Future;

    /// Installation trait for version builders
    pub trait Installer {
        fn install(
            &self,
            builder: &Version,
            #[cfg(feature = "events")] event_bus: Option<&EventBus>,
        ) -> impl Future<Output = InstallerResult<()>> + Send;
    }
}

impl<T> Installer for T
where
    T: VersionInfo<LoaderType = Loader> + WithMods,
{
    /// Installs all dependencies in parallel.
    ///
    /// Resolves the optional modpack + user-attached mods (Modrinth/CurseForge)
    /// and merges them into a local clone of `builder.mods` before running the
    /// regular install pipeline.
    async fn install(
        &self,
        builder: &Version,
        #[cfg(feature = "events")] event_bus: Option<&EventBus>,
    ) -> InstallerResult<()> {
        lighty_core::trace_info!("[Installer] Starting installation for {}", self.name());

        create_directories(self).await;

        let resolved = resolve_extra_mods(
            self,
            builder,
            #[cfg(feature = "events")]
            event_bus,
        )
        .await?;
        let builder: &Version = resolved.as_ref().unwrap_or(builder);

        lighty_core::trace_info!("[Installer] Verifying installed files...");
        let mods_slice = builder.mods.as_deref().unwrap_or(&[]);
        let (
            library_tasks,
            client_task,
            asset_tasks,
            mod_tasks,
            resourcepack_tasks,
            shaderpack_tasks,
            datapack_tasks,
            (native_download_tasks, native_extract_paths),
        ) = tokio::join!(
            libraries::collect_library_tasks(self, &builder.libraries),
            client::collect_client_task(self, builder.client.as_ref()),
            assets::collect_asset_tasks(self, builder.assets.as_ref()),
            mods::collect_mod_tasks(self, mods_slice),
            resourcepacks::collect_resourcepack_tasks(self, mods_slice),
            shaderpacks::collect_shaderpack_tasks(self, mods_slice),
            datapacks::collect_datapack_tasks(self, mods_slice),
            natives::collect_native_tasks(self, builder.natives.as_deref().unwrap_or(&[])),
        );

        let total_downloads = library_tasks.len()
            + client_task.as_ref().map(|_| 1).unwrap_or(0)
            + asset_tasks.len()
            + mod_tasks.len()
            + resourcepack_tasks.len()
            + shaderpack_tasks.len()
            + datapack_tasks.len()
            + native_download_tasks.len();

        if total_downloads == 0 {
            #[cfg(feature = "events")]
            if let Some(bus) = event_bus {
                bus.emit(Event::Launch(LaunchEvent::IsInstalled {
                    version: self.name().to_string(),
                }));
            }

            lighty_core::trace_info!("[Installer] All files already up-to-date");

            // Natives still need re-extraction (they're cleaned on each run).
            if !native_extract_paths.is_empty() {
                natives::download_and_extract_natives(
                    self,
                    builder.arguments.jvm.as_deref(),
                    native_download_tasks,
                    native_extract_paths,
                    #[cfg(feature = "events")]
                    event_bus,
                )
                    .await?;
            }

            lighty_core::trace_info!("[Installer] Installation completed successfully!");
            return Ok(());
        }

        let total_bytes: u64 = library_tasks
            .iter()
            .chain(&asset_tasks)
            .chain(&mod_tasks)
            .chain(&resourcepack_tasks)
            .chain(&shaderpack_tasks)
            .chain(&datapack_tasks)
            .chain(&native_download_tasks)
            .chain(&client_task)
            .map(|task| task.size)
            .sum();

        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            bus.emit(Event::Launch(LaunchEvent::InstallStarted {
                version: self.name().to_string(),
                total_bytes,
            }));
        }

        lighty_core::trace_info!("[Installer] Downloading {} file(s)...", total_downloads);

        time_it!("Total installation", {
            tokio::try_join!(
                libraries::download_libraries(
                    library_tasks,
                    #[cfg(feature = "events")]
                    event_bus
                ),
                natives::download_and_extract_natives(
                    self,
                    builder.arguments.jvm.as_deref(),
                    native_download_tasks,
                    native_extract_paths,
                    #[cfg(feature = "events")]
                    event_bus
                ),
                mods::download_mods(
                    mod_tasks,
                    #[cfg(feature = "events")]
                    event_bus
                ),
                resourcepacks::download_resourcepacks(
                    resourcepack_tasks,
                    #[cfg(feature = "events")]
                    event_bus
                ),
                shaderpacks::download_shaderpacks(
                    shaderpack_tasks,
                    #[cfg(feature = "events")]
                    event_bus
                ),
                datapacks::download_datapacks(
                    datapack_tasks,
                    #[cfg(feature = "events")]
                    event_bus
                ),
                client::download_client(
                    client_task,
                    #[cfg(feature = "events")]
                    event_bus
                ),
                assets::download_assets(
                    asset_tasks,
                    #[cfg(feature = "events")]
                    event_bus
                ),
            )?;
        });

        #[cfg(feature = "events")]
        if let Some(bus) = event_bus {
            bus.emit(Event::Launch(LaunchEvent::InstallCompleted {
                version: self.name().to_string(),
                total_bytes,
            }));
        }

        lighty_core::trace_info!("[Installer] Installation completed successfully!");
        Ok(())
    }
}

/// Resolves the optional modpack + the user-attached mod requests concurrently,
/// merging both into a clone of `builder.mods`. Returns `None` when neither
/// produced any entries so the caller can keep the original borrow.
async fn resolve_extra_mods<T>(
    version: &T,
    builder: &Version,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<Option<Version>>
where
    T: VersionInfo<LoaderType = Loader> + WithMods,
{
    // Modpack files come first so user mods win on filename-based SHA1 dedup
    // performed downstream in `mods::collect_mod_tasks`.
    let (modpack_mods, user_mods) = tokio::try_join!(
        resolve_modpack_mods(
            version,
            #[cfg(feature = "events")]
            event_bus,
        ),
        resolve_user_mod_requests(
            version,
            #[cfg(feature = "events")]
            event_bus,
        ),
    )?;

    if modpack_mods.is_empty() && user_mods.is_empty() {
        return Ok(None);
    }

    let mut merged = builder.clone();
    let combined_capacity = modpack_mods.len() + user_mods.len();
    let slot = merged
        .mods
        .get_or_insert_with(|| Vec::with_capacity(combined_capacity));
    slot.extend(modpack_mods);
    slot.extend(user_mods);
    Ok(Some(merged))
}

/// Resolves the modpack source attached to `version` (if any) into a
/// `Vec<Mods>`. Returns an empty Vec when no modpack is configured or
/// when no mod-source feature is active.
async fn resolve_modpack_mods<T>(
    version: &T,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<Vec<Mods>>
where
    T: VersionInfo<LoaderType = Loader> + WithMods,
{
    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    {
        match version.modpack() {
            Some(source) => {
                super::ressources::modpack::process(
                    source,
                    version,
                    #[cfg(feature = "events")]
                    event_bus,
                )
                .await
            }
            None => Ok(Vec::new()),
        }
    }
    #[cfg(not(any(feature = "modrinth", feature = "curseforge")))]
    {
        let _ = version;
        Ok(Vec::new())
    }
}

/// Resolves the user-attached `ModRequest` list via the Modrinth/CurseForge
/// clients. Empty Vec when no requests are queued or no mod feature is active.
async fn resolve_user_mod_requests<T>(
    version: &T,
    #[cfg(feature = "events")] event_bus: Option<&EventBus>,
) -> InstallerResult<Vec<Mods>>
where
    T: VersionInfo<LoaderType = Loader> + WithMods,
{
    #[cfg(any(feature = "modrinth", feature = "curseforge"))]
    {
        let mod_requests = WithMods::mod_requests(version);
        if mod_requests.is_empty() {
            return Ok(Vec::new());
        }
        // The instance dictates the TTL applied to every cached fetch.
        let ttl = version.ttl();
        Ok(lighty_modsloader::resolver::resolve(
            mod_requests,
            version.minecraft_version(),
            version.loader(),
            ttl,
            #[cfg(feature = "events")]
            event_bus,
        )
        .await?)
    }
    #[cfg(not(any(feature = "modrinth", feature = "curseforge")))]
    {
        let _ = version;
        Ok(Vec::new())
    }
}

/// Creates necessary installation directories
async fn create_directories(version: &impl VersionInfo) {
    let parent_path = version.game_dirs().to_path_buf();
    // runtime_dir() defaults to game_dirs but may be rewritten by the runner
    // via set_runtime_dir() to honour arg_overrides[KEY_GAME_DIRECTORY].
    mkdir!(version.runtime_dir());
    mkdir!(parent_path.join("libraries"));
    mkdir!(parent_path.join("natives"));
    mkdir!(parent_path.join("assets").join("objects"));
}

